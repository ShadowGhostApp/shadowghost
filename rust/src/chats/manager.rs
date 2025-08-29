use crate::chats::{
    Chat, ChatExportOptions, ChatInfo, ChatSearchResult, ChatStatistics, DraftMessage,
    MessageFilter,
};
use crate::events::{AppEvent, EventBus, StorageEvent};
use crate::network::{ChatMessage, ChatMessageType, DeliveryStatus};
use crate::storage::StorageManager;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug)]
pub enum ChatError {
    ChatNotFound(String),
    MessageNotFound(String),
    InvalidInput(String),
    StorageError(String),
    SerializationError(String),
}

impl std::fmt::Display for ChatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChatError::ChatNotFound(msg) => write!(f, "Chat not found: {}", msg),
            ChatError::MessageNotFound(msg) => write!(f, "Message not found: {}", msg),
            ChatError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            ChatError::StorageError(msg) => write!(f, "Storage error: {}", msg),
            ChatError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for ChatError {}

pub struct Manager {
    chats: Arc<RwLock<HashMap<String, Chat>>>,
    draft_messages: Arc<RwLock<HashMap<String, DraftMessage>>>,
    storage: Arc<RwLock<StorageManager>>,
    event_bus: EventBus,
    statistics: Arc<RwLock<ChatStatistics>>,
}

impl Manager {
    pub fn new(storage: Arc<RwLock<StorageManager>>, event_bus: EventBus) -> Result<Self, String> {
        Ok(Self {
            chats: Arc::new(RwLock::new(HashMap::new())),
            draft_messages: Arc::new(RwLock::new(HashMap::new())),
            storage,
            event_bus,
            statistics: Arc::new(RwLock::new(ChatStatistics::new())),
        })
    }

    pub async fn initialize(&mut self) -> Result<(), ChatError> {
        // Load chats from storage
        self.load_chats().await?;
        self.update_statistics().await?;

        self.event_bus
            .emit(AppEvent::Storage(StorageEvent::ChatHistoryLoaded {
                chat_id: "all".to_string(),
                message_count: self.get_total_message_count().await,
            }));

        Ok(())
    }

    pub async fn create_chat(&self, name: String, is_group: bool) -> Result<Chat, ChatError> {
        let chat = Chat::new(name, is_group);

        let mut chats = self.chats.write().await;
        chats.insert(chat.id.clone(), chat.clone());

        self.save_chats().await?;
        self.update_statistics().await?;

        Ok(chat)
    }

    pub async fn get_chat(&self, chat_id: &str) -> Result<Chat, ChatError> {
        let chats = self.chats.read().await;
        chats
            .get(chat_id)
            .cloned()
            .ok_or_else(|| ChatError::ChatNotFound(chat_id.to_string()))
    }

    pub async fn get_all_chats(&self) -> Vec<Chat> {
        let chats = self.chats.read().await;
        chats.values().cloned().collect()
    }

    pub async fn send_message(
        &self,
        chat_id: &str,
        sender: &str,
        recipient: &str,
        content: &str,
        message_type: ChatMessageType,
    ) -> Result<String, ChatError> {
        // Check if chat exists
        let mut chats = self.chats.write().await;
        let chat = chats
            .get_mut(chat_id)
            .ok_or_else(|| ChatError::ChatNotFound(chat_id.to_string()))?;

        let message = ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            from: sender.to_string(),
            to: recipient.to_string(),
            content: content.to_string(),
            msg_type: message_type,
            timestamp: chrono::Utc::now().timestamp() as u64,
            delivery_status: DeliveryStatus::Pending,
        };

        // Save message to storage
        let storage = self.storage.read().await;
        storage
            .save_message(chat_id, &message)
            .await
            .map_err(|e| ChatError::StorageError(e.to_string()))?;

        // Update chat
        chat.increment_message_count();

        drop(chats);

        self.save_chats().await?;
        self.update_statistics().await?;

        Ok(message.id)
    }

    pub async fn get_messages(
        &self,
        chat_id: &str,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<ChatMessage>, ChatError> {
        let storage = self.storage.read().await;
        let messages = storage
            .get_messages(chat_id)
            .await
            .map_err(|e| ChatError::StorageError(e.to_string()))?;

        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(messages.len());

        let end = std::cmp::min(offset + limit, messages.len());

        Ok(messages[offset..end].to_vec())
    }

    pub async fn search_messages(
        &self,
        filter: MessageFilter,
    ) -> Result<Vec<ChatSearchResult>, ChatError> {
        let storage = self.storage.read().await;
        let mut results = Vec::new();

        if let Some(chat_id) = &filter.chat_id {
            let messages = storage
                .get_messages(chat_id)
                .await
                .map_err(|e| ChatError::StorageError(e.to_string()))?;

            let filtered_messages = self.apply_filter(&messages, &filter);

            if !filtered_messages.is_empty() {
                results.push(ChatSearchResult {
                    chat_id: chat_id.clone(),
                    messages: filtered_messages.clone(),
                    total_matches: filtered_messages.len(),
                });
            }
        } else {
            // Search across all chats
            let chats = self.chats.read().await;
            for chat_id in chats.keys() {
                let messages = storage
                    .get_messages(chat_id)
                    .await
                    .map_err(|e| ChatError::StorageError(e.to_string()))?;

                let filtered_messages = self.apply_filter(&messages, &filter);

                if !filtered_messages.is_empty() {
                    results.push(ChatSearchResult {
                        chat_id: chat_id.clone(),
                        messages: filtered_messages.clone(),
                        total_matches: filtered_messages.len(),
                    });
                }
            }
        }

        Ok(results)
    }

    pub async fn get_chat_info(&self, chat_id: &str) -> Result<ChatInfo, ChatError> {
        let chat = self.get_chat(chat_id).await?;

        let storage = self.storage.read().await;
        let messages = storage
            .get_messages(chat_id)
            .await
            .map_err(|e| ChatError::StorageError(e.to_string()))?;

        let unread_count = messages
            .iter()
            .filter(|m| matches!(m.delivery_status, DeliveryStatus::Delivered))
            .count() as u64;

        Ok(ChatInfo {
            chat_id: chat.id,
            name: chat.name,
            participant_count: chat.participants.len(),
            message_count: chat.message_count,
            last_activity: chat.last_message_at,
            unread_count,
        })
    }

    pub async fn delete_chat(&self, chat_id: &str) -> Result<(), ChatError> {
        let mut chats = self.chats.write().await;
        chats
            .remove(chat_id)
            .ok_or_else(|| ChatError::ChatNotFound(chat_id.to_string()))?;

        // Also delete from storage
        let storage = self.storage.write().await;
        storage
            .delete_chat(chat_id)
            .await
            .map_err(|e| ChatError::StorageError(e.to_string()))?;

        drop(chats);
        drop(storage);

        self.save_chats().await?;
        self.update_statistics().await?;

        Ok(())
    }

    pub async fn update_message_status(
        &self,
        message_id: &str,
        new_status: DeliveryStatus,
    ) -> Result<(), ChatError> {
        let storage = self.storage.write().await;
        storage
            .update_message_status(message_id, new_status)
            .await
            .map_err(|e| ChatError::StorageError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_statistics(&self) -> ChatStatistics {
        let stats = self.statistics.read().await;
        stats.clone()
    }

    pub async fn export_chat(
        &self,
        chat_id: &str,
        options: ChatExportOptions,
    ) -> Result<String, ChatError> {
        let storage = self.storage.read().await;
        let format_str = match options.format {
            crate::chats::ExportFormat::Json => "json",
            crate::chats::ExportFormat::Csv => "csv",
            crate::chats::ExportFormat::Txt => "txt",
            crate::chats::ExportFormat::Html => "html",
        };

        storage
            .export_chat_data(chat_id, format_str)
            .await
            .map_err(|e| ChatError::StorageError(e.to_string()))
    }

    // Private helper methods

    async fn load_chats(&self) -> Result<(), ChatError> {
        // This would load chats from storage
        // For now, we'll create a simple implementation
        Ok(())
    }

    async fn save_chats(&self) -> Result<(), ChatError> {
        // This would save chats to storage
        // For now, we'll create a simple implementation
        Ok(())
    }

    async fn update_statistics(&self) -> Result<(), ChatError> {
        let chats = self.chats.read().await;
        let mut stats = self.statistics.write().await;

        stats.total_chats = chats.len();
        stats.total_messages = chats.values().map(|c| c.message_count).sum();

        // Find most active chat
        if let Some(most_active) = chats.values().max_by_key(|c| c.message_count) {
            stats.most_active_chat = Some(most_active.id.clone());
        }

        Ok(())
    }

    async fn get_total_message_count(&self) -> usize {
        let chats = self.chats.read().await;
        chats.values().map(|c| c.message_count as usize).sum()
    }

    fn apply_filter(&self, messages: &[ChatMessage], filter: &MessageFilter) -> Vec<ChatMessage> {
        messages
            .iter()
            .filter(|msg| {
                if let Some(sender) = &filter.sender {
                    if msg.from != *sender {
                        return false;
                    }
                }

                if let Some(msg_type) = &filter.message_type {
                    if msg.msg_type != *msg_type {
                        return false;
                    }
                }

                if let Some(status) = &filter.delivery_status {
                    if msg.delivery_status != *status {
                        return false;
                    }
                }

                if let Some(from_ts) = filter.from_timestamp {
                    if msg.timestamp < from_ts {
                        return false;
                    }
                }

                if let Some(to_ts) = filter.to_timestamp {
                    if msg.timestamp > to_ts {
                        return false;
                    }
                }

                if let Some(search_term) = &filter.content_search {
                    if !msg
                        .content
                        .to_lowercase()
                        .contains(&search_term.to_lowercase())
                    {
                        return false;
                    }
                }

                true
            })
            .cloned()
            .collect()
    }
}
