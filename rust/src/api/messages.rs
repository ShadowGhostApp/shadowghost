use super::core::CORE;
use crate::network::{ChatMessage, ChatMessageType, DeliveryStatus};
use chrono::Utc;
use flutter_rust_bridge::frb;

pub async fn send_text_message(contact_id: String, content: String) -> Result<String, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        let message = ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            from: "local_user".to_string(),
            to: contact_id,
            content,
            timestamp: Utc::now().timestamp() as u64,
            msg_type: ChatMessageType::Text,
            delivery_status: DeliveryStatus::Pending,
        };

        match core
            .lock()
            .await
            .send_chat_message(&message.to, &message.content)
            .await
        {
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
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        let message = ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            from: "local_user".to_string(),
            to: contact_id,
            content: format!("{}|{}", file_name, file_path),
            timestamp: Utc::now().timestamp() as u64,
            msg_type: ChatMessageType::File,
            delivery_status: DeliveryStatus::Pending,
        };

        match core
            .lock()
            .await
            .send_chat_message(&message.to, &message.content)
            .await
        {
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
    _offset: u32,
) -> Result<Vec<ChatMessage>, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        match core.lock().await.get_chat_messages(&contact_id).await {
            Ok(mut messages) => {
                messages.truncate(limit as usize);
                Ok(messages)
            }
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
    get_messages(contact_id, limit, 0).await
}

#[frb(sync)]
pub fn get_unread_message_count(contact_id: String) -> Result<u32, String> {
    let rt = tokio::runtime::Handle::current();
    let core_guard = rt.block_on(CORE.lock());
    if let Some(core) = core_guard.as_ref() {
        match rt.block_on(core.lock()).get_unread_count(&contact_id) {
            Ok(count) => Ok(count as u32),
            Err(e) => Err(format!("Failed to get unread count: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}
