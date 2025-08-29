use crate::events::types::{AppEvent, EventBus, StorageEvent};
use crate::network::{ChatMessage, Contact, DeliveryStatus};
use crate::storage::types::*;
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
    pub fn new(data_path: &Path, event_bus: EventBus) -> Result<Self, StorageError> {
        let config = StorageConfig::default();

        // Ensure data directory exists
        if !data_path.exists() {
            std::fs::create_dir_all(data_path)
                .map_err(|e| StorageError::PermissionDenied(e.to_string()))?;
        }

        Ok(Self {
            config,
            data_path: data_path.to_path_buf(),
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

        for (_, messages) in chat_storage.messages.iter_mut() {
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
        self.event_bus
            .emit(AppEvent::Storage(StorageEvent::ChatHistorySaved {
                chat_id: "various".to_string(),
                message_count: 1,
            }));
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

    pub async fn get_messages_by_status(
        &self,
        chat_id: &str,
        status: DeliveryStatus,
    ) -> Result<Vec<ChatMessage>, StorageError> {
        let chat_storage = self.chat_storage.read().await;
        if let Some(messages) = chat_storage.messages.get(chat_id) {
            let filtered_messages: Vec<ChatMessage> = messages
                .iter()
                .filter(|m| m.delivery_status == status)
                .cloned()
                .collect();
            Ok(filtered_messages)
        } else {
            Ok(vec![])
        }
    }

    pub async fn get_failed_messages(&self) -> Result<Vec<ChatMessage>, StorageError> {
        let mut failed_messages = Vec::new();
        let chat_storage = self.chat_storage.read().await;

        for (_, messages) in &chat_storage.messages {
            for message in messages {
                if matches!(message.delivery_status, DeliveryStatus::Failed) {
                    failed_messages.push(message.clone());
                }
            }
        }

        Ok(failed_messages)
    }

    pub async fn cleanup_old_messages(&self, days: u32) -> Result<u32, StorageError> {
        let cutoff_time = chrono::Utc::now().timestamp() as u64 - (days as u64 * 24 * 60 * 60);
        let mut removed_count = 0;
        let mut chat_storage = self.chat_storage.write().await;

        for (_, messages) in chat_storage.messages.iter_mut() {
            let original_len = messages.len();
            messages.retain(|m| m.timestamp >= cutoff_time);
            removed_count += (original_len - messages.len()) as u32;
        }

        if removed_count > 0 {
            self.save_chats_to_disk(&chat_storage).await?;
            drop(chat_storage);
            self.update_stats().await?;

            self.event_bus
                .emit(AppEvent::Storage(StorageEvent::CleanupCompleted {
                    removed_items: removed_count as usize,
                }));
        }

        Ok(removed_count)
    }

    pub async fn get_chat_size(&self, chat_id: &str) -> Result<u64, StorageError> {
        let chat_storage = self.chat_storage.read().await;
        if let Some(messages) = chat_storage.messages.get(chat_id) {
            let size = serde_json::to_string(messages)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?
                .len() as u64;
            Ok(size)
        } else {
            Ok(0)
        }
    }

    pub async fn optimize_storage(&self) -> Result<StorageOptimizationResult, StorageError> {
        let stats = self.stats.read().await;
        let original_size = stats.total_size_bytes;
        drop(stats);

        self.compact_messages().await?;
        self.remove_duplicate_messages().await?;
        self.update_stats().await?;

        let stats = self.stats.read().await;
        let new_size = stats.total_size_bytes;
        let space_saved = original_size.saturating_sub(new_size);

        Ok(StorageOptimizationResult {
            original_size_bytes: original_size,
            optimized_size_bytes: new_size,
            space_saved_bytes: space_saved,
            messages_deduplicated: 0,
            optimization_time: chrono::Utc::now(),
        })
    }

    async fn compact_messages(&self) -> Result<(), StorageError> {
        let mut chat_storage = self.chat_storage.write().await;
        for (_, messages) in chat_storage.messages.iter_mut() {
            messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        }
        Ok(())
    }

    async fn remove_duplicate_messages(&self) -> Result<u32, StorageError> {
        let mut removed_count = 0;
        let mut chat_storage = self.chat_storage.write().await;

        for (_, messages) in chat_storage.messages.iter_mut() {
            let mut seen_ids = std::collections::HashSet::new();
            let original_len = messages.len();

            messages.retain(|message| {
                if seen_ids.contains(&message.id) {
                    false
                } else {
                    seen_ids.insert(message.id.clone());
                    true
                }
            });

            removed_count += (original_len - messages.len()) as u32;
        }

        if removed_count > 0 {
            self.save_chats_to_disk(&chat_storage).await?;
        }

        Ok(removed_count)
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

    pub async fn validate_contacts(&self) -> Result<Vec<String>, StorageError> {
        let mut issues = Vec::new();
        let contacts = self.contacts.read().await;

        for (contact_id, contact) in contacts.iter() {
            if contact.id != *contact_id {
                issues.push(format!(
                    "Contact ID mismatch: {} vs {}",
                    contact.id, contact_id
                ));
            }

            if contact.name.trim().is_empty() {
                issues.push(format!("Contact {} has empty name", contact_id));
            }

            if contact.address.trim().is_empty() {
                issues.push(format!("Contact {} has empty address", contact_id));
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
            "html" => {
                let mut html_data = String::from("<html><body><h1>Chat Export</h1><div>");
                for message in messages {
                    let time = chrono::DateTime::from_timestamp(message.timestamp as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "Unknown time".to_string());

                    html_data.push_str(&format!(
                        "<p><strong>[{}] {}:</strong> {}</p>",
                        time, message.from, message.content
                    ));
                }
                html_data.push_str("</div></body></html>");
                Ok(html_data)
            }
            _ => Err(StorageError::SerializationError(
                "Unsupported format".to_string(),
            )),
        }
    }

    pub async fn import_chat_data(
        &self,
        chat_id: &str,
        data: &str,
        format: &str,
    ) -> Result<u32, StorageError> {
        let imported_messages: Vec<ChatMessage> = match format.to_lowercase().as_str() {
            "json" => serde_json::from_str(data)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?,
            _ => {
                return Err(StorageError::SerializationError(
                    "Unsupported import format".to_string(),
                ))
            }
        };

        let import_count = imported_messages.len() as u32;

        for message in imported_messages {
            self.save_message(chat_id, &message).await?;
        }

        Ok(import_count)
    }

    pub async fn get_storage_health(&self) -> Result<StorageHealth, StorageError> {
        let issues = self.validate_all_data().await?;
        let stats = self.stats.read().await;
        let total_size = stats.total_size_bytes;
        drop(stats);
        let fragmentation = self.calculate_fragmentation().await?;

        Ok(StorageHealth {
            is_healthy: issues.is_empty(),
            total_size_bytes: total_size,
            fragmentation_percent: fragmentation,
            corruption_issues: issues.len() as u32,
            last_check: chrono::Utc::now(),
            recommendations: self.get_health_recommendations(fragmentation, &issues),
        })
    }

    async fn validate_all_data(&self) -> Result<Vec<String>, StorageError> {
        let mut issues = Vec::new();

        issues.extend(self.validate_contacts().await?);
        issues.extend(self.validate_chats().await?);

        Ok(issues)
    }

    async fn calculate_fragmentation(&self) -> Result<f64, StorageError> {
        let mut total_messages = 0;
        let mut fragmented_chats = 0;
        let chat_storage = self.chat_storage.read().await;

        for (_, messages) in &chat_storage.messages {
            total_messages += messages.len();

            let mut timestamps: Vec<u64> = messages.iter().map(|m| m.timestamp).collect();
            timestamps.sort();

            let mut gaps = 0;
            for window in timestamps.windows(2) {
                if window[1] - window[0] > 3600 {
                    gaps += 1;
                }
            }

            if gaps > messages.len() / 10 {
                fragmented_chats += 1;
            }
        }

        if chat_storage.messages.len() > 0 {
            Ok((fragmented_chats as f64 / chat_storage.messages.len() as f64) * 100.0)
        } else {
            Ok(0.0)
        }
    }

    fn get_health_recommendations(&self, fragmentation: f64, issues: &[String]) -> Vec<String> {
        let mut recommendations = Vec::new();

        if fragmentation > 30.0 {
            recommendations
                .push("Consider running storage optimization to reduce fragmentation".to_string());
        }

        if !issues.is_empty() {
            recommendations.push("Fix data corruption issues found during validation".to_string());
        }

        let stats = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.stats.read())
        });

        if stats.total_size_bytes > 100 * 1024 * 1024 {
            recommendations
                .push("Consider archiving old messages to reduce storage size".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Storage is healthy, no actions needed".to_string());
        }

        recommendations
    }
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
