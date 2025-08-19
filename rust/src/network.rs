use crate::crypto::CryptoManager;
use crate::events::EventBus;
use crate::peer::Peer;
use crate::protocol::MessageType;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Notify, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChatMessageType {
    Text,
    File,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ContactStatus {
    Online,
    Offline,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DeliveryStatus {
    Pending,
    Sent,
    Delivered,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrustLevel {
    Unknown,
    Low,
    Medium,
    High,
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
pub struct Contact {
    pub id: String,
    pub name: String,
    pub address: String,
    pub status: ContactStatus,
    pub trust_level: TrustLevel,
    pub last_seen: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerData {
    pub id: String,
    pub name: String,
    pub address: String,
    pub public_key: Vec<u8>,
    pub connected_at: u64,
}

#[derive(Debug, Default, Clone)]
pub struct NetworkStats {
    pub total_connections: u32,
    pub active_connections: u32,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkMessage {
    pub message_type: MessageType,
    pub sender_id: String,
    pub recipient_id: String,
    pub content: Vec<u8>,
    pub timestamp: u64,
    pub message_id: Option<String>,
}

#[derive(Clone)]
pub struct NetworkManager {
    peer: Arc<RwLock<Peer>>,
    chats: Arc<RwLock<HashMap<String, Vec<ChatMessage>>>>,
    crypto: Arc<RwLock<CryptoManager>>,
    pub event_bus: EventBus,
    stats: Arc<RwLock<NetworkStats>>,
    blocked_peers: Arc<RwLock<HashMap<String, bool>>>,
    shutdown_signal: Arc<Notify>,
    is_running: Arc<AtomicBool>,
    server_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
    pending_acknowledgments: Arc<RwLock<HashMap<String, String>>>,
}

impl NetworkManager {
    pub fn new(peer: Peer, event_bus: EventBus) -> Result<Self, Box<dyn std::error::Error>> {
        let crypto = CryptoManager::new()?;

        Ok(Self {
            peer: Arc::new(RwLock::new(peer)),
            chats: Arc::new(RwLock::new(HashMap::new())),
            crypto: Arc::new(RwLock::new(crypto)),
            event_bus,
            stats: Arc::new(RwLock::new(NetworkStats::default())),
            blocked_peers: Arc::new(RwLock::new(HashMap::new())),
            shutdown_signal: Arc::new(Notify::new()),
            is_running: Arc::new(AtomicBool::new(false)),
            server_handle: Arc::new(RwLock::new(None)),
            pending_acknowledgments: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn get_peer(&self) -> Peer {
        let peer = self.peer.read().await;
        peer.clone()
    }

    pub async fn update_peer_name(&self, new_name: String) {
        let mut peer = self.peer.write().await;
        peer.name = new_name;
    }

    pub async fn update_peer_address(&self, new_address: String) {
        let mut peer = self.peer.write().await;
        peer.address = new_address;
    }

    pub fn get_crypto(&self) -> Arc<RwLock<CryptoManager>> {
        self.crypto.clone()
    }

    pub async fn get_chats(&self) -> HashMap<String, Vec<ChatMessage>> {
        let chats = self.chats.read().await;
        chats.clone()
    }

    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }

    async fn create_chat_key(&self, contact_name: &str) -> String {
        let peer = self.peer.read().await;
        if peer.name.as_str() < contact_name {
            format!("{}_{}", peer.name, contact_name)
        } else {
            format!("{}_{}", contact_name, peer.name)
        }
    }

    pub async fn start_server(
        &self,
        port: u16,
        contacts: Arc<RwLock<HashMap<String, Contact>>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_running.load(Ordering::Relaxed) {
            return Err("Server is already running".into());
        }

        let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
        self.is_running.store(true, Ordering::Relaxed);

        self.event_bus
            .emit_network(crate::events::NetworkEvent::ServerStarted { port });

        let manager = self.clone();
        let contacts_ref = contacts.clone();
        let shutdown_signal = self.shutdown_signal.clone();
        let is_running = self.is_running.clone();

        let handle = tokio::spawn(async move {
            loop {
                tokio::select! {
                    accept_result = listener.accept() => {
                        match accept_result {
                            Ok((stream, addr)) => {
                                println!("New connection from: {}", addr);
                                let manager_clone = manager.clone();
                                let contacts_clone = contacts_ref.clone();

                                tokio::spawn(async move {
                                    if let Err(e) = manager_clone.handle_connection(stream, contacts_clone).await {
                                        manager_clone
                                            .event_bus
                                            .emit_network(crate::events::NetworkEvent::Error {
                                                error: e.to_string(),
                                                context: Some("Connection handling".to_string()),
                                            });
                                    }
                                });
                            }
                            Err(e) => {
                                println!("Accept error: {}", e);
                                tokio::time::sleep(Duration::from_millis(100)).await;
                            }
                        }
                    }
                    _ = shutdown_signal.notified() => {
                        println!("Server shutdown signal received");
                        break;
                    }
                }
            }

            is_running.store(false, Ordering::Relaxed);
            println!("Server loop ended");
        });

        {
            let mut server_handle = self.server_handle.write().await;
            *server_handle = Some(handle);
        }

        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.is_running.load(Ordering::Relaxed) {
            return Ok(());
        }

        println!("Initiating network manager shutdown...");
        self.shutdown_signal.notify_one();

        let handle = {
            let mut server_handle = self.server_handle.write().await;
            server_handle.take()
        };

        if let Some(handle) = handle {
            let _ = tokio::time::timeout(Duration::from_secs(5), handle).await;
        }

        self.is_running.store(false, Ordering::Relaxed);
        println!("Network manager shutdown completed");

        self.event_bus
            .emit_network(crate::events::NetworkEvent::ServerStopped);

        Ok(())
    }

    async fn handle_connection(
        &self,
        mut stream: TcpStream,
        contacts: Arc<RwLock<HashMap<String, Contact>>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use tokio::io::AsyncReadExt;

        let mut buffer = [0; 4096];

        match tokio::time::timeout(Duration::from_secs(15), stream.read(&mut buffer)).await {
            Ok(Ok(n)) => {
                if n > 0 {
                    if let Ok(message) = serde_json::from_slice::<NetworkMessage>(&buffer[..n]) {
                        if self.is_peer_blocked(&message.sender_id).await {
                            println!("Blocked message from: {}", message.sender_id);
                            return Ok(());
                        }

                        let peer = self.peer.read().await;
                        Self::process_message(
                            &message,
                            &peer,
                            contacts,
                            self.chats.clone(),
                            self.crypto.clone(),
                            self.event_bus.clone(),
                            self.pending_acknowledgments.clone(),
                            stream,
                        )
                        .await?;

                        let mut stats = self.stats.write().await;
                        stats.messages_received += 1;
                        stats.bytes_received += n as u64;
                    } else {
                        println!("Failed to parse message from stream");
                    }
                }
                if n == 0 {
                    println!("Connection closed by peer");
                }
            }
            Ok(Err(e)) => {
                println!("Stream read error: {}", e);
            }
            Err(_) => {
                println!("Connection timeout during read");
            }
        }

        Ok(())
    }

    async fn is_peer_blocked(&self, peer_id: &str) -> bool {
        let blocked = self.blocked_peers.read().await;
        blocked.get(peer_id).copied().unwrap_or(false)
    }

    pub async fn process_message(
        message: &NetworkMessage,
        peer: &Peer,
        contacts: Arc<RwLock<HashMap<String, Contact>>>,
        chats: Arc<RwLock<HashMap<String, Vec<ChatMessage>>>>,
        _crypto: Arc<RwLock<CryptoManager>>,
        event_bus: EventBus,
        pending_acknowledgments: Arc<RwLock<HashMap<String, String>>>,
        mut stream: TcpStream,
    ) -> Result<(), Box<dyn std::error::Error>> {
        match message.message_type {
            MessageType::TextMessage => {
                let contacts_guard = contacts.read().await;
                let sender_contact = contacts_guard.get(&message.sender_id);
                let sender_name = sender_contact
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| message.sender_id.clone());
                drop(contacts_guard);

                let content = String::from_utf8_lossy(&message.content);
                let chat_message = ChatMessage {
                    id: message
                        .message_id
                        .clone()
                        .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                    from: sender_name.clone(),
                    to: peer.name.clone(),
                    content: content.to_string(),
                    msg_type: ChatMessageType::Text,
                    timestamp: message.timestamp,
                    delivery_status: DeliveryStatus::Delivered,
                };

                let chat_key = if peer.name < sender_name {
                    format!("{}_{}", peer.name, sender_name)
                } else {
                    format!("{}_{}", sender_name, peer.name)
                };

                {
                    let mut chats_guard = chats.write().await;
                    chats_guard
                        .entry(chat_key.clone())
                        .or_default()
                        .push(chat_message.clone());
                }

                if let Some(msg_id) = &message.message_id {
                    let acknowledgment_message = NetworkMessage {
                        message_type: MessageType::MessageAcknowledgment,
                        sender_id: peer.id.clone(),
                        recipient_id: message.sender_id.clone(),
                        content: msg_id.as_bytes().to_vec(),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        message_id: None,
                    };

                    let _ = Self::send_acknowledgment_through_stream(
                        &mut stream,
                        &acknowledgment_message,
                    )
                    .await;
                }

                event_bus.emit_network(crate::events::NetworkEvent::MessageReceived {
                    message: chat_message,
                });
            }
            MessageType::MessageAcknowledgment => {
                let msg_id = String::from_utf8_lossy(&message.content).to_string();
                {
                    let mut pending = pending_acknowledgments.write().await;
                    if let Some(chat_key) = pending.remove(&msg_id) {
                        let mut chats_guard = chats.write().await;
                        if let Some(messages) = chats_guard.get_mut(&chat_key) {
                            if let Some(msg) = messages.iter_mut().find(|m| m.id == msg_id) {
                                msg.delivery_status = DeliveryStatus::Delivered;
                                println!("Message {} marked as delivered", msg_id);
                            }
                        }
                    }
                }
            }
            MessageType::Ping => {
                let pong_message = NetworkMessage {
                    message_type: MessageType::Pong,
                    sender_id: peer.id.clone(),
                    recipient_id: message.sender_id.clone(),
                    content: message.content.clone(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    message_id: None,
                };

                let _ = Self::send_acknowledgment_through_stream(&mut stream, &pong_message).await;
            }
            MessageType::Handshake => {
                let contact_data: PeerData = serde_json::from_slice(&message.content)?;
                let contact = Contact {
                    id: contact_data.id.clone(),
                    name: contact_data.name.clone(),
                    address: contact_data.address,
                    status: ContactStatus::Online,
                    trust_level: TrustLevel::Low,
                    last_seen: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)?
                        .as_secs(),
                };

                {
                    let mut contacts_guard = contacts.write().await;
                    contacts_guard.insert(contact_data.id, contact.clone());
                }

                event_bus.emit_network(crate::events::NetworkEvent::ContactAdded { contact });
            }
            _ => {
                println!("Unhandled message type: {:?}", message.message_type);
            }
        }

        Ok(())
    }

    async fn send_acknowledgment_through_stream(
        stream: &mut TcpStream,
        message: &NetworkMessage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use tokio::io::AsyncWriteExt;
        let data = serde_json::to_vec(message)?;

        match tokio::time::timeout(Duration::from_secs(5), stream.write_all(&data)).await {
            Ok(Ok(_)) => {
                let _ = stream.flush().await;
                Ok(())
            }
            Ok(Err(e)) => Err(format!("Write error: {}", e).into()),
            Err(_) => Err("Write timeout".into()),
        }
    }

    pub async fn send_chat_message(
        &self,
        contact: &Contact,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let peer = self.peer.read().await;
        let message_id = uuid::Uuid::new_v4().to_string();

        let message = NetworkMessage {
            message_type: MessageType::TextMessage,
            sender_id: peer.id.clone(),
            recipient_id: contact.id.clone(),
            content: content.as_bytes().to_vec(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            message_id: Some(message_id.clone()),
        };

        let mut chat_message = ChatMessage {
            id: message_id.clone(),
            from: peer.name.clone(),
            to: contact.name.clone(),
            content: content.to_string(),
            msg_type: ChatMessageType::Text,
            timestamp: message.timestamp,
            delivery_status: DeliveryStatus::Pending,
        };

        let chat_key = if peer.name < contact.name {
            format!("{}_{}", peer.name, contact.name)
        } else {
            format!("{}_{}", contact.name, peer.name)
        };

        match Self::send_message_with_multiple_attempts(&contact.address, &message).await {
            Ok(true) => {
                chat_message.delivery_status = DeliveryStatus::Delivered;

                let mut stats = self.stats.write().await;
                stats.messages_sent += 1;
                stats.bytes_sent += content.len() as u64;
            }
            Ok(false) => {
                chat_message.delivery_status = DeliveryStatus::Sent;

                {
                    let mut pending = self.pending_acknowledgments.write().await;
                    pending.insert(message_id.clone(), chat_key.clone());
                }

                tokio::spawn({
                    let pending_acknowledgments = self.pending_acknowledgments.clone();
                    let chats = self.chats.clone();
                    let msg_id = message_id.clone();
                    let chat_key_clone = chat_key.clone();

                    async move {
                        tokio::time::sleep(Duration::from_secs(30)).await;

                        let mut pending = pending_acknowledgments.write().await;
                        if pending.remove(&msg_id).is_some() {
                            let mut chats_guard = chats.write().await;
                            if let Some(messages) = chats_guard.get_mut(&chat_key_clone) {
                                if let Some(msg) = messages.iter_mut().find(|m| m.id == msg_id) {
                                    msg.delivery_status = DeliveryStatus::Failed;
                                    println!("Message {} marked as failed after timeout", msg_id);
                                }
                            }
                        }
                    }
                });
            }
            Err(e) => {
                chat_message.delivery_status = DeliveryStatus::Failed;

                let error_msg = if e.to_string().contains("Connection refused")
                    || e.to_string().contains("10061")
                {
                    format!(
                        "Failed to connect to {}: Contact unavailable",
                        contact.address
                    )
                } else if e.to_string().contains("timeout") {
                    format!("Connection timeout with {}", contact.address)
                } else {
                    format!("Network error: {}", e)
                };

                self.event_bus
                    .emit_network(crate::events::NetworkEvent::Error {
                        error: error_msg.clone(),
                        context: Some(format!("Send to {}", contact.name)),
                    });

                let mut chats = self.chats.write().await;
                chats.entry(chat_key).or_default().push(chat_message);

                return Err(error_msg.into());
            }
        }

        {
            let mut chats = self.chats.write().await;
            chats.entry(chat_key).or_default().push(chat_message);
        }

        Ok(())
    }

    async fn send_message_with_multiple_attempts(
        address: &str,
        message: &NetworkMessage,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        if let Ok((host, original_port)) = Self::parse_address(address) {
            let ports_to_try = vec![original_port, 443, 80, 8080, 8443, 8000, 9000, 3000];

            for port in ports_to_try {
                let test_address = format!("{}:{}", host, port);
                println!("Trying to connect to: {}", test_address);

                match Self::send_message_to_address_with_acknowledgment(&test_address, message)
                    .await
                {
                    Ok(result) => {
                        println!("Successfully connected to {}", test_address);
                        return Ok(result);
                    }
                    Err(e) => {
                        println!("Failed to connect to {}: {}", test_address, e);
                        continue;
                    }
                }
            }

            Err("All connection attempts failed".into())
        } else {
            Self::send_message_to_address_with_acknowledgment(address, message).await
        }
    }

    fn parse_address(address: &str) -> Result<(String, u16), Box<dyn std::error::Error>> {
        if let Some(colon_pos) = address.rfind(':') {
            let host = &address[..colon_pos];
            let port_str = &address[colon_pos + 1..];
            let port: u16 = port_str.parse()?;
            Ok((host.to_string(), port))
        } else {
            Err("Address must contain port".into())
        }
    }

    async fn send_message_to_address_with_acknowledgment(
        address: &str,
        message: &NetworkMessage,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        let stream_result =
            tokio::time::timeout(Duration::from_secs(10), TcpStream::connect(address)).await;

        let mut stream = match stream_result {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => {
                let error_msg = if e.to_string().contains("10061")
                    || e.to_string().contains("Connection refused")
                {
                    "Connection refused - recipient may not be online".to_string()
                } else {
                    format!("Failed to connect: {}", e)
                };
                return Err(error_msg.into());
            }
            Err(_) => return Err("Connection timeout".into()),
        };

        let data = serde_json::to_vec(message)?;

        match tokio::time::timeout(Duration::from_secs(5), stream.write_all(&data)).await {
            Ok(Ok(_)) => {
                let _ = stream.flush().await;

                if message.message_id.is_some() {
                    let mut buffer = [0; 1024];
                    match tokio::time::timeout(Duration::from_secs(10), stream.read(&mut buffer))
                        .await
                    {
                        Ok(Ok(n)) if n > 0 => {
                            if let Ok(acknowledgment_msg) =
                                serde_json::from_slice::<NetworkMessage>(&buffer[..n])
                            {
                                if acknowledgment_msg.message_type
                                    == MessageType::MessageAcknowledgment
                                {
                                    return Ok(true);
                                }
                            }
                        }
                        _ => {}
                    }
                    return Ok(false);
                }

                Ok(true)
            }
            Ok(Err(e)) => Err(format!("Data send error: {}", e).into()),
            Err(_) => Err("Write timeout".into()),
        }
    }

    pub async fn check_contact_online(&self, contact: &Contact) -> bool {
        let ping_message = NetworkMessage {
            message_type: MessageType::Ping,
            sender_id: "ping".to_string(),
            recipient_id: contact.id.clone(),
            content: vec![],
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            message_id: None,
        };

        match Self::send_message_with_multiple_attempts(&contact.address, &ping_message).await {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub async fn get_chat_messages(&self, contact_name: &str) -> Vec<ChatMessage> {
        let chat_key = self.create_chat_key(contact_name).await;
        let chats = self.chats.read().await;
        chats.get(&chat_key).cloned().unwrap_or_default()
    }

    pub async fn block_peer(&self, peer_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut blocked = self.blocked_peers.write().await;
        blocked.insert(peer_id.to_string(), true);
        Ok(())
    }

    pub async fn unblock_peer(&self, peer_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut blocked = self.blocked_peers.write().await;
        blocked.remove(peer_id);
        Ok(())
    }

    pub async fn get_network_stats(&self) -> NetworkStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    pub async fn check_contact_availability(&self, contact: &Contact) -> bool {
        self.check_contact_online(contact).await
    }

    pub async fn restart_server(
        &self,
        port: u16,
        contacts: Arc<RwLock<HashMap<String, Contact>>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if self.is_running() {
            self.shutdown().await?;
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        self.start_server(port, contacts).await
    }
}
