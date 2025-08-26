use crate::events::{AppEvent, EventBus, StorageEvent};
use crate::network::{ChatMessage, Contact, DeliveryStatus};
use crate::storage::{
    BackupInfo, BackupStatus, BackupType, ChatMetadata, ChatStorage, StorageConfig, StorageError,
    StorageHealth, StorageOptimizationResult, StorageStats,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct StorageManager {
    config: StorageConfig,
    data_path: PathBuf,
    event_bus: EventBus,
    chat_storage: Arc<RwLock<ChatStorage>>,
    contacts: Arc<RwLock<HashMap<String, Contact>>>,
    stats: Arc<RwLock<StorageStats>>,
}

impl StorageManager {
    pub fn new(data_path: &PathBuf, event_bus: EventBus) -> Result<Self, StorageError> {
        let config = StorageConfig::default();

        // Ensure data directory exists
        if !data_path.exists() {
            std::fs::create_dir_all(data_path)
                .map_err(|e| StorageError::PermissionDenied(e.to_string()))?;
        }

        Ok(Self {
            config,
            data_path: data_path.clone(),
            event_bus,
            chat_storage: Arc::new(RwLock::new(ChatStorage::new())),
            contacts: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(StorageStats::new())),
        })
    }

    pub async fn initialize(&self) -> Result<(), String> {
        // Load existing data
        self.load_chats().await.map_err(|e| e.to_string())?;
        self.load_contacts().await.map_err(|e| e.to_string())?;
        self.update_stats().await.map_err(|e| e.to_string())?;

        println!("Storage manager initialized successfully");
        Ok(())
    }

    pub async fn save_message(
        &self,
        chat_id: &str,
        message: &ChatMessage,
    ) -> Result<(), StorageError> {
        let mut chat_storage = self.chat_storage.write().await;
        chat_storage.add_message(chat_id, message.clone());

        // Save to disk
        self.save_chats_to_disk(&chat_storage).await?;
        self.update_stats().await?;

        self.event_bus
            .emit(AppEvent::Storage(StorageEvent::ChatHistorySaved {
                chat_id: chat_id.to_string(),
                message_count: 1,
            }));

        Ok(())
    }

    pub async fn get_messages(&self, chat_id: &str) -> Result<Vec<ChatMessage>, StorageError> {
        let chat_storage = self.chat_storage.read().await;
        Ok(chat_storage.get_messages(chat_id))
    }

    pub async fn delete_message(&self, message_id: &str) -> Result<(), StorageError> {
        let mut chat_storage = self.chat_storage.write().await;
        let mut message_found = false;

        for (chat_id, messages) in chat_storage.messages.iter_mut() {
            if let Some(pos) = messages.iter().position(|m| m.id == message_id) {
                messages.remove(pos);
                message_found = true;
                break;
            }
        }

        if !message_found {
            return Err(StorageError::NotFound(format!(
                "Message {} not found",
                message_id
            )));
        }

        self.save_chats_to_disk(&chat_storage).await?;
        self.update_stats().await?;

        Ok(())
    }

    pub async fn delete_chat(&self, chat_id: &str) -> Result<(), StorageError> {
        let mut chat_storage = self.chat_storage.write().await;
        chat_storage.messages.remove(chat_id);
        chat_storage.metadata.remove(chat_id);

        self.save_chats_to_disk(&chat_storage).await?;
        self.update_stats().await?;

        Ok(())
    }

    pub async fn update_message_status(
        &self,
        message_id: &str,
        new_status: DeliveryStatus,
    ) -> Result<(), StorageError> {
        let mut chat_storage = self.chat_storage.write().await;
        let mut message_found = false;

        for (_, messages) in chat_storage.messages.iter_mut() {
            if let Some(message) = messages.iter_mut().find(|m| m.id == message_id) {
                message.delivery_status = new_status;
                message_found = true;
                break;
            }
        }

        if !message_found {
            return Err(StorageError::NotFound(format!(
                "Message {} not found",
                message_id
            )));
        }

        self.save_chats_to_disk(&chat_storage).await?;
        Ok(())
    }

    pub async fn save_contact(&self, contact: &Contact) -> Result<(), StorageError> {
        let mut contacts = self.contacts.write().await;
        contacts.insert(contact.id.clone(), contact.clone());

        self.save_contacts_to_disk(&contacts).await?;
        self.update_stats().await?;

        Ok(())
    }

    pub async fn get_contacts(&self) -> Result<Vec<Contact>, StorageError> {
        let contacts = self.contacts.read().await;
        Ok(contacts.values().cloned().collect())
    }

    pub async fn delete_contact(&self, contact_id: &str) -> Result<(), StorageError> {
        let mut contacts = self.contacts.write().await;
        contacts
            .remove(contact_id)
            .ok_or_else(|| StorageError::NotFound(format!("Contact {} not found", contact_id)))?;

        self.save_contacts_to_disk(&contacts).await?;
        self.update_stats().await?;

        Ok(())
    }

    pub async fn backup(&self) -> Result<String, StorageError> {
        let backup_id = uuid::Uuid::new_v4().to_string();
        let backup_path = self
            .data_path
            .join("backups")
            .join(format!("{}.backup", backup_id));

        // Ensure backup directory exists
        if let Some(parent) = backup_path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| StorageError::PermissionDenied(e.to_string()))?;
        }

        let chat_storage = self.chat_storage.read().await;
        let contacts = self.contacts.read().await;

        let backup_data = BackupData {
            chat_storage: (*chat_storage).clone(),
            contacts: (*contacts).clone(),
            created_at: Utc::now(),
        };

        let serialized = serde_json::to_string_pretty(&backup_data)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        tokio::fs::write(&backup_path, serialized)
            .await
            .map_err(|e| StorageError::PermissionDenied(e.to_string()))?;

        self.event_bus
            .emit(AppEvent::Storage(StorageEvent::BackupCreated {
                file_path: backup_path.to_string_lossy().to_string(),
            }));

        Ok(backup_path.to_string_lossy().to_string())
    }

    pub async fn restore_from_backup(&self, backup_path: &str) -> Result<(), StorageError> {
        let backup_data = tokio::fs::read_to_string(backup_path)
            .await
            .map_err(|e| StorageError::FileNotFound(e.to_string()))?;

        let backup: BackupData = serde_json::from_str(&backup_data)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        // Restore data
        let mut chat_storage = self.chat_storage.write().await;
        *chat_storage = backup.chat_storage;

        let mut contacts = self.contacts.write().await;
        *contacts = backup.contacts;

        // Save restored data
        self.save_chats_to_disk(&chat_storage).await?;
        self.save_contacts_to_disk(&contacts).await?;
        self.update_stats().await?;

        Ok(())
    }

    pub async fn get_stats(&self) -> Result<StorageStats, StorageError> {
        let stats = self.stats.read().await;
        Ok(stats.clone())
    }

    pub async fn validate_chats(&self) -> Result<Vec<String>, StorageError> {
        let mut issues = Vec::new();
        let chat_storage = self.chat_storage.read().await;

        for (chat_id, messages) in &chat_storage.messages {
            if chat_id.is_empty() {
                issues.push("Empty chat ID found".to_string());
            }

            for message in messages {
                if message.id.is_empty() {
                    issues.push(format!("Message with empty ID in chat {}", chat_id));
                }

                if message.from.is_empty() {
                    issues.push(format!("Message with empty sender in chat {}", chat_id));
                }

                if message.timestamp == 0 {
                    issues.push(format!(
                        "Message with invalid timestamp in chat {}",
                        chat_id
                    ));
                }
            }
        }

        Ok(issues)
    }

    pub async fn export_chat_data(
        &self,
        chat_id: &str,
        format: &str,
    ) -> Result<String, StorageError> {
        let messages = self.get_messages(chat_id).await?;

        match format.to_lowercase().as_str() {
            "json" => serde_json::to_string_pretty(&messages)
                .map_err(|e| StorageError::SerializationError(e.to_string())),
            "csv" => {
                let mut csv_data = String::from("timestamp,from,to,content,type,status\n");
                for message in messages {
                    csv_data.push_str(&format!(
                        "{},{},{},{},{:?},{:?}\n",
                        message.timestamp,
                        message.from,
                        message.to,
                        message.content.replace(',', ";"),
                        message.msg_type,
                        message.delivery_status
                    ));
                }
                Ok(csv_data)
            }
            "txt" => {
                let mut text_data = String::new();
                for message in messages {
                    let time = chrono::DateTime::from_timestamp(message.timestamp as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "Unknown time".to_string());

                    text_data.push_str(&format!(
                        "[{}] {}: {}\n",
                        time, message.from, message.content
                    ));
                }
                Ok(text_data)
            }
            _ => Err(StorageError::SerializationError(
                "Unsupported format".to_string(),
            )),
        }
    }

    // Private helper methods

    async fn load_chats(&self) -> Result<(), StorageError> {
        let chat_file = self.data_path.join("chats.json");

        if chat_file.exists() {
            let content = tokio::fs::read_to_string(&chat_file)
                .await
                .map_err(|e| StorageError::FileNotFound(e.to_string()))?;

            let chat_storage: ChatStorage = serde_json::from_str(&content)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;

            let mut current_storage = self.chat_storage.write().await;
            *current_storage = chat_storage;
        }

        Ok(())
    }

    async fn save_chats(&self) -> Result<(), StorageError> {
        let chat_storage = self.chat_storage.read().await;
        self.save_chats_to_disk(&chat_storage).await
    }

    async fn save_chats_to_disk(&self, chat_storage: &ChatStorage) -> Result<(), StorageError> {
        let chat_file = self.data_path.join("chats.json");

        let content = serde_json::to_string_pretty(chat_storage)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        tokio::fs::write(&chat_file, content)
            .await
            .map_err(|e| StorageError::PermissionDenied(e.to_string()))?;

        Ok(())
    }

    async fn load_contacts(&self) -> Result<(), StorageError> {
        let contacts_file = self.data_path.join("contacts.json");

        if contacts_file.exists() {
            let content = tokio::fs::read_to_string(&contacts_file)
                .await
                .map_err(|e| StorageError::FileNotFound(e.to_string()))?;

            let contacts: HashMap<String, Contact> = serde_json::from_str(&content)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;

            let mut current_contacts = self.contacts.write().await;
            *current_contacts = contacts;
        }

        Ok(())
    }

    async fn save_contacts_to_disk(
        &self,
        contacts: &HashMap<String, Contact>,
    ) -> Result<(), StorageError> {
        let contacts_file = self.data_path.join("contacts.json");

        let content = serde_json::to_string_pretty(contacts)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        tokio::fs::write(&contacts_file, content)
            .await
            .map_err(|e| StorageError::PermissionDenied(e.to_string()))?;

        Ok(())
    }

    async fn update_stats(&self) -> Result<(), StorageError> {
        let mut stats = self.stats.write().await;

        let chat_storage = self.chat_storage.read().await;
        let contacts = self.contacts.read().await;

        stats.update_message_count(chat_storage.get_total_message_count());
        stats.update_chat_count(chat_storage.messages.len() as u32);
        stats.update_contact_count(contacts.len() as u32);

        // Calculate total size (approximation)
        let chat_size = serde_json::to_string(&*chat_storage)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?
            .len() as u64;

        let contacts_size = serde_json::to_string(&*contacts)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?
            .len() as u64;

        stats.update_size(chat_size + contacts_size);

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackupData {
    chat_storage: ChatStorage,
    contacts: HashMap<String, Contact>,
    created_at: DateTime<Utc>,
}
