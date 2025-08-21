use crate::events::EventBus;
use crate::peer::Peer;
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
    pub messages_sent: u64,     // Alias for compatibility
    pub messages_received: u64, // Alias for compatibility
    pub total_connections: u32, // Alias for compatibility
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
    peer: Peer,
    event_bus: EventBus,
    is_active: bool,
    connected_peers: HashMap<String, PeerData>,
    stats: NetworkStats,
    chats: Arc<RwLock<HashMap<String, Vec<ChatMessage>>>>,
}

impl NetworkManager {

    pub fn new(peer: Peer, event_bus: EventBus) -> Result<Self, NetworkError> {
        Ok(Self {
            peer,
            event_bus,
            is_active: false,
            connected_peers: HashMap::new(),
            stats: NetworkStats {
                connected_peers: 0,
                total_messages_sent: 0,
                total_messages_received: 0,
                bytes_sent: 0,
                bytes_received: 0,
                uptime_seconds: 0,
                messages_sent: 0,
                messages_received: 0,
                total_connections: 0,
            },
            chats: Arc::new(RwLock::new(HashMap::new())),
        })
    }


    pub fn new_default() -> Result<Self, NetworkError> {
        let peer = Peer::new("default_user".to_string(), "127.0.0.1:8080".to_string());
        let event_bus = EventBus::new();
        Self::new(peer, event_bus)
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

    pub async fn get_peer(&self) -> Peer {
        self.peer.clone()
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

    pub async fn send_chat_message(
        &self,
        contact: &Contact,
        content: &str,
    ) -> Result<String, NetworkError> {
        if !self.is_active {
            return Err(NetworkError::SendFailed("Network not active".to_string()));
        }

        let message_id = uuid::Uuid::new_v4().to_string();

        let message = ChatMessage {
            id: message_id.clone(),
            from: self.peer.name.clone(),
            to: contact.name.clone(),
            content: content.to_string(),
            msg_type: ChatMessageType::Text,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            delivery_status: DeliveryStatus::Sent,
        };

        let mut chats = self.chats.write().await;
        let chat_key = format!("chat_{}", contact.name);
        chats.entry(chat_key).or_insert_with(Vec::new).push(message);

        self.stats.total_messages_sent += 1;
        self.stats.messages_sent += 1;

        Ok(message_id)
    }

    pub async fn send_chat_message_by_name(
        &mut self,
        contact_name: &str,
        content: &str,
    ) -> Result<String, NetworkError> {
        if !self.is_active {
            return Err(NetworkError::SendFailed("Network not active".to_string()));
        }

        let message_id = uuid::Uuid::new_v4().to_string();

        let message = ChatMessage {
            id: message_id.clone(),
            from: self.peer.name.clone(),
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

        self.stats.total_messages_sent += 1;
        self.stats.messages_sent += 1;

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

    pub async fn get_chats(&self) -> HashMap<String, Vec<ChatMessage>> {
        let chats = self.chats.read().await;
        chats.clone()
    }

    pub async fn update_peer_name(&self, new_name: String) -> Result<(), NetworkError> {


        Ok(())
    }

    pub fn get_connected_peers(&self) -> Vec<PeerData> {
        self.connected_peers.values().cloned().collect()
    }

    pub fn add_peer(&mut self, peer_data: PeerData) {
        self.connected_peers.insert(peer_data.id.clone(), peer_data);
        self.stats.connected_peers = self.connected_peers.len() as u32;
        self.stats.total_connections = self.stats.connected_peers;
    }

    pub fn remove_peer(&mut self, peer_id: &str) {
        self.connected_peers.remove(peer_id);
        self.stats.connected_peers = self.connected_peers.len() as u32;
        self.stats.total_connections = self.stats.connected_peers;
    }

    pub fn get_peer_by_id(&self, peer_id: &str) -> Option<&PeerData> {
        self.connected_peers.get(peer_id)
    }

    pub fn is_peer_connected(&self, peer_id: &str) -> bool {
        self.connected_peers.contains_key(peer_id)
    }

    pub fn get_peer_count(&self) -> usize {
        self.connected_peers.len()
    }

    pub fn update_stats(&mut self, bytes_sent: u64, bytes_received: u64) {
        self.stats.bytes_sent += bytes_sent;
        self.stats.bytes_received += bytes_received;
    }

    pub fn increment_messages_sent(&mut self) {
        self.stats.total_messages_sent += 1;
        self.stats.messages_sent += 1;
    }

    pub fn increment_messages_received(&mut self) {
        self.stats.total_messages_received += 1;
        self.stats.messages_received += 1;
    }

    pub fn reset_stats(&mut self) {
        self.stats = NetworkStats {
            connected_peers: self.connected_peers.len() as u32,
            total_messages_sent: 0,
            total_messages_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            uptime_seconds: 0,
            messages_sent: 0,
            messages_received: 0,
            total_connections: self.connected_peers.len() as u32,
        };
    }

    pub async fn simulate_message_received(
        &self,
        from: &str,
        content: &str,
    ) -> Result<(), NetworkError> {
        let message = ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            from: from.to_string(),
            to: self.peer.name.clone(),
            content: content.to_string(),
            msg_type: ChatMessageType::Text,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            delivery_status: DeliveryStatus::Delivered,
        };

        let mut chats = self.chats.write().await;
        let chat_key = format!("chat_{}", from);
        chats
            .entry(chat_key)
            .or_insert_with(Vec::new)
            .push(message.clone());


        use crate::events::{AppEvent, NetworkEvent};
        self.event_bus
            .emit(AppEvent::Network(NetworkEvent::MessageReceived { message }));

        Ok(())
    }
}
