use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::net::IpAddr;

// Core network types
#[derive(Debug)]
pub enum NetworkError {
    ConnectionFailed(String),
    SendFailed(String),
    InvalidAddress(String),
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            NetworkError::SendFailed(msg) => write!(f, "Send failed: {}", msg),
            NetworkError::InvalidAddress(msg) => write!(f, "Invalid address: {}", msg),
        }
    }
}

impl Error for NetworkError {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContactStatus {
    Online,
    Offline,
    Away,
    Busy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrustLevel {
    Unknown,
    Pending,
    Low,
    Medium,
    High,
    Trusted,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: String,
    pub name: String,
    pub address: String,
    pub status: ContactStatus,
    pub trust_level: TrustLevel,
    pub last_seen: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChatMessageType {
    Text,
    File,
    Image,
    Voice,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeliveryStatus {
    Pending,
    Sent,
    Delivered,
    Read,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub from: String,
    pub to: String,
    pub content: String,
    pub msg_type: ChatMessageType,
    pub timestamp: u64,
    pub delivery_status: DeliveryStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub connected_peers: u32,
    pub total_messages_sent: u64,
    pub total_messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub uptime_seconds: u64,
    pub messages_sent: u64,     // Required field
    pub messages_received: u64, // Required field
    pub total_connections: u32, // Required field
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerData {
    pub id: String,
    pub name: String,
    pub address: String,
    pub public_key: Vec<u8>,
    pub connected_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

// Protocol types
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

// Discovery types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPeer {
    pub id: String,
    pub address: IpAddr,
    pub port: u16,
    pub name: String,
    pub last_seen: u64,
    pub public_key: Vec<u8>,
    pub protocol_version: u8,
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnouncementMessage {
    pub peer_id: String,
    pub peer_name: String,
    pub port: u16,
    pub public_key: Vec<u8>,
    pub protocol_version: u8,
    pub capabilities: Vec<String>,
    pub timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct DiscoveryStatistics {
    pub total_discovered: usize,
    pub active_peers: usize,
    pub unique_capabilities: usize,
    pub discovery_uptime: u64,
    pub last_discovery: chrono::DateTime<chrono::Utc>,
}

// TLS Masking types
#[derive(Debug)]
pub enum TlsError {
    HandshakeFailed(String),
    CertificateError(String),
    ConnectionError(String),
}

impl fmt::Display for TlsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TlsError::HandshakeFailed(msg) => write!(f, "TLS handshake failed: {}", msg),
            TlsError::CertificateError(msg) => write!(f, "Certificate error: {}", msg),
            TlsError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
        }
    }
}

impl Error for TlsError {}

// Constants
pub const PROTOCOL_VERSION: u8 = 1;
pub const MAX_MESSAGE_SIZE: usize = 1024 * 1024;
pub const HANDSHAKE_TIMEOUT: u64 = 30;
pub const MESSAGE_TIMEOUT: u64 = 60;

// Helper functions
pub fn validate_message_size(data: &[u8]) -> bool {
    data.len() <= MAX_MESSAGE_SIZE
}

pub fn is_protocol_compatible(version: u8) -> bool {
    version == PROTOCOL_VERSION
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

    pub fn to_bytes(&self) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let data = serde_json::to_vec(self)?;
        Ok(data)
    }

    pub fn from_bytes(data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let message = serde_json::from_slice(data)?;
        Ok(message)
    }

    pub fn is_valid(&self) -> bool {
        !self.sender_id.is_empty()
            && !self.recipient_id.is_empty()
            && !self.message_id.is_empty()
            && self.timestamp > 0
    }
}