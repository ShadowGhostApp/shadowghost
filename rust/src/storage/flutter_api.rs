use crate::core::ENGINE;
use crate::network::{ChatMessage, Contact, DeliveryStatus};
use crate::storage::StorageStats;
use flutter_rust_bridge::frb;

#[frb]
pub async fn save_message(chat_id: String, message: ChatMessage) -> Result<(), String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .storage()
        .save_message(&chat_id, &message)
        .await
        .map_err(|e| e.to_string())
}

#[frb]
pub async fn get_messages(chat_id: String) -> Result<Vec<ChatMessage>, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .storage()
        .get_messages(&chat_id)
        .await
        .map_err(|e| e.to_string())
}

#[frb]
pub async fn delete_message(message_id: String) -> Result<(), String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .storage()
        .delete_message(&message_id)
        .await
        .map_err(|e| e.to_string())
}

#[frb]
pub async fn delete_chat(chat_id: String) -> Result<(), String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .storage()
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
        .storage()
        .update_message_status(&message_id, new_status)
        .await
        .map_err(|e| e.to_string())
}

#[frb]
pub async fn save_contact(contact: Contact) -> Result<(), String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .storage()
        .save_contact(&contact)
        .await
        .map_err(|e| e.to_string())
}

#[frb]
pub async fn get_contacts() -> Result<Vec<Contact>, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .storage()
        .get_contacts()
        .await
        .map_err(|e| e.to_string())
}

#[frb]
pub async fn delete_contact(contact_id: String) -> Result<(), String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .storage()
        .delete_contact(&contact_id)
        .await
        .map_err(|e| e.to_string())
}

#[frb]
pub async fn create_backup() -> Result<String, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine.storage().backup().await.map_err(|e| e.to_string())
}

#[frb]
pub async fn restore_from_backup(backup_path: String) -> Result<(), String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .storage()
        .restore_from_backup(&backup_path)
        .await
        .map_err(|e| e.to_string())
}

#[frb]
pub async fn get_storage_stats() -> Result<StorageStats, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .storage()
        .get_stats()
        .await
        .map_err(|e| e.to_string())
}

#[frb]
pub async fn validate_storage() -> Result<Vec<String>, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .storage()
        .validate_chats()
        .await
        .map_err(|e| e.to_string())
}

#[frb]
pub async fn export_chat(chat_id: String, format: String) -> Result<String, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .storage()
        .export_chat_data(&chat_id, &format)
        .await
        .map_err(|e| e.to_string())
}

#[frb]
pub async fn get_chat_message_count(chat_id: String) -> Result<u64, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    let messages = engine
        .storage()
        .get_messages(&chat_id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(messages.len() as u64)
}

#[frb]
pub async fn search_messages_by_content(
    search_term: String,
    chat_id: Option<String>,
) -> Result<Vec<ChatMessage>, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;

    if let Some(chat_id) = chat_id {
        let messages = engine
            .storage()
            .get_messages(&chat_id)
            .await
            .map_err(|e| e.to_string())?;

        Ok(messages
            .into_iter()
            .filter(|m| {
                m.content
                    .to_lowercase()
                    .contains(&search_term.to_lowercase())
            })
            .collect())
    } else {
        // Search across all chats would need additional implementation
        Ok(Vec::new())
    }
}

#[frb]
pub async fn get_failed_messages() -> Result<Vec<ChatMessage>, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    let stats = engine
        .storage()
        .get_stats()
        .await
        .map_err(|e| e.to_string())?;

    // This is a simplified implementation
    // In reality, you'd need to search through all chats
    Ok(Vec::new())
}
