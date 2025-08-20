use super::core::CORE;
use crate::prelude::*;
use flutter_rust_bridge::frb;

pub async fn send_text_message(contact_id: String, content: String) -> Result<String, String> {
    let core_guard = CORE.lock().unwrap();
    if let Some(core) = core_guard.clone() {
        drop(core_guard);

        let message = ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            from: core.get_public_key().to_string(),
            to: contact_id,
            content,
            timestamp: chrono::Utc::now().timestamp() as u64,
            msg_type: ChatMessageType::Text,
            delivery_status: DeliveryStatus::Pending,
        };

        match core.send_message(message).await {
            Ok(_) => Ok("Message sent successfully".to_string()),
            Err(e) => Err(format!("Failed to send message: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

pub async fn send_file_message(
    contact_id: String,
    file_path: String,
    file_name: String,
) -> Result<String, String> {
    let core_guard = CORE.lock().unwrap();
    if let Some(core) = core_guard.clone() {
        drop(core_guard);

        let message = ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            from: core.get_public_key().to_string(),
            to: contact_id,
            content: format!("{}|{}", file_name, file_path),
            timestamp: chrono::Utc::now().timestamp() as u64,
            msg_type: ChatMessageType::File,
            delivery_status: DeliveryStatus::Pending,
        };

        match core.send_message(message).await {
            Ok(_) => Ok("File message sent successfully".to_string()),
            Err(e) => Err(format!("Failed to send file: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

pub async fn get_messages(
    contact_id: String,
    limit: u32,
    offset: u32,
) -> Result<Vec<ChatMessage>, String> {
    let core_guard = CORE.lock().unwrap();
    if let Some(core) = core_guard.clone() {
        drop(core_guard);
        match core
            .get_messages_paginated(&contact_id, limit as usize, offset as usize)
            .await
        {
            Ok(messages) => Ok(messages),
            Err(e) => Err(format!("Failed to get messages: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

pub async fn get_recent_messages(
    contact_id: String,
    limit: u32,
) -> Result<Vec<ChatMessage>, String> {
    let core_guard = CORE.lock().unwrap();
    if let Some(core) = core_guard.clone() {
        drop(core_guard);
        match core.get_messages(&contact_id, limit as usize).await {
            Ok(messages) => Ok(messages),
            Err(e) => Err(format!("Failed to get recent messages: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

pub async fn mark_message_as_read(message_id: String) -> Result<String, String> {
    let core_guard = CORE.lock().unwrap();
    if let Some(core) = core_guard.clone() {
        drop(core_guard);
        match core.mark_message_read(&message_id).await {
            Ok(_) => Ok("Message marked as read".to_string()),
            Err(e) => Err(format!("Failed to mark message as read: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

pub async fn delete_message(message_id: String) -> Result<String, String> {
    let core_guard = CORE.lock().unwrap();
    if let Some(core) = core_guard.clone() {
        drop(core_guard);
        match core.delete_message(&message_id).await {
            Ok(_) => Ok("Message deleted successfully".to_string()),
            Err(e) => Err(format!("Failed to delete message: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb(sync)]
pub fn get_unread_message_count(contact_id: String) -> Result<u32, String> {
    let core_guard = CORE.lock().unwrap();
    if let Some(core) = core_guard.as_ref() {
        match core.get_unread_count(&contact_id) {
            Ok(count) => Ok(count as u32),
            Err(e) => Err(format!("Failed to get unread count: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

pub async fn search_messages(
    query: String,
    contact_id: Option<String>,
) -> Result<Vec<ChatMessage>, String> {
    let core_guard = CORE.lock().unwrap();
    if let Some(core) = core_guard.clone() {
        drop(core_guard);
        match core.search_messages(&query, contact_id.as_deref()).await {
            Ok(messages) => Ok(messages),
            Err(e) => Err(format!("Failed to search messages: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

pub async fn get_message_by_id(message_id: String) -> Result<ChatMessage, String> {
    let core_guard = CORE.lock().unwrap();
    if let Some(core) = core_guard.clone() {
        drop(core_guard);
        match core.get_message(&message_id).await {
            Ok(Some(message)) => Ok(message),
            Ok(None) => Err("Message not found".to_string()),
            Err(e) => Err(format!("Failed to get message: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}
