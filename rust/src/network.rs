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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChatMessageType {
    Text,
    File,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContactStatus {
    Online,
    Offline,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryStatus {
    Sent,
    Delivered,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
}

#[derive(Clone)]
pub struct NetworkManager {
    peer: Peer,
    chats: Arc<RwLock<HashMap<String, Vec<ChatMessage>>>>,
    crypto: Arc<RwLock<CryptoManager>>,
    pub event_bus: EventBus,
    stats: Arc<RwLock<NetworkStats>>,
    blocked_peers: Arc<RwLock<HashMap<String, bool>>>,
    // Добавляем поля для управления shutdown
    shutdown_signal: Arc<Notify>,
    is_running: Arc<AtomicBool>,
    server_handle: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl NetworkManager {
    pub fn new(peer: Peer, event_bus: EventBus) -> Result<Self, Box<dyn std::error::Error>> {
        let crypto = CryptoManager::new()?;

        Ok(Self {
            peer,
            chats: Arc::new(RwLock::new(HashMap::new())),
            crypto: Arc::new(RwLock::new(crypto)),
            event_bus,
            stats: Arc::new(RwLock::new(NetworkStats::default())),
            blocked_peers: Arc::new(RwLock::new(HashMap::new())),
            shutdown_signal: Arc::new(Notify::new()),
            is_running: Arc::new(AtomicBool::new(false)),
            server_handle: Arc::new(RwLock::new(None)),
        })
    }

    pub fn get_peer(&self) -> &Peer {
        &self.peer
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

    fn create_chat_key(&self, contact_name: &str) -> String {
        if self.peer.name.as_str() < contact_name {
            format!("{}_{}", self.peer.name, contact_name)
        } else {
            format!("{}_{}", contact_name, self.peer.name)
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

        // Клонируем необходимые данные для background task
        let manager = self.clone();
        let contacts_ref = contacts.clone();
        let shutdown_signal = self.shutdown_signal.clone();
        let is_running = self.is_running.clone();

        // Создаем server task
        let handle = tokio::spawn(async move {
            println!("🚀 Сервер запущен на порту {}", port);

            loop {
                tokio::select! {
                    // Ожидаем новые соединения
                    accept_result = listener.accept() => {
                        match accept_result {
                            Ok((stream, addr)) => {
                                println!("📞 Новое соединение от {}", addr);

                                let manager_clone = manager.clone();
                                let contacts_clone = contacts_ref.clone();

                                // Обрабатываем соединение в отдельном task
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
                                eprintln!("❌ Ошибка принятия соединения: {}", e);
                                break;
                            }
                        }
                    }
                    // Ожидаем сигнал shutdown
                    _ = shutdown_signal.notified() => {
                        println!("🛑 Получен сигнал остановки сервера");
                        break;
                    }
                }
            }

            is_running.store(false, Ordering::Relaxed);
            println!("✅ Сервер остановлен");
        });

        // Сохраняем handle для возможности отмены
        {
            let mut server_handle = self.server_handle.write().await;
            *server_handle = Some(handle);
        }

        Ok(())
    }

    // Новый метод shutdown
    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        if !self.is_running.load(Ordering::Relaxed) {
            println!("⚠️ Сервер уже остановлен");
            return Ok(());
        }

        println!("🛑 Останавливаем сервер...");

        // Отправляем сигнал остановки
        self.shutdown_signal.notify_one();

        // Ждем завершения server task
        let handle = {
            let mut server_handle = self.server_handle.write().await;
            server_handle.take()
        };

        if let Some(handle) = handle {
            if let Err(e) = handle.await {
                eprintln!("❌ Ошибка при остановке сервера: {}", e);
            }
        }

        self.is_running.store(false, Ordering::Relaxed);

        self.event_bus
            .emit_network(crate::events::NetworkEvent::Error {
                error: "Server shutdown completed".to_string(),
                context: Some("Shutdown".to_string()),
            });

        println!("✅ Сервер успешно остановлен");
        Ok(())
    }

    async fn handle_connection(
        &self,
        mut stream: TcpStream,
        contacts: Arc<RwLock<HashMap<String, Contact>>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use tokio::io::AsyncReadExt;

        let mut buffer = [0; 4096];

        match tokio::time::timeout(Duration::from_secs(10), stream.read(&mut buffer)).await {
            Ok(Ok(n)) if n > 0 => {
                if let Ok(message) = serde_json::from_slice::<NetworkMessage>(&buffer[..n]) {
                    if self.is_peer_blocked(&message.sender_id).await {
                        println!(
                            "🚫 Сообщение от заблокированного пользователя: {}",
                            message.sender_id
                        );
                        return Ok(());
                    }

                    Self::process_message(
                        &message,
                        &self.peer,
                        contacts,
                        self.chats.clone(),
                        self.crypto.clone(),
                        self.event_bus.clone(),
                    )
                    .await?;

                    // Обновляем статистику
                    let mut stats = self.stats.write().await;
                    stats.messages_received += 1;
                    stats.bytes_received += n as u64;
                }
            }
            Ok(Ok(_)) => {
                println!("⚠️ Получено пустое сообщение");
            }
            Ok(Err(e)) => {
                println!("❌ Ошибка чтения: {}", e);
            }
            Err(_) => {
                println!("⏰ Таймаут при чтении сообщения");
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
                    id: uuid::Uuid::new_v4().to_string(),
                    from: sender_name.clone(),
                    to: peer.name.clone(),
                    content: content.to_string(),
                    msg_type: ChatMessageType::Text,
                    timestamp: message.timestamp,
                    delivery_status: DeliveryStatus::Delivered,
                };

                // Создаем консистентный ключ чата
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

                println!("💬 Получено сообщение от {}: {}", sender_name, content);

                event_bus.emit_network(crate::events::NetworkEvent::MessageReceived {
                    message: chat_message,
                });
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

                println!("🤝 Добавлен новый контакт: {}", contact_data.name);

                event_bus.emit_network(crate::events::NetworkEvent::ContactAdded { contact });
            }
            _ => {
                println!(
                    "❓ Получен неизвестный тип сообщения: {:?}",
                    message.message_type
                );
            }
        }

        Ok(())
    }

    pub async fn send_chat_message(
        &self,
        contact: &Contact,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let message = NetworkMessage {
            message_type: MessageType::TextMessage,
            sender_id: self.peer.id.clone(),
            recipient_id: contact.id.clone(),
            content: content.as_bytes().to_vec(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
        };

        let mut chat_message = ChatMessage {
            id: uuid::Uuid::new_v4().to_string(),
            from: self.peer.name.clone(),
            to: contact.name.clone(),
            content: content.to_string(),
            msg_type: ChatMessageType::Text,
            timestamp: message.timestamp,
            delivery_status: DeliveryStatus::Sent,
        };

        let chat_key = self.create_chat_key(&contact.name);

        println!("📤 Отправка сообщения для {}: {}", contact.name, content);

        match Self::send_message_to_address(&contact.address, &message).await {
            Ok(_) => {
                chat_message.delivery_status = DeliveryStatus::Delivered;
                println!("✅ Сообщение доставлено для {}", contact.name);

                let mut stats = self.stats.write().await;
                stats.messages_sent += 1;
                stats.bytes_sent += content.len() as u64;
            }
            Err(e) => {
                chat_message.delivery_status = DeliveryStatus::Failed;
                println!("❌ Ошибка отправки для {}: {}", contact.name, e);

                let error_msg = if e.to_string().contains("Connection refused")
                    || e.to_string().contains("10061")
                {
                    format!(
                        "Не удалось подключиться к {}: Контакт недоступен",
                        contact.address
                    )
                } else if e.to_string().contains("timeout") {
                    format!("Таймаут соединения с {}", contact.address)
                } else {
                    format!("Сетевая ошибка: {}", e)
                };

                self.event_bus
                    .emit_network(crate::events::NetworkEvent::Error {
                        error: error_msg.clone(),
                        context: Some(format!("Отправка для {}", contact.name)),
                    });

                // Сохраняем неудачное сообщение в историю
                let mut chats = self.chats.write().await;
                chats.entry(chat_key).or_default().push(chat_message);

                return Err(error_msg.into());
            }
        }

        // Сохраняем успешное сообщение в историю
        {
            let mut chats = self.chats.write().await;
            chats.entry(chat_key).or_default().push(chat_message);
        }

        Ok(())
    }

    async fn send_message_to_address(
        address: &str,
        message: &NetworkMessage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        use tokio::io::AsyncWriteExt;

        let stream_result =
            tokio::time::timeout(Duration::from_secs(5), TcpStream::connect(address)).await;

        let mut stream = match stream_result {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => {
                let error_msg = if e.to_string().contains("10061")
                    || e.to_string().contains("Connection refused")
                {
                    "Соединение отклонено - получатель может быть не в сети".to_string()
                } else {
                    format!("Не удалось подключиться: {}", e)
                };
                return Err(error_msg.into());
            }
            Err(_) => return Err("Таймаут соединения".into()),
        };

        let data = serde_json::to_vec(message)?;

        match tokio::time::timeout(Duration::from_secs(3), stream.write_all(&data)).await {
            Ok(Ok(_)) => {
                // Принудительная отправка данных
                let _ = stream.flush().await;
                Ok(())
            }
            Ok(Err(e)) => Err(format!("Ошибка отправки данных: {}", e).into()),
            Err(_) => Err("Таймаут записи".into()),
        }
    }

    pub async fn get_chat_messages(&self, contact_name: &str) -> Vec<ChatMessage> {
        let chat_key = self.create_chat_key(contact_name);
        let chats = self.chats.read().await;
        chats.get(&chat_key).cloned().unwrap_or_default()
    }

    pub async fn block_peer(&self, peer_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut blocked = self.blocked_peers.write().await;
        blocked.insert(peer_id.to_string(), true);
        println!("🚫 Пользователь {} заблокирован", peer_id);
        Ok(())
    }

    pub async fn unblock_peer(&self, peer_id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut blocked = self.blocked_peers.write().await;
        blocked.remove(peer_id);
        println!("✅ Пользователь {} разблокирован", peer_id);
        Ok(())
    }

    pub async fn get_network_stats(&self) -> NetworkStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    pub async fn check_contact_availability(&self, contact: &Contact) -> bool {
        let stream_result =
            tokio::time::timeout(Duration::from_secs(2), TcpStream::connect(&contact.address))
                .await;

        matches!(stream_result, Ok(Ok(_)))
    }

    // Дополнительный метод для graceful restart
    pub async fn restart_server(
        &self,
        port: u16,
        contacts: Arc<RwLock<HashMap<String, Contact>>>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔄 Перезапуск сервера...");

        if self.is_running() {
            self.shutdown().await?;
            // Даем время на корректное завершение
            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        self.start_server(port, contacts).await
    }
}
