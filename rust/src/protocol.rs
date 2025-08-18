use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Handshake,
    TextMessage,
    FileTransfer,
    StatusUpdate,
    Ping,
    Pong,
    KeyExchange,
    Disconnect,
    FileShare,
    VoiceCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolHeader {
    pub version: u8,
    pub message_type: MessageType,
    pub sender_id: String,
    pub recipient_id: String,
    pub timestamp: u64,
    pub sequence_number: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakeMessage {
    pub peer_id: String,
    pub peer_name: String,
    pub listen_address: String,
    pub public_key: Vec<u8>,
    pub protocol_version: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextMessage {
    pub content: String,
    pub message_id: String,
    pub reply_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMessage {
    pub file_name: String,
    pub file_size: u64,
    pub file_hash: String,
    pub chunk_data: Vec<u8>,
    pub chunk_index: u32,
    pub total_chunks: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusMessage {
    pub status: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingMessage {
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PongMessage {
    pub original_timestamp: u64,
    pub response_timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    Handshake(HandshakeMessage),
    Text(TextMessage),
    File(FileMessage),
    Status(StatusMessage),
    Ping(PingMessage),
    Pong(PongMessage),
    KeyExchange(Vec<u8>),
    Disconnect,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMessage {
    pub header: ProtocolHeader,
    pub payload: MessagePayload,
    pub signature: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub from: String,
    pub to: String,
    pub msg_type: MessageType,
    pub content: Vec<u8>,
    pub timestamp: u64,
}

impl ProtocolMessage {
    pub fn new(
        message_type: MessageType,
        sender_id: String,
        recipient_id: String,
        payload: MessagePayload,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            header: ProtocolHeader {
                version: 1,
                message_type,
                sender_id,
                recipient_id,
                timestamp,
                sequence_number: 0,
            },
            payload,
            signature: None,
        }
    }

    pub fn create_handshake(
        sender_id: String,
        sender_name: String,
        listen_address: String,
        public_key: Vec<u8>,
    ) -> Self {
        Self::new(
            MessageType::Handshake,
            sender_id.clone(),
            "broadcast".to_string(),
            MessagePayload::Handshake(HandshakeMessage {
                peer_id: sender_id,
                peer_name: sender_name,
                listen_address,
                public_key,
                protocol_version: 1,
            }),
        )
    }

    pub fn create_text_message(
        sender_id: String,
        recipient_id: String,
        content: String,
        message_id: String,
    ) -> Self {
        Self::new(
            MessageType::TextMessage,
            sender_id,
            recipient_id,
            MessagePayload::Text(TextMessage {
                content,
                message_id,
                reply_to: None,
            }),
        )
    }

    pub fn create_ping(sender_id: String, recipient_id: String) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self::new(
            MessageType::Ping,
            sender_id,
            recipient_id,
            MessagePayload::Ping(PingMessage { timestamp }),
        )
    }

    pub fn create_pong(sender_id: String, recipient_id: String, original_timestamp: u64) -> Self {
        let response_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self::new(
            MessageType::Pong,
            sender_id,
            recipient_id,
            MessagePayload::Pong(PongMessage {
                original_timestamp,
                response_timestamp,
            }),
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
}
