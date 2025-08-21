use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
pub struct MessageHeader {
    pub message_type: MessageType,
    pub sender_id: String,
    pub recipient_id: String,
    pub timestamp: u64,
    pub message_id: String,
    pub sequence_number: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandshakePayload {
    pub peer_id: String,
    pub peer_name: String,
    pub address: String,
    pub public_key: Vec<u8>,
    pub protocol_version: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPayload {
    pub content: String,
    pub message_id: String,
    pub reply_to: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PingPayload {
    pub timestamp: u64,
    pub sequence: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PongPayload {
    pub original_timestamp: u64,
    pub response_timestamp: u64,
    pub sequence: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePayload {
    pub file_name: String,
    pub file_size: u64,
    pub file_hash: String,
    pub chunk_data: Vec<u8>,
    pub chunk_index: u32,
    pub total_chunks: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AckPayload {
    pub original_message_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessagePayload {
    Handshake(HandshakePayload),
    Text(TextPayload),
    Ping(PingPayload),
    Pong(PongPayload),
    File(FilePayload),
    Ack(AckPayload),
    Empty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolMessage {
    pub header: MessageHeader,
    pub payload: MessagePayload,
    pub signature: Option<Vec<u8>>,


    pub message_type: MessageType,
    pub sender_id: String,
    pub recipient_id: String,
    pub content: Vec<u8>,
    pub timestamp: u64,
    pub message_id: String,
}

impl ProtocolMessage {
    pub fn new(
        message_type: MessageType,
        sender_id: String,
        recipient_id: String,
        content: Vec<u8>,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let message_id = uuid::Uuid::new_v4().to_string();

        let header = MessageHeader {
            message_type: message_type.clone(),
            sender_id: sender_id.clone(),
            recipient_id: recipient_id.clone(),
            timestamp,
            message_id: message_id.clone(),
            sequence_number: 0,
        };

        Self {
            header,
            payload: MessagePayload::Empty,
            signature: None,
            message_type,
            sender_id,
            recipient_id,
            content,
            timestamp,
            message_id,
        }
    }

    pub fn create_handshake(
        peer_id: String,
        peer_name: String,
        address: String,
        public_key: Vec<u8>,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let message_id = uuid::Uuid::new_v4().to_string();

        let header = MessageHeader {
            message_type: MessageType::Handshake,
            sender_id: peer_id.clone(),
            recipient_id: "broadcast".to_string(),
            timestamp,
            message_id: message_id.clone(),
            sequence_number: 0,
        };

        let payload = MessagePayload::Handshake(HandshakePayload {
            peer_id: peer_id.clone(),
            peer_name,
            address,
            public_key: public_key.clone(),
            protocol_version: PROTOCOL_VERSION,
        });

        Self {
            header,
            payload,
            signature: None,
            message_type: MessageType::Handshake,
            sender_id: peer_id,
            recipient_id: "broadcast".to_string(),
            content: public_key,
            timestamp,
            message_id,
        }
    }

    pub fn create_text_message(
        sender_id: String,
        recipient_id: String,
        content: String,
        message_id: String,
    ) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let header = MessageHeader {
            message_type: MessageType::Chat,
            sender_id: sender_id.clone(),
            recipient_id: recipient_id.clone(),
            timestamp,
            message_id: message_id.clone(),
            sequence_number: 0,
        };

        let payload = MessagePayload::Text(TextPayload {
            content: content.clone(),
            message_id: message_id.clone(),
            reply_to: None,
        });

        Self {
            header,
            payload,
            signature: None,
            message_type: MessageType::Chat,
            sender_id,
            recipient_id,
            content: content.into_bytes(),
            timestamp,
            message_id,
        }
    }

    pub fn create_ping(sender_id: String, recipient_id: String) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let message_id = uuid::Uuid::new_v4().to_string();

        let header = MessageHeader {
            message_type: MessageType::Ping,
            sender_id: sender_id.clone(),
            recipient_id: recipient_id.clone(),
            timestamp,
            message_id: message_id.clone(),
            sequence_number: 0,
        };

        let payload = MessagePayload::Ping(PingPayload {
            timestamp,
            sequence: 0,
        });

        Self {
            header,
            payload,
            signature: None,
            message_type: MessageType::Ping,
            sender_id,
            recipient_id,
            content: vec![],
            timestamp,
            message_id,
        }
    }

    pub fn create_pong(sender_id: String, recipient_id: String, original_timestamp: u64) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let message_id = uuid::Uuid::new_v4().to_string();

        let header = MessageHeader {
            message_type: MessageType::Pong,
            sender_id: sender_id.clone(),
            recipient_id: recipient_id.clone(),
            timestamp,
            message_id: message_id.clone(),
            sequence_number: 0,
        };

        let payload = MessagePayload::Pong(PongPayload {
            original_timestamp,
            response_timestamp: timestamp,
            sequence: 0,
        });

        Self {
            header,
            payload,
            signature: None,
            message_type: MessageType::Pong,
            sender_id,
            recipient_id,
            content: vec![],
            timestamp,
            message_id,
        }
    }

    pub fn ping(sender_id: String, recipient_id: String) -> Self {
        Self::create_ping(sender_id, recipient_id)
    }

    pub fn pong(sender_id: String, recipient_id: String) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self::create_pong(sender_id, recipient_id, timestamp)
    }

    pub fn chat_message(sender_id: String, recipient_id: String, content: String) -> Self {
        let message_id = uuid::Uuid::new_v4().to_string();
        Self::create_text_message(sender_id, recipient_id, content, message_id)
    }

    pub fn handshake(sender_id: String, _recipient_id: String, public_key: Vec<u8>) -> Self {
        Self::create_handshake(
            sender_id,
            "Unknown".to_string(),
            "127.0.0.1:8080".to_string(),
            public_key,
        )
    }

    pub fn acknowledgment(sender_id: String, recipient_id: String, message_id: String) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let ack_id = uuid::Uuid::new_v4().to_string();

        let header = MessageHeader {
            message_type: MessageType::Acknowledgment,
            sender_id: sender_id.clone(),
            recipient_id: recipient_id.clone(),
            timestamp,
            message_id: ack_id.clone(),
            sequence_number: 0,
        };

        let payload = MessagePayload::Ack(AckPayload {
            original_message_id: message_id.clone(),
            status: "delivered".to_string(),
        });

        Self {
            header,
            payload,
            signature: None,
            message_type: MessageType::Acknowledgment,
            sender_id,
            recipient_id,
            content: message_id.into_bytes(),
            timestamp,
            message_id: ack_id,
        }
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

    pub fn get_size(&self) -> usize {
        self.to_bytes().map(|b| b.len()).unwrap_or(0)
    }

    pub fn is_valid(&self) -> bool {
        !self.sender_id.is_empty()
            && !self.recipient_id.is_empty()
            && !self.message_id.is_empty()
            && self.timestamp > 0
    }

    pub fn get_header(&self) -> &MessageHeader {
        &self.header
    }

    pub fn get_payload(&self) -> &MessagePayload {
        &self.payload
    }

    pub fn set_sequence_number(&mut self, seq: u64) {
        self.header.sequence_number = seq;
    }

    pub fn get_sequence_number(&self) -> u64 {
        self.header.sequence_number
    }


    pub fn get_text_content(&self) -> Option<String> {
        match &self.payload {
            MessagePayload::Text(text) => Some(text.content.clone()),
            _ => None,
        }
    }

    pub fn get_ping_timestamp(&self) -> Option<u64> {
        match &self.payload {
            MessagePayload::Ping(ping) => Some(ping.timestamp),
            _ => None,
        }
    }

    pub fn get_pong_timestamps(&self) -> Option<(u64, u64)> {
        match &self.payload {
            MessagePayload::Pong(pong) => Some((pong.original_timestamp, pong.response_timestamp)),
            _ => None,
        }
    }

    pub fn get_handshake_info(&self) -> Option<&HandshakePayload> {
        match &self.payload {
            MessagePayload::Handshake(handshake) => Some(handshake),
            _ => None,
        }
    }
}

pub const PROTOCOL_VERSION: u8 = 1;
pub const MAX_MESSAGE_SIZE: usize = 1024 * 1024;
pub const HANDSHAKE_TIMEOUT: u64 = 30;
pub const MESSAGE_TIMEOUT: u64 = 60;


pub fn validate_message_size(data: &[u8]) -> bool {
    data.len() <= MAX_MESSAGE_SIZE
}

pub fn is_protocol_compatible(version: u8) -> bool {
    version == PROTOCOL_VERSION
}

pub fn create_error_response(original_message_id: String, error: String) -> ProtocolMessage {
    ProtocolMessage::acknowledgment(
        "system".to_string(),
        "error".to_string(),
        format!("{}:{}", original_message_id, error),
    )
}
