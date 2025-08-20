use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContactStatus {
    Online,
    Offline,
    Away,
    Busy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrustLevel {
    Unknown,
    Pending,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatMessageType {
    Text,
    File,
    Image,
    Voice,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

pub struct NetworkManager {
    is_active: bool,
    connected_peers: HashMap<String, PeerData>,
    stats: NetworkStats,
    chats: Arc<RwLock<HashMap<String, Vec<ChatMessage>>>>,
    _event_bus: Option<Arc<dyn std::any::Any + Send + Sync>>,
}

impl NetworkManager {
    pub fn new() -> Result<Self, NetworkError> {
        Ok(Self {
            is_active: false,
            connected_peers: HashMap::new(),
            stats: NetworkStats {
                connected_peers: 0,
                total_messages_sent: 0,
                total_messages_received: 0,
                bytes_sent: 0,
                bytes_received: 0,
                uptime_seconds: 0,
            },
            chats: Arc::new(RwLock::new(HashMap::new())),
            _event_bus: None,
        })
    }

    pub fn start(&mut self) -> Result<(), NetworkError> {
        self.is_active = true;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), NetworkError> {
        self.is_active = false;
        self.connected_peers.clear();
        self.stats.connected_peers = 0;
        Ok(())
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn is_running(&self) -> bool {
        self.is_active
    }

    pub async fn start_server(&mut self) -> Result<(), NetworkError> {
        self.is_active = true;
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), NetworkError> {
        Ok(())
    }

    pub fn get_stats(&self) -> Result<NetworkStats, NetworkError> {
        Ok(self.stats.clone())
    }

    pub async fn get_network_stats(&self) -> Result<NetworkStats, NetworkError> {
        Ok(self.stats.clone())
    }

    pub fn get_connected_peers(&self) -> Result<Vec<PeerData>, NetworkError> {
        Ok(self.connected_peers.values().cloned().collect())
    }

    pub fn connect_to_peer(&mut self, peer_address: &str) -> Result<(), NetworkError> {
        if !self.is_active {
            return Err(NetworkError::ConnectionFailed(
                "Network not active".to_string(),
            ));
        }

        let peer_id = format!("peer_{}", uuid::Uuid::new_v4());
        let peer_data = PeerData {
            id: peer_id.clone(),
            name: "Unknown".to_string(),
            address: peer_address.to_string(),
            public_key: vec![],
            connected_at: Utc::now(),
            last_seen: Utc::now(),
            bytes_sent: 0,
            bytes_received: 0,
        };

        self.connected_peers.insert(peer_id, peer_data);
        self.stats.connected_peers = self.connected_peers.len() as u32;
        Ok(())
    }

    pub fn disconnect_from_peer(&mut self, peer_id: &str) -> Result<(), NetworkError> {
        self.connected_peers.remove(peer_id);
        self.stats.connected_peers = self.connected_peers.len() as u32;
        Ok(())
    }

    pub fn send_message(
        &mut self,
        _contact_id: &str,
        content: &str,
    ) -> Result<String, NetworkError> {
        if !self.is_active {
            return Err(NetworkError::SendFailed("Network not active".to_string()));
        }

        let message_id = uuid::Uuid::new_v4().to_string();
        self.stats.total_messages_sent += 1;
        self.stats.bytes_sent += content.len() as u64;

        Ok(message_id)
    }

    pub async fn send_chat_message(
        &self,
        contact_name: &str,
        content: &str,
    ) -> Result<String, NetworkError> {
        if !self.is_active {
            return Err(NetworkError::SendFailed("Network not active".to_string()));
        }

        let message_id = uuid::Uuid::new_v4().to_string();

        let message = ChatMessage {
            id: message_id.clone(),
            from: "local_user".to_string(),
            to: contact_name.to_string(),
            content: content.to_string(),
            msg_type: ChatMessageType::Text,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            delivery_status: DeliveryStatus::Sent,
        };

        let mut chats = self.chats.write().await;
        let chat_key = format!("chat_{}", contact_name);
        chats.entry(chat_key).or_insert_with(Vec::new).push(message);

        Ok(message_id)
    }

    pub async fn get_chat_messages(
        &self,
        contact_name: &str,
    ) -> Result<Vec<ChatMessage>, NetworkError> {
        let chats = self.chats.read().await;
        let chat_key = format!("chat_{}", contact_name);
        Ok(chats.get(&chat_key).cloned().unwrap_or_default())
    }

    pub async fn get_chats(&self) -> Result<HashMap<String, Vec<ChatMessage>>, NetworkError> {
        let chats = self.chats.read().await;
        Ok(chats.clone())
    }

    pub async fn get_peer(&self, peer_id: &str) -> Option<PeerData> {
        self.connected_peers.get(peer_id).cloned()
    }

    pub fn get_address(&self) -> Result<String, NetworkError> {
        Ok("127.0.0.1:8080".to_string())
    }

    pub fn ping_peer(&self, peer_id: &str) -> Result<u64, NetworkError> {
        if self.connected_peers.contains_key(peer_id) {
            Ok(50)
        } else {
            Err(NetworkError::ConnectionFailed(
                "Peer not connected".to_string(),
            ))
        }
    }

    pub fn update_config(&mut self, _max_peers: usize, _port: u16) -> Result<(), NetworkError> {
        Ok(())
    }

    pub async fn update_peer_address(&self, _new_address: String) -> Result<(), NetworkError> {
        Ok(())
    }

    pub async fn update_peer_name(&self, _new_name: String) -> Result<(), NetworkError> {
        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_crypto(&self) -> Option<Arc<RwLock<dyn std::any::Any + Send + Sync>>> {
        None
    }
}
