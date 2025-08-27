use crate::network::{ChatMessage, ChatMessageType, DeliveryStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chat {
    pub id: String,
    pub name: String,
    pub participants: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub last_message_at: Option<DateTime<Utc>>,
    pub message_count: u64,
    pub is_group: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatInfo {
    pub chat_id: String,
    pub name: String,
    pub participant_count: usize,
    pub message_count: u64,
    pub last_activity: Option<DateTime<Utc>>,
    pub unread_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageFilter {
    pub chat_id: Option<String>,
    pub sender: Option<String>,
    pub message_type: Option<ChatMessageType>,
    pub delivery_status: Option<DeliveryStatus>,
    pub from_timestamp: Option<u64>,
    pub to_timestamp: Option<u64>,
    pub content_search: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatStatistics {
    pub total_chats: usize,
    pub total_messages: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub failed_messages: u64,
    pub average_response_time: f64,
    pub most_active_chat: Option<String>,
    pub daily_message_count: HashMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageBatch {
    pub messages: Vec<ChatMessage>,
    pub total_count: u64,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatSearchResult {
    pub chat_id: String,
    pub messages: Vec<ChatMessage>,
    pub total_matches: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DraftMessage {
    pub chat_id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatExportOptions {
    pub format: ExportFormat,
    pub include_metadata: bool,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub participants_filter: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Txt,
    Html,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatBackup {
    pub backup_id: String,
    pub created_at: DateTime<Utc>,
    pub chat_count: usize,
    pub message_count: u64,
    pub file_path: String,
    pub file_size: u64,
}

impl Chat {
    pub fn new(name: String, is_group: bool) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            participants: Vec::new(),
            created_at: Utc::now(),
            last_message_at: None,
            message_count: 0,
            is_group,
        }
    }

    pub fn add_participant(&mut self, participant: String) {
        if !self.participants.contains(&participant) {
            self.participants.push(participant);
        }
    }

    pub fn remove_participant(&mut self, participant: &str) {
        self.participants.retain(|p| p != participant);
    }

    pub fn update_last_activity(&mut self) {
        self.last_message_at = Some(Utc::now());
    }

    pub fn increment_message_count(&mut self) {
        self.message_count += 1;
        self.update_last_activity();
    }
}

impl Default for MessageFilter {
    fn default() -> Self {
        Self {
            chat_id: None,
            sender: None,
            message_type: None,
            delivery_status: None,
            from_timestamp: None,
            to_timestamp: None,
            content_search: None,
        }
    }
}

impl ChatStatistics {
    pub fn new() -> Self {
        Self {
            total_chats: 0,
            total_messages: 0,
            messages_sent: 0,
            messages_received: 0,
            failed_messages: 0,
            average_response_time: 0.0,
            most_active_chat: None,
            daily_message_count: HashMap::new(),
        }
    }

    pub fn get_success_rate(&self) -> f64 {
        if self.total_messages == 0 {
            return 0.0;
        }
        let successful = self.total_messages - self.failed_messages;
        successful as f64 / self.total_messages as f64
    }
}