use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Ping,
    Pong,
    Chat,
    File,
    Handshake,
    Acknowledgment,
    KeyExchange,
    Status,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMessage {
    pub message_type: MessageType,
    pub sender_id: String,
    pub recipient_id: String,
    pub content: Vec<u8>,
    pub timestamp: u64,
    pub message_id: String,
    pub signature: Option<Vec<u8>>,
}

impl ProtocolMessage {
    pub fn new(
        message_type: MessageType,
        sender_id: String,
        recipient_id: String,
        content: Vec<u8>,
    ) -> Self {
        Self {
            message_type,
            sender_id,
            recipient_id,
            content,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            message_id: uuid::Uuid::new_v4().to_string(),
            signature: None,
        }
    }

    pub fn ping(sender_id: String, recipient_id: String) -> Self {
        Self::new(MessageType::Ping, sender_id, recipient_id, vec![])
    }

    pub fn pong(sender_id: String, recipient_id: String) -> Self {
        Self::new(MessageType::Pong, sender_id, recipient_id, vec![])
    }

    pub fn chat_message(sender_id: String, recipient_id: String, content: String) -> Self {
        Self::new(
            MessageType::Chat,
            sender_id,
            recipient_id,
            content.into_bytes(),
        )
    }

    pub fn handshake(sender_id: String, recipient_id: String, public_key: Vec<u8>) -> Self {
        Self::new(MessageType::Handshake, sender_id, recipient_id, public_key)
    }

    pub fn acknowledgment(sender_id: String, recipient_id: String, message_id: String) -> Self {
        Self::new(
            MessageType::Acknowledgment,
            sender_id,
            recipient_id,
            message_id.into_bytes(),
        )
    }

    pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let data = serde_json::to_vec(self)?;
        Ok(data)
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let message = serde_json::from_slice(data)?;
        Ok(message)
    }

    pub fn sign(&mut self, signature: Vec<u8>) {
        self.signature = Some(signature);
    }

    pub fn verify_signature(&self, _public_key: &[u8]) -> bool {
        self.signature.is_some()
    }
}

pub const PROTOCOL_VERSION: u8 = 1;
pub const MAX_MESSAGE_SIZE: usize = 1024 * 1024;
pub const HANDSHAKE_TIMEOUT: u64 = 30;
pub const MESSAGE_TIMEOUT: u64 = 60;
