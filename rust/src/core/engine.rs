// This file contains implementations for the ShadowGhostCore
// The actual implementations are now moved to the main ShadowGhostCore impl block in mod.rs

use super::CoreError;
use crate::network::ChatMessage;

// Additional engine-specific functions can be implemented here
pub struct EngineUtils;

impl EngineUtils {
    pub fn validate_message_content(content: &str) -> Result<(), CoreError> {
        if content.is_empty() {
            return Err(CoreError::InvalidState(
                "Message content cannot be empty".to_string(),
            ));
        }

        if content.len() > 10000 {
            return Err(CoreError::InvalidState(
                "Message content too long".to_string(),
            ));
        }

        Ok(())
    }

    pub fn create_message_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    pub fn get_current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    pub fn format_chat_message(
        from: &str,
        to: &str,
        content: &str,
        msg_type: crate::network::ChatMessageType,
    ) -> ChatMessage {
        ChatMessage {
            id: Self::create_message_id(),
            from: from.to_string(),
            to: to.to_string(),
            content: content.to_string(),
            msg_type,
            timestamp: Self::get_current_timestamp(),
            delivery_status: crate::network::DeliveryStatus::Pending,
        }
    }
}
