use crate::chats::{Chat, ChatExportOptions, ChatInfo, ChatStatistics, MessageFilter};
use crate::core::ENGINE;
use crate::network::{ChatMessage, ChatMessageType, DeliveryStatus};
use flutter_rust_bridge::frb;

#[frb]
pub async fn create_chat(name: String, is_group: bool) -> Result<Chat, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .chats()
        .create_chat(name, is_group)
        .await
        .map_err(|e| e.to_string())
}

#[frb]
pub async fn get_chat(chat_id: String) -> Result<Chat, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .chats()
        .get_chat(&chat_id)
        .await
        .map_err(|e| e.to_string())
}

#[frb]
pub async fn get_all_chats() -> Result<Vec<Chat>, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    Ok(engine.chats().get_all_chats().await)
}

#[frb]
pub async fn send_message(
    chat_id: String,
    sender: String,
    recipient: String,
    content: String,
    message_type: ChatMessageType,
) -> Result<String, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .chats()
        .send_message(&chat_id, &sender, &recipient, &content, message_type)
        .await
        .map_err(|e| e.to_string())
}

#[frb]
pub async fn get_messages(
    chat_id: String,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Result<Vec<ChatMessage>, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .chats()
        .get_messages(&chat_id, limit, offset)
        .await
        .map_err(|e| e.to_string())
}

#[frb]
pub async fn get_chat_info(chat_id: String) -> Result<ChatInfo, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .chats()
        .get_chat_info(&chat_id)
        .await
        .map_err(|e| e.to_string())
}

#[frb]
pub async fn delete_chat(chat_id: String) -> Result<(), String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .chats()
        .delete_chat(&chat_id)
        .await
        .map_err(|e| e.to_string())
}

#[frb]
pub async fn update_message_status(
    message_id: String,
    new_status: DeliveryStatus,
) -> Result<(), String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .chats()
        .update_message_status(&message_id, new_status)
        .await
        .map_err(|e| e.to_string())
}

#[frb]
pub async fn get_chat_statistics() -> Result<ChatStatistics, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    Ok(engine.chats().get_statistics().await)
}

#[frb]
pub async fn export_chat(chat_id: String, options: ChatExportOptions) -> Result<String, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .chats()
        .export_chat(&chat_id, options)
        .await
        .map_err(|e| e.to_string())
}

#[frb]
pub async fn search_messages_by_content(
    chat_id: Option<String>,
    content: String,
) -> Result<Vec<ChatMessage>, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;

    let filter = MessageFilter {
        chat_id,
        content_search: Some(content),
        ..Default::default()
    };

    let results = engine
        .chats()
        .search_messages(filter)
        .await
        .map_err(|e| e.to_string())?;

    Ok(results.into_iter().flat_map(|r| r.messages).collect())
}

#[frb]
pub async fn get_unread_message_count(chat_id: String) -> Result<u64, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    let info = engine
        .chats()
        .get_chat_info(&chat_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(info.unread_count)
}

#[frb]
pub async fn get_failed_messages(chat_id: Option<String>) -> Result<Vec<ChatMessage>, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;

    let filter = MessageFilter {
        chat_id,
        delivery_status: Some(DeliveryStatus::Failed),
        ..Default::default()
    };

    let results = engine
        .chats()
        .search_messages(filter)
        .await
        .map_err(|e| e.to_string())?;

    Ok(results.into_iter().flat_map(|r| r.messages).collect())
}
