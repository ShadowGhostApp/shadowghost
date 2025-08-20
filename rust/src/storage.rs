use crate::network::{ChatMessage, Contact};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::path::Path;

#[derive(Debug)]
pub enum StorageError {
    IoError(String),
    SerializationError(String),
    DatabaseError(String),
    NotFound(String),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::IoError(msg) => write!(f, "IO error: {}", msg),
            StorageError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            StorageError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            StorageError::NotFound(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

impl Error for StorageError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_contacts: u64,
    pub total_messages: u64,
    pub storage_size_bytes: u64,
    pub last_backup: Option<DateTime<Utc>>,
}

impl Default for StorageStats {
    fn default() -> Self {
        Self {
            total_contacts: 0,
            total_messages: 0,
            storage_size_bytes: 0,
            last_backup: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatStorage {
    pub messages: HashMap<String, Vec<ChatMessage>>,
    pub last_message_id: u64,
}

impl Default for ChatStorage {
    fn default() -> Self {
        Self {
            messages: HashMap::new(),
            last_message_id: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactStorage {
    pub contacts: HashMap<String, Contact>,
    pub last_contact_id: u64,
}

impl Default for ContactStorage {
    fn default() -> Self {
        Self {
            contacts: HashMap::new(),
            last_contact_id: 0,
        }
    }
}

pub struct StorageManager {
    base_path: String,
    chat_storage: ChatStorage,
    contact_storage: ContactStorage,
    stats: StorageStats,
}

impl StorageManager {
    pub fn new(base_path: String) -> Result<Self, StorageError> {
        let path = Path::new(&base_path);
        if !path.exists() {
            std::fs::create_dir_all(path).map_err(|e| StorageError::IoError(e.to_string()))?;
        }

        Ok(Self {
            base_path,
            chat_storage: ChatStorage::default(),
            contact_storage: ContactStorage::default(),
            stats: StorageStats::default(),
        })
    }

    pub async fn initialize(&mut self) -> Result<(), StorageError> {
        self.load_contacts().await?;
        self.load_chats().await?;
        self.update_stats().await?;
        Ok(())
    }

    pub async fn save_contact(&mut self, contact: &Contact) -> Result<(), StorageError> {
        self.contact_storage
            .contacts
            .insert(contact.id.clone(), contact.clone());
        self.save_contacts().await?;
        self.update_stats().await?;
        Ok(())
    }

    pub async fn get_contact(&self, contact_id: &str) -> Result<Option<Contact>, StorageError> {
        Ok(self.contact_storage.contacts.get(contact_id).cloned())
    }

    pub async fn get_all_contacts(&self) -> Result<Vec<Contact>, StorageError> {
        Ok(self.contact_storage.contacts.values().cloned().collect())
    }

    pub async fn delete_contact(&mut self, contact_id: &str) -> Result<(), StorageError> {
        self.contact_storage.contacts.remove(contact_id);
        self.save_contacts().await?;
        self.update_stats().await?;
        Ok(())
    }

    pub async fn save_message(
        &mut self,
        chat_id: &str,
        message: &ChatMessage,
    ) -> Result<(), StorageError> {
        self.chat_storage
            .messages
            .entry(chat_id.to_string())
            .or_insert_with(Vec::new)
            .push(message.clone());

        self.save_chats().await?;
        self.update_stats().await?;
        Ok(())
    }

    pub async fn get_messages(&self, chat_id: &str) -> Result<Vec<ChatMessage>, StorageError> {
        Ok(self
            .chat_storage
            .messages
            .get(chat_id)
            .cloned()
            .unwrap_or_default())
    }

    pub async fn get_all_chats(&self) -> Result<HashMap<String, Vec<ChatMessage>>, StorageError> {
        Ok(self.chat_storage.messages.clone())
    }

    pub async fn delete_chat(&mut self, chat_id: &str) -> Result<(), StorageError> {
        self.chat_storage.messages.remove(chat_id);
        self.save_chats().await?;
        self.update_stats().await?;
        Ok(())
    }

    pub async fn get_stats(&self) -> Result<StorageStats, StorageError> {
        Ok(self.stats.clone())
    }

    pub async fn backup(&mut self) -> Result<String, StorageError> {
        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_path = format!("{}/backup_{}.json", self.base_path, timestamp);

        let backup_data = BackupData {
            contacts: self.contact_storage.clone(),
            chats: self.chat_storage.clone(),
            stats: self.stats.clone(),
            created_at: Utc::now(),
        };

        let json_data = serde_json::to_string_pretty(&backup_data)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        tokio::fs::write(&backup_path, json_data)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        self.stats.last_backup = Some(Utc::now());
        Ok(backup_path)
    }

    pub async fn restore_from_backup(&mut self, backup_path: &str) -> Result<(), StorageError> {
        let data = tokio::fs::read_to_string(backup_path)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;

        let backup_data: BackupData = serde_json::from_str(&data)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        self.contact_storage = backup_data.contacts;
        self.chat_storage = backup_data.chats;
        self.stats = backup_data.stats;

        self.save_contacts().await?;
        self.save_chats().await?;
        Ok(())
    }

    async fn load_contacts(&mut self) -> Result<(), StorageError> {
        let contacts_path = format!("{}/contacts.json", self.base_path);
        match tokio::fs::read_to_string(&contacts_path).await {
            Ok(data) => {
                self.contact_storage = serde_json::from_str(&data)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
            }
            Err(_) => {
                self.contact_storage = ContactStorage::default();
            }
        }
        Ok(())
    }

    async fn save_contacts(&self) -> Result<(), StorageError> {
        let contacts_path = format!("{}/contacts.json", self.base_path);
        let data = serde_json::to_string_pretty(&self.contact_storage)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        tokio::fs::write(&contacts_path, data)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        Ok(())
    }

    async fn load_chats(&mut self) -> Result<(), StorageError> {
        let chats_path = format!("{}/chats.json", self.base_path);
        match tokio::fs::read_to_string(&chats_path).await {
            Ok(data) => {
                self.chat_storage = serde_json::from_str(&data)
                    .map_err(|e| StorageError::SerializationError(e.to_string()))?;
            }
            Err(_) => {
                self.chat_storage = ChatStorage::default();
            }
        }
        Ok(())
    }

    async fn save_chats(&self) -> Result<(), StorageError> {
        let chats_path = format!("{}/chats.json", self.base_path);
        let data = serde_json::to_string_pretty(&self.chat_storage)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;

        tokio::fs::write(&chats_path, data)
            .await
            .map_err(|e| StorageError::IoError(e.to_string()))?;
        Ok(())
    }

    async fn update_stats(&mut self) -> Result<(), StorageError> {
        self.stats.total_contacts = self.contact_storage.contacts.len() as u64;
        self.stats.total_messages = self
            .chat_storage
            .messages
            .values()
            .map(|messages| messages.len() as u64)
            .sum();

        let contacts_size = self.estimate_size(&self.contact_storage)?;
        let chats_size = self.estimate_size(&self.chat_storage)?;
        self.stats.storage_size_bytes = contacts_size + chats_size;

        Ok(())
    }

    fn estimate_size<T: Serialize>(&self, data: &T) -> Result<u64, StorageError> {
        let json = serde_json::to_string(data)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        Ok(json.len() as u64)
    }

    pub async fn clear_all_data(&mut self) -> Result<(), StorageError> {
        self.contact_storage = ContactStorage::default();
        self.chat_storage = ChatStorage::default();
        self.stats = StorageStats::default();

        self.save_contacts().await?;
        self.save_chats().await?;
        Ok(())
    }

    pub async fn get_unread_message_count(&self, chat_id: &str) -> Result<u64, StorageError> {
        if let Some(messages) = self.chat_storage.messages.get(chat_id) {
            let unread_count = messages
                .iter()
                .filter(|msg| {
                    matches!(
                        msg.delivery_status,
                        crate::network::DeliveryStatus::Delivered
                    )
                })
                .count() as u64;
            Ok(unread_count)
        } else {
            Ok(0)
        }
    }

    pub async fn mark_messages_as_read(&mut self, chat_id: &str) -> Result<(), StorageError> {
        if let Some(messages) = self.chat_storage.messages.get_mut(chat_id) {
            for message in messages.iter_mut() {
                if matches!(
                    message.delivery_status,
                    crate::network::DeliveryStatus::Delivered
                ) {
                    message.delivery_status = crate::network::DeliveryStatus::Read;
                }
            }
            self.save_chats().await?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackupData {
    contacts: ContactStorage,
    chats: ChatStorage,
    stats: StorageStats,
    created_at: DateTime<Utc>,
}
