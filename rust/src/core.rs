use crate::config::{AppConfig, ConfigManager};
use crate::contact_manager::{ContactError, ContactManager};
use crate::events::{EventBus, NetworkEvent};
use crate::network::{ChatMessage, Contact, NetworkManager};
use crate::peer::Peer;
use crate::storage::StorageManager;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use tokio::sync::Mutex;

static PORT_LOCK: OnceLock<Arc<Mutex<()>>> = OnceLock::new();

fn get_app_data_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let data_dir = if cfg!(target_os = "windows") {
        std::env::var("APPDATA")
            .map(PathBuf::from)
            .or_else(|_| {
                std::env::var("USERPROFILE")
                    .map(|p| PathBuf::from(p).join("AppData").join("Roaming"))
            })
            .unwrap_or_else(|_| PathBuf::from("C:\\Users\\Default\\AppData\\Roaming"))
    } else if cfg!(target_os = "macos") {
        std::env::var("HOME")
            .map(|p| PathBuf::from(p).join("Library").join("Application Support"))
            .unwrap_or_else(|_| PathBuf::from("/Users/Shared/Library/Application Support"))
    } else {
        std::env::var("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|_| {
                std::env::var("HOME").map(|p| PathBuf::from(p).join(".local").join("share"))
            })
            .unwrap_or_else(|_| PathBuf::from("/tmp"))
    };

    let app_data_dir = data_dir.join("ShadowGhost");

    if !app_data_dir.exists() {
        std::fs::create_dir_all(&app_data_dir)?;
    }

    Ok(app_data_dir)
}

#[derive(Debug)]
pub enum CoreError {
    Network(String),
    Storage(String),
    Crypto(String),
    Config(String),
    Contact(String),
    InvalidState(String),
}

impl std::fmt::Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreError::Network(msg) => write!(f, "Сетевая ошибка: {}", msg),
            CoreError::Storage(msg) => write!(f, "Ошибка хранения: {}", msg),
            CoreError::Crypto(msg) => write!(f, "Ошибка шифрования: {}", msg),
            CoreError::Config(msg) => write!(f, "Ошибка конфигурации: {}", msg),
            CoreError::Contact(msg) => write!(f, "Ошибка контакта: {}", msg),
            CoreError::InvalidState(msg) => write!(f, "Некорректное состояние: {}", msg),
        }
    }
}

impl std::error::Error for CoreError {}

impl From<ContactError> for CoreError {
    fn from(error: ContactError) -> Self {
        CoreError::Contact(error.to_string())
    }
}

pub struct ShadowGhostCore {
    config_manager: ConfigManager,
    network_manager: Option<NetworkManager>,
    contact_manager: Option<ContactManager>,
    storage_manager: StorageManager,
    event_bus: EventBus,
    is_initialized: bool,
    allocated_port: Option<u16>,
    server_started: bool,
}

impl ShadowGhostCore {
    pub fn new(config_path: &PathBuf) -> Result<Self, CoreError> {
        let app_data_dir = get_app_data_dir().map_err(|e| CoreError::Config(e.to_string()))?;

        let actual_config_path = if config_path.exists() {
            config_path.clone()
        } else {
            app_data_dir.join("config.toml")
        };

        let mut config_manager = ConfigManager::new(&actual_config_path)
            .map_err(|e| CoreError::Config(e.to_string()))?;

        config_manager
            .update_config(|config| {
                config.storage.data_dir = app_data_dir.clone();
            })
            .map_err(|e| CoreError::Config(e.to_string()))?;

        let event_bus = EventBus::new();

        let storage_manager =
            StorageManager::new(config_manager.get_config().clone(), event_bus.clone())
                .map_err(|e| CoreError::Storage(e.to_string()))?;

        Ok(Self {
            config_manager,
            network_manager: None,
            contact_manager: None,
            storage_manager,
            event_bus,
            is_initialized: false,
            allocated_port: None,
            server_started: false,
        })
    }

    pub async fn initialize(&mut self, user_name: Option<String>) -> Result<(), CoreError> {
        if self.is_initialized {
            return Err(CoreError::InvalidState("Уже инициализировано".to_string()));
        }

        if let Some(name) = user_name {
            self.config_manager
                .set_user_name(name)
                .map_err(|e| CoreError::Config(e.to_string()))?;
        }

        let config = self.config_manager.get_config();

        let allocated_port = Self::find_and_reserve_port(config.network.default_port)
            .await
            .map_err(|e| CoreError::Network(format!("Не удалось выделить порт: {}", e)))?;

        let address = format!("127.0.0.1:{}", allocated_port);
        let peer = Peer::new_with_entropy(config.user.name.clone(), address);

        println!("🚀 Инициализация с информацией о пользователе:");
        println!("  Имя: {}", peer.name);
        println!("  Адрес: {}", peer.address);
        println!("  ID: {}", peer.get_short_id());

        let network_manager = NetworkManager::new(peer.clone(), self.event_bus.clone())
            .map_err(|e| CoreError::Network(e.to_string()))?;

        let contact_manager =
            ContactManager::new(peer, network_manager.get_crypto(), self.event_bus.clone());

        self.load_saved_data(&contact_manager).await?;

        self.contact_manager = Some(contact_manager);
        self.network_manager = Some(network_manager);
        self.allocated_port = Some(allocated_port);
        self.is_initialized = true;

        Ok(())
    }

    async fn load_saved_data(&self, contact_manager: &ContactManager) -> Result<(), CoreError> {
        if let Ok(contacts) = self.storage_manager.load_contacts().await {
            println!("📂 Загружено {} сохраненных контактов", contacts.len());
            contact_manager
                .load_contacts(contacts)
                .await
                .map_err(|e| CoreError::Contact(e.to_string()))?;
        }

        Ok(())
    }

    pub fn get_event_bus(&self) -> &EventBus {
        &self.event_bus
    }

    pub fn get_config(&self) -> &AppConfig {
        self.config_manager.get_config()
    }

    async fn find_and_reserve_port(start_port: u16) -> Result<u16, std::io::Error> {
        let port_lock = PORT_LOCK.get_or_init(|| Arc::new(Mutex::new(())));
        let _lock = port_lock.lock().await;

        let pid = std::process::id();
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u16;

        let offset = (pid as u16 ^ timestamp) % 100;
        let search_start = start_port.saturating_add(offset);

        for i in 0..200 {
            let port = search_start.wrapping_add(i);
            if port < 1024 {
                continue;
            }

            match TcpListener::bind(("127.0.0.1", port)) {
                Ok(listener) => {
                    drop(listener);
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

                    match TcpListener::bind(("127.0.0.1", port)) {
                        Ok(_) => return Ok(port),
                        Err(_) => continue,
                    }
                }
                Err(_) => continue,
            }
        }

        Err(std::io::Error::new(
            std::io::ErrorKind::AddrInUse,
            "Не найдено доступных портов после обширного поиска",
        ))
    }

    pub async fn start_server(&mut self) -> Result<(), CoreError> {
        self.ensure_initialized()?;

        if self.server_started {
            return Err(CoreError::InvalidState("Сервер уже запущен".to_string()));
        }

        let port = self.allocated_port.unwrap();
        let network_manager = self.network_manager.as_ref().unwrap();
        let contact_manager = self.contact_manager.as_ref().unwrap();
        let nm_clone = network_manager.clone();
        let cm_clone = contact_manager.get_contacts_ref();

        println!("🚀 Запуск сервера на порту {}", port);

        tokio::spawn(async move {
            if let Err(e) = nm_clone.start_server(port, cm_clone).await {
                nm_clone.event_bus.emit_network(NetworkEvent::Error {
                    error: e.to_string(),
                    context: Some("Запуск сервера".to_string()),
                });
            }
        });

        // Даем время серверу запуститься
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

        // Проверяем, что сервер действительно запустился
        if let Some(nm) = &self.network_manager {
            if nm.is_running() {
                self.server_started = true;
                println!("✅ Сервер успешно запущен!");
            } else {
                return Err(CoreError::Network(
                    "Не удалось запустить сервер".to_string(),
                ));
            }
        }

        Ok(())
    }

    pub async fn stop_server(&mut self) -> Result<(), CoreError> {
        self.ensure_initialized()?;

        if !self.server_started {
            return Err(CoreError::InvalidState("Сервер не запущен".to_string()));
        }

        println!("🛑 Остановка сервера...");

        if let Some(nm) = &self.network_manager {
            nm.shutdown()
                .await
                .map_err(|e| CoreError::Network(e.to_string()))?;
        }

        self.server_started = false;
        println!("✅ Сервер остановлен");
        Ok(())
    }

    pub async fn restart_server(&mut self) -> Result<(), CoreError> {
        println!("🔄 Перезапуск сервера...");

        if self.server_started {
            self.stop_server().await?;
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        self.start_server().await
    }

    pub fn is_server_started(&self) -> bool {
        self.server_started
            && self
                .network_manager
                .as_ref()
                .map(|nm| nm.is_running())
                .unwrap_or(false)
    }

    pub async fn generate_sg_link(&self) -> Result<String, CoreError> {
        self.ensure_initialized()?;
        self.contact_manager
            .as_ref()
            .unwrap()
            .generate_sg_link()
            .await
            .map_err(|e| e.into())
    }

    pub async fn add_contact_by_sg_link(&self, sg_link: &str) -> Result<(), CoreError> {
        self.ensure_initialized()?;

        let result = self
            .contact_manager
            .as_ref()
            .unwrap()
            .add_contact_by_sg_link(sg_link)
            .await;

        if result.is_ok() {
            self.save_contacts().await?;
            println!("💾 Контакты сохранены");
        }

        result.map(|_| ()).map_err(|e| e.into())
    }

    pub async fn send_message(&self, contact_name: &str, content: &str) -> Result<(), CoreError> {
        self.ensure_initialized()?;

        if !self.server_started {
            return Err(CoreError::InvalidState(
                "Сервер не запущен. Выполните команду 'start' сначала.".to_string(),
            ));
        }

        let contact = self
            .contact_manager
            .as_ref()
            .unwrap()
            .get_contact_by_name(contact_name)
            .await
            .ok_or_else(|| CoreError::Contact(format!("Контакт {} не найден", contact_name)))?;

        let result = self
            .network_manager
            .as_ref()
            .unwrap()
            .send_chat_message(&contact, content)
            .await;

        if result.is_ok() {
            self.save_chat_data(contact_name).await?;
        }

        result.map_err(|e| CoreError::Network(e.to_string()))
    }

    pub async fn get_contacts(&self) -> Result<Vec<Contact>, CoreError> {
        self.ensure_initialized()?;
        Ok(self.contact_manager.as_ref().unwrap().get_contacts().await)
    }

    pub async fn get_chat_messages(
        &self,
        contact_name: &str,
    ) -> Result<Vec<ChatMessage>, CoreError> {
        self.ensure_initialized()?;
        Ok(self
            .network_manager
            .as_ref()
            .unwrap()
            .get_chat_messages(contact_name)
            .await)
    }

    pub async fn get_unread_message_count(&self, contact_name: &str) -> Result<usize, CoreError> {
        self.ensure_initialized()?;

        let messages = self
            .network_manager
            .as_ref()
            .unwrap()
            .get_chat_messages(contact_name)
            .await;

        let my_name = &self.network_manager.as_ref().unwrap().get_peer().name;

        // Считаем сообщения, отправленные контакту (не от нас)
        let received_count = messages
            .iter()
            .filter(|msg| msg.from != *my_name && msg.to == *my_name)
            .count();

        Ok(received_count)
    }

    pub async fn shutdown(&mut self) -> Result<(), CoreError> {
        if !self.is_initialized {
            println!("⚠️ Система уже не инициализирована");
            return Ok(());
        }

        println!("🛑 Завершение работы ShadowGhost...");

        // Сначала останавливаем сервер, если он запущен
        if self.server_started {
            if let Some(nm) = &self.network_manager {
                nm.shutdown()
                    .await
                    .map_err(|e| CoreError::Network(e.to_string()))?;
            }
            self.server_started = false;
        }

        // Сохраняем данные контактов
        if let Some(cm) = &self.contact_manager {
            let contacts = cm.get_contacts_map().await;
            if let Err(e) = self.storage_manager.save_contacts(&contacts).await {
                println!("⚠️ Предупреждение: Не удалось сохранить контакты: {}", e);
            } else {
                println!("💾 Контакты сохранены");
            }
        }

        // Сохраняем историю чатов
        if let Some(nm) = &self.network_manager {
            let chats = nm.get_chats().await;
            let mut saved_chats = 0;
            for (chat_key, messages) in chats.iter() {
                if let Err(e) = self
                    .storage_manager
                    .save_chat_history_with_cleanup(chat_key, messages)
                    .await
                {
                    println!(
                        "⚠️ Предупреждение: Не удалось сохранить чат {}: {}",
                        chat_key, e
                    );
                } else {
                    saved_chats += 1;
                }
            }
            if saved_chats > 0 {
                println!("💾 Сохранено {} чатов", saved_chats);
            }
        }

        // Очищаем ресурсы
        self.network_manager = None;
        self.contact_manager = None;
        self.allocated_port = None;
        self.is_initialized = false;

        println!("✅ ShadowGhost корректно завершен");
        Ok(())
    }

    async fn save_contacts(&self) -> Result<(), CoreError> {
        if let Some(cm) = &self.contact_manager {
            let contacts = cm.get_contacts_map().await;
            self.storage_manager
                .save_contacts(&contacts)
                .await
                .map_err(|e| CoreError::Storage(e.to_string()))?;
        }
        Ok(())
    }

    async fn save_chat_data(&self, contact_name: &str) -> Result<(), CoreError> {
        if let Some(nm) = &self.network_manager {
            let chats = nm.get_chats().await;
            let peer_name = &nm.get_peer().name.as_str();

            let chat_key = if peer_name < &contact_name {
                format!("{}_{}", peer_name, contact_name)
            } else {
                format!("{}_{}", contact_name, peer_name)
            };

            if let Some(messages) = chats.get(&chat_key) {
                self.storage_manager
                    .save_chat_history_with_cleanup(&chat_key, messages)
                    .await
                    .map_err(|e| CoreError::Storage(e.to_string()))?;
            }
        }
        Ok(())
    }

    fn ensure_initialized(&self) -> Result<(), CoreError> {
        if !self.is_initialized {
            return Err(CoreError::InvalidState(
                "Система не инициализирована".to_string(),
            ));
        }
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    pub fn get_peer_info(&self) -> Option<String> {
        if let Some(nm) = &self.network_manager {
            let peer = nm.get_peer();
            Some(format!("{} ({})", peer.name, peer.address))
        } else {
            None
        }
    }

    // Дополнительные методы для мониторинга состояния
    pub async fn get_server_status(&self) -> String {
        if !self.is_initialized {
            return "Не инициализировано".to_string();
        }

        if let Some(nm) = &self.network_manager {
            if nm.is_running() {
                format!("Запущен на порту {}", self.allocated_port.unwrap_or(0))
            } else if self.server_started {
                "Запускается...".to_string()
            } else {
                "Остановлен".to_string()
            }
        } else {
            "Ошибка менеджера сети".to_string()
        }
    }

    pub async fn get_network_stats(&self) -> Result<crate::network::NetworkStats, CoreError> {
        self.ensure_initialized()?;

        if let Some(nm) = &self.network_manager {
            Ok(nm.get_network_stats().await)
        } else {
            Err(CoreError::InvalidState(
                "Сетевой менеджер недоступен".to_string(),
            ))
        }
    }
}
