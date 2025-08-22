use super::core_api::CORE;
use crate::frb_generated::StreamSink;
use crate::network::manager::{ChatMessage, ChatMessageType, DeliveryStatus};
use chrono::Utc;
use flutter_rust_bridge::frb;

#[frb]
pub async fn send_text_message(contact_id: String, content: String) -> Result<String, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        // Basic validation
        if content.trim().is_empty() {
            return Err("Message content cannot be empty".to_string());
        }

        if content.len() > 10000 {
            return Err("Message content too long (max 10000 characters)".to_string());
        }

        let message = ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            from: "local_user".to_string(),
            to: contact_id.clone(),
            content: content.trim().to_string(),
            timestamp: Utc::now().timestamp() as u64,
            msg_type: ChatMessageType::Text,
            delivery_status: DeliveryStatus::Pending,
        };

        match core
            .lock()
            .await
            .send_chat_message(&contact_id, &message.content)
            .await
        {
            Ok(_) => Ok(message.id),
            Err(e) => Err(format!("Failed to send message: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn send_file_message(
    contact_id: String,
    file_path: String,
    file_name: String,
) -> Result<String, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        // Basic validation
        if file_name.trim().is_empty() {
            return Err("File name cannot be empty".to_string());
        }

        if file_path.trim().is_empty() {
            return Err("File path cannot be empty".to_string());
        }

        // Check if file exists
        if !std::path::Path::new(&file_path).exists() {
            return Err("File does not exist".to_string());
        }

        let message = ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            from: "local_user".to_string(),
            to: contact_id.clone(),
            content: format!("{}|{}", file_name.trim(), file_path),
            timestamp: Utc::now().timestamp() as u64,
            msg_type: ChatMessageType::File,
            delivery_status: DeliveryStatus::Pending,
        };

        match core
            .lock()
            .await
            .send_chat_message(&contact_id, &message.content)
            .await
        {
            Ok(_) => Ok(message.id),
            Err(e) => Err(format!("Failed to send file: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn get_messages(
    contact_id: String,
    limit: u32,
    offset: u32,
) -> Result<Vec<ChatMessage>, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        match core.lock().await.get_chat_messages(&contact_id).await {
            Ok(mut messages) => {
                // Sort by time (newest first)
                messages.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

                // Apply offset and limit
                let start = offset as usize;
                let end = std::cmp::min(start + limit as usize, messages.len());

                if start < messages.len() {
                    Ok(messages[start..end].to_vec())
                } else {
                    Ok(vec![])
                }
            }
            Err(e) => Err(format!("Failed to get messages: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn get_recent_messages(
    contact_id: String,
    limit: u32,
) -> Result<Vec<ChatMessage>, String> {
    get_messages(contact_id, limit, 0).await
}

#[frb]
pub async fn get_unread_message_count(contact_id: String) -> Result<u32, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        match core.lock().await.get_unread_count(&contact_id) {
            Ok(count) => Ok(count as u32),
            Err(e) => Err(format!("Failed to get unread count: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn search_messages(
    contact_id: String,
    query: String,
    limit: u32,
) -> Result<Vec<ChatMessage>, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        match core.lock().await.get_chat_messages(&contact_id).await {
            Ok(messages) => {
                let query_lower = query.to_lowercase();
                let filtered: Vec<ChatMessage> = messages
                    .into_iter()
                    .filter(|msg| msg.content.to_lowercase().contains(&query_lower))
                    .take(limit as usize)
                    .collect();

                Ok(filtered)
            }
            Err(e) => Err(format!("Failed to search messages: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn mark_messages_as_read(contact_id: String) -> Result<String, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        // TODO: Implement mark as read functionality in core
        Ok(format!("Messages with {} marked as read", contact_id))
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn delete_message(message_id: String) -> Result<String, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        // TODO: Implement message deletion in core
        Ok(format!("Message {} deleted", message_id))
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn get_chat_statistics(contact_id: String) -> Result<ChatStatistics, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        match core.lock().await.get_chat_messages(&contact_id).await {
            Ok(messages) => {
                let total_messages = messages.len();
                let sent_messages = messages.iter().filter(|m| m.from == "local_user").count();
                let received_messages = total_messages - sent_messages;
                let unread_count = messages
                    .iter()
                    .filter(|m| matches!(m.delivery_status, DeliveryStatus::Delivered))
                    .count();

                let first_message_time = messages
                    .iter()
                    .min_by_key(|m| m.timestamp)
                    .map(|m| m.timestamp);

                let last_message_time = messages
                    .iter()
                    .max_by_key(|m| m.timestamp)
                    .map(|m| m.timestamp);

                Ok(ChatStatistics {
                    total_messages: total_messages as u32,
                    sent_messages: sent_messages as u32,
                    received_messages: received_messages as u32,
                    unread_messages: unread_count as u32,
                    first_message_timestamp: first_message_time,
                    last_message_timestamp: last_message_time,
                    file_messages: messages
                        .iter()
                        .filter(|m| matches!(m.msg_type, ChatMessageType::File))
                        .count() as u32,
                    media_messages: messages
                        .iter()
                        .filter(|m| {
                            matches!(m.msg_type, ChatMessageType::Image | ChatMessageType::Voice)
                        })
                        .count() as u32,
                })
            }
            Err(e) => Err(format!("Failed to get chat statistics: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn export_chat_history(contact_id: String, format: String) -> Result<String, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        match core.lock().await.get_chat_messages(&contact_id).await {
            Ok(messages) => match format.to_lowercase().as_str() {
                "json" => match serde_json::to_string_pretty(&messages) {
                    Ok(json) => Ok(json),
                    Err(e) => Err(format!("Failed to serialize to JSON: {}", e)),
                },
                "txt" => {
                    let mut text = String::new();
                    text.push_str(&format!("Chat history with {}\n", contact_id));
                    text.push_str("=".repeat(50).as_str());
                    text.push('\n');

                    for message in messages.iter() {
                        let time = chrono::DateTime::from_timestamp(message.timestamp as i64, 0)
                            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                            .unwrap_or_else(|| "Unknown time".to_string());

                        text.push_str(&format!(
                            "[{}] {}: {}\n",
                            time, message.from, message.content
                        ));
                    }

                    Ok(text)
                }
                _ => Err("Unsupported format. Use 'json' or 'txt'".to_string()),
            },
            Err(e) => Err(format!("Failed to get chat history: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn listen_to_message_updates(
    contact_id: String,
    sink: StreamSink<MessageUpdate>,
) -> Result<(), String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.clone() {
        drop(core_guard);

        tokio::spawn(async move {
            let mut event_receiver = core.lock().await.get_event_bus().subscribe();

            while let Ok(event) = event_receiver.recv().await {
                if let crate::events::AppEvent::Network(
                    crate::events::NetworkEvent::MessageReceived { message },
                ) = &event
                {
                    if message.to == contact_id || message.from == contact_id {
                        let update = MessageUpdate {
                            message: message.clone(),
                            update_type: MessageUpdateType::Received,
                            timestamp: chrono::Utc::now().timestamp() as u64,
                        };
                        let _ = sink.add(update);
                    }
                }
            }
        });

        Ok(())
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
#[derive(Debug, Clone)]
pub struct ChatStatistics {
    pub total_messages: u32,
    pub sent_messages: u32,
    pub received_messages: u32,
    pub unread_messages: u32,
    pub first_message_timestamp: Option<u64>,
    pub last_message_timestamp: Option<u64>,
    pub file_messages: u32,
    pub media_messages: u32,
}

#[frb]
#[derive(Debug, Clone)]
pub struct MessageUpdate {
    pub message: ChatMessage,
    pub update_type: MessageUpdateType,
    pub timestamp: u64,
}

#[frb]
#[derive(Debug, Clone)]
pub enum MessageUpdateType {
    Received,
    Sent,
    Deleted,
    Updated,
}
