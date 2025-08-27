use crate::network::ChatMessage;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_size_bytes: u64,
    pub message_count: u64,
    pub chat_count: u32,
    pub contact_count: u32,
    pub last_backup: Option<DateTime<Utc>>,
    pub storage_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatStorage {
    pub messages: HashMap<String, Vec<ChatMessage>>,
    pub metadata: HashMap<String, ChatMetadata>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMetadata {
    pub chat_id: String,
    pub name: String,
    pub participants: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_message_at: Option<DateTime<Utc>>,
    pub message_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupInfo {
    pub backup_id: String,
    pub created_at: DateTime<Utc>,
    pub file_path: String,
    pub file_size: u64,
    pub backup_type: BackupType,
    pub status: BackupStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupType {
    Full,
    Incremental,
    ChatOnly,
    ContactsOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupStatus {
    InProgress,
    Completed,
    Failed,
    Corrupted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub data_path: String,
    pub max_file_size: u64,
    pub compression_enabled: bool,
    pub encryption_enabled: bool,
    pub backup_interval_hours: u32,
    pub max_backups: u32,
}

#[derive(Debug)]
pub enum StorageError {
    FileNotFound(String),
    PermissionDenied(String),
    SerializationError(String),
    CompressionError(String),
    EncryptionError(String),
    CorruptedData(String),
    InsufficientSpace(String),
    NotFound(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::FileNotFound(msg) => write!(f, "File not found: {}", msg),
            StorageError::PermissionDenied(msg) => write!(f, "Permission denied: {}", msg),
            StorageError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            StorageError::CompressionError(msg) => write!(f, "Compression error: {}", msg),
            StorageError::EncryptionError(msg) => write!(f, "Encryption error: {}", msg),
            StorageError::CorruptedData(msg) => write!(f, "Corrupted data: {}", msg),
            StorageError::InsufficientSpace(msg) => write!(f, "Insufficient space: {}", msg),
            StorageError::NotFound(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

impl std::error::Error for StorageError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageOptimizationResult {
    pub original_size_bytes: u64,
    pub optimized_size_bytes: u64,
    pub space_saved_bytes: u64,
    pub messages_deduplicated: u32,
    pub optimization_time: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageHealth {
    pub is_healthy: bool,
    pub total_size_bytes: u64,
    pub fragmentation_percent: f64,
    pub corruption_issues: u32,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub recommendations: Vec<String>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_path: "./data".to_string(),
            max_file_size: 100 * 1024 * 1024, // 100MB
            compression_enabled: true,
            encryption_enabled: true,
            backup_interval_hours: 24,
            max_backups: 7,
        }
    }
}

impl ChatStorage {
    pub fn new() -> Self {
        Self {
            messages: HashMap::new(),
            metadata: HashMap::new(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }

    pub fn add_message(&mut self, chat_id: &str, message: ChatMessage) {
        let messages = self
            .messages
            .entry(chat_id.to_string())
            .or_insert_with(Vec::new);
        messages.push(message);
        self.updated_at = Utc::now();

        // Update metadata
        let metadata = self
            .metadata
            .entry(chat_id.to_string())
            .or_insert_with(|| ChatMetadata {
                chat_id: chat_id.to_string(),
                name: format!("Chat {}", chat_id),
                participants: Vec::new(),
                created_at: Utc::now(),
                last_message_at: None,
                message_count: 0,
            });

        metadata.message_count += 1;
        metadata.last_message_at = Some(Utc::now());
    }

    pub fn get_messages(&self, chat_id: &str) -> Vec<ChatMessage> {
        self.messages.get(chat_id).cloned().unwrap_or_default()
    }

    pub fn get_message_count(&self, chat_id: &str) -> u64 {
        self.messages
            .get(chat_id)
            .map(|msgs| msgs.len() as u64)
            .unwrap_or(0)
    }

    pub fn get_total_message_count(&self) -> u64 {
        self.messages.values().map(|msgs| msgs.len() as u64).sum()
    }

    pub fn get_chat_ids(&self) -> Vec<String> {
        self.messages.keys().cloned().collect()
    }
}

impl StorageStats {
    pub fn new() -> Self {
        Self {
            total_size_bytes: 0,
            message_count: 0,
            chat_count: 0,
            contact_count: 0,
            last_backup: None,
            storage_version: "1.0.0".to_string(),
        }
    }

    pub fn update_size(&mut self, size: u64) {
        self.total_size_bytes = size;
    }

    pub fn update_message_count(&mut self, count: u64) {
        self.message_count = count;
    }

    pub fn update_chat_count(&mut self, count: u32) {
        self.chat_count = count;
    }

    pub fn update_contact_count(&mut self, count: u32) {
        self.contact_count = count;
    }

    pub fn set_last_backup(&mut self, timestamp: DateTime<Utc>) {
        self.last_backup = Some(timestamp);
    }
}