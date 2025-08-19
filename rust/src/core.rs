use crate::config::{AppConfig, ConfigManager};
use crate::contact_manager::{ContactError, ContactManager};
use crate::events::{EventBus, NetworkEvent};
use crate::network::{ChatMessage, Contact, NetworkManager};
use crate::network_discovery::NetworkDiscovery;
use crate::peer::Peer;
use crate::storage::StorageManager;
use std::net::TcpListener;
use std::path::PathBuf;
use std::sync::{Arc, OnceLock};
use tokio::sync::{Mutex, RwLock};

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
    Validation(String),
}

impl std::fmt::Display for CoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreError::Network(msg) => write!(f, "Network error: {}", msg),
            CoreError::Storage(msg) => write!(f, "Storage error: {}", msg),
            CoreError::Crypto(msg) => write!(f, "Crypto error: {}", msg),
            CoreError::Config(msg) => write!(f, "Config error: {}", msg),
            CoreError::Contact(msg) => write!(f, "Contact error: {}", msg),
            CoreError::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
            CoreError::Validation(msg) => write!(f, "Validation error: {}", msg),
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
    external_ip: Arc<RwLock<Option<std::net::IpAddr>>>,
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
            external_ip: Arc::new(RwLock::new(None)),
        })
    }

    pub fn new_for_test(test_id: &str) -> Result<Self, CoreError> {
        let temp_dir = std::env::temp_dir().join("shadowghost_test").join(test_id);
        std::fs::create_dir_all(&temp_dir).map_err(|e| CoreError::Config(e.to_string()))?;

        let config_path = temp_dir.join("config.toml");
        let mut config_manager =
            ConfigManager::new(&config_path).map_err(|e| CoreError::Config(e.to_string()))?;

        config_manager
            .update_config(|config| {
                config.storage.data_dir = temp_dir.clone();
            })
            .map_err(|e| CoreError::Config(e.to_string()))?;

        config_manager
            .enable_test_mode()
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
            external_ip: Arc::new(RwLock::new(None)),
        })
    }

    pub async fn get_stored_user_name(&self) -> Option<String> {
        let config = self.config_manager.get_config();
        if config.user.name != "user" {
            Some(config.user.name.clone())
        } else {
            None
        }
    }

    async fn detect_external_ip(&self) -> Result<(), CoreError> {
        let config = self.config_manager.get_config();

        if config.network.test_mode || !config.network.auto_detect_external_ip {
            let mut external_ip = self.external_ip.write().await;
            *external_ip = None;
            if config.network.test_mode {
                println!("Test mode: Using localhost only");
            }
            return Ok(());
        }

        match NetworkDiscovery::get_external_ip().await {
            Ok(ip) => {
                let mut external_ip = self.external_ip.write().await;
                *external_ip = Some(ip);
                println!("Detected external IP: {}", ip);
            }
            Err(e) => {
                println!("Could not detect external IP: {}", e);
                println!("Using local address for connections");
                let mut external_ip = self.external_ip.write().await;
                *external_ip = None;
            }
        }
        Ok(())
    }

    async fn get_or_allocate_port(&self) -> Result<u16, CoreError> {
        let config = self.config_manager.get_config();

        if config.network.test_mode {
            let test_port = Self::find_and_reserve_port(0)
                .await
                .map_err(|e| CoreError::Network(format!("Failed to allocate test port: {}", e)))?;
            println!("Test mode: Using random port: {}", test_port);
            return Ok(test_port);
        }

        if config.network.use_fixed_port {
            let desired_port = config.network.default_port;
            if self.is_port_available(desired_port).await {
                println!("Using fixed port: {}", desired_port);
                self.save_port(desired_port).await?;
                return Ok(desired_port);
            } else {
                println!(
                    "Fixed port {} not available, trying alternatives",
                    desired_port
                );

                for offset in 1..=10 {
                    let alt_port = desired_port + offset;
                    if self.is_port_available(alt_port).await {
                        println!("Using alternative port: {}", alt_port);
                        self.save_port(alt_port).await?;
                        return Ok(alt_port);
                    }
                }
                return Err(CoreError::Network(format!(
                    "No available ports near {}",
                    desired_port
                )));
            }
        }

        if let Some(saved_port) = self.get_saved_port().await {
            if self.is_port_available(saved_port).await {
                println!("Using saved port: {}", saved_port);
                return Ok(saved_port);
            } else {
                println!(
                    "Saved port {} not available, allocating new one",
                    saved_port
                );
            }
        }

        let new_port = Self::find_and_reserve_port(config.network.default_port)
            .await
            .map_err(|e| CoreError::Network(format!("Failed to allocate port: {}", e)))?;

        self.save_port(new_port).await?;
        println!("Allocated new port: {}", new_port);
        Ok(new_port)
    }

    async fn get_saved_port(&self) -> Option<u16> {
        let config = self.config_manager.get_config();
        if config.network.test_mode {
            return None;
        }

        let port_file = self.get_app_data_dir().ok()?.join("saved_port.txt");
        if let Ok(content) = tokio::fs::read_to_string(&port_file).await {
            content.trim().parse().ok()
        } else {
            None
        }
    }

    async fn save_port(&self, port: u16) -> Result<(), CoreError> {
        let config = self.config_manager.get_config();
        if config.network.test_mode {
            return Ok(());
        }

        let port_file = self
            .get_app_data_dir()
            .map_err(|e| CoreError::Config(e.to_string()))?
            .join("saved_port.txt");

        tokio::fs::write(&port_file, port.to_string())
            .await
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        Ok(())
    }

    async fn is_port_available(&self, port: u16) -> bool {
        match TcpListener::bind(("127.0.0.1", port)) {
            Ok(listener) => {
                drop(listener);
                true
            }
            Err(_) => false,
        }
    }

    fn get_app_data_dir(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        get_app_data_dir()
    }

    async fn validate_stored_data(&self) -> Result<(), CoreError> {
        match self.storage_manager.validate_contacts().await {
            Ok(issues) => {
                if !issues.is_empty() {
                    println!("Data validation issues found:");
                    for issue in issues {
                        println!("  - {}", issue);
                    }
                }
            }
            Err(e) => {
                return Err(CoreError::Validation(format!(
                    "Failed to validate contacts: {}",
                    e
                )));
            }
        }

        match self.storage_manager.validate_chats().await {
            Ok(issues) => {
                if !issues.is_empty() {
                    println!("Chat history validation issues:");
                    for issue in issues {
                        println!("  - {}", issue);
                    }
                }
            }
            Err(e) => {
                return Err(CoreError::Validation(format!(
                    "Failed to validate chats: {}",
                    e
                )));
            }
        }

        Ok(())
    }

    pub async fn initialize(&mut self, user_name: Option<String>) -> Result<(), CoreError> {
        if self.is_initialized {
            return Err(CoreError::InvalidState("Already initialized".to_string()));
        }

        if let Some(name) = user_name {
            self.config_manager
                .set_user_name(name)
                .map_err(|e| CoreError::Config(e.to_string()))?;
        }

        self.detect_external_ip().await?;
        self.validate_stored_data().await?;

        let config = self.config_manager.get_config();

        let allocated_port = self.get_or_allocate_port().await?;

        let public_address = {
            let external_ip = self.external_ip.read().await;
            if let Some(ip) = *external_ip {
                format!("{}:{}", ip, allocated_port)
            } else {
                format!("127.0.0.1:{}", allocated_port)
            }
        };

        let local_address = format!("127.0.0.1:{}", allocated_port);

        let peer = Peer::new_with_entropy(config.user.name.clone(), public_address.clone());

        println!("Initializing with user info:");
        println!("  Name: {}", peer.name);
        println!("  Public Address: {}", public_address);
        println!("  Local Address: {}", local_address);
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
            println!("Loaded {} saved contacts", contacts.len());
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

    pub fn get_config(&self) -> AppConfig {
        self.config_manager.get_config()
    }

    async fn find_and_reserve_port(start_port: u16) -> Result<u16, std::io::Error> {
        let port_lock = PORT_LOCK.get_or_init(|| Arc::new(Mutex::new(())));
        let _lock = port_lock.lock().await;

        if start_port == 0 {
            use rand::Rng;
            let mut rng = rand::rng();
            for _ in 0..100 {
                let random_port = rng.random_range(49152..65535);
                match TcpListener::bind(("127.0.0.1", random_port)) {
                    Ok(listener) => {
                        drop(listener);
                        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

                        match TcpListener::bind(("127.0.0.1", random_port)) {
                            Ok(_) => return Ok(random_port),
                            Err(_) => continue,
                        }
                    }
                    Err(_) => continue,
                }
            }
            return Err(std::io::Error::new(
                std::io::ErrorKind::AddrInUse,
                "No available random ports found",
            ));
        }

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
            "No available ports found after extensive search",
        ))
    }

    pub async fn start_server(&mut self) -> Result<(), CoreError> {
        self.ensure_initialized()?;

        if self.server_started {
            return Err(CoreError::InvalidState(
                "Server already running".to_string(),
            ));
        }

        let port = self.allocated_port.unwrap();
        let network_manager = self.network_manager.as_ref().unwrap();
        let contact_manager = self.contact_manager.as_ref().unwrap();
        let nm_clone = network_manager.clone();
        let cm_clone = contact_manager.get_contacts_ref();

        println!("Starting server on all interfaces (0.0.0.0:{})", port);
        let external_ip = self.external_ip.read().await;
        if let Some(ip) = *external_ip {
            println!("External connections available at: {}:{}", ip, port);
        }

        tokio::spawn(async move {
            if let Err(e) = nm_clone.start_server(port, cm_clone).await {
                nm_clone.event_bus.emit_network(NetworkEvent::Error {
                    error: e.to_string(),
                    context: Some("Server start".to_string()),
                });
            }
        });

        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;

        if let Some(nm) = &self.network_manager {
            if nm.is_running() {
                self.server_started = true;
                println!("Server started successfully!");
            } else {
                return Err(CoreError::Network("Failed to start server".to_string()));
            }
        }

        Ok(())
    }

    pub async fn stop_server(&mut self) -> Result<(), CoreError> {
        self.ensure_initialized()?;

        if !self.server_started {
            return Err(CoreError::InvalidState("Server not running".to_string()));
        }

        println!("Stopping server...");

        if let Some(nm) = &self.network_manager {
            nm.shutdown()
                .await
                .map_err(|e| CoreError::Network(e.to_string()))?;
        }

        self.server_started = false;
        println!("Server stopped");
        Ok(())
    }

    pub async fn restart_server(&mut self) -> Result<(), CoreError> {
        println!("Restarting server...");

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
            println!("Contacts saved");
        }

        result.map(|_| ()).map_err(|e| e.into())
    }

    pub async fn send_message(&self, contact_name: &str, content: &str) -> Result<(), CoreError> {
        self.ensure_initialized()?;

        if !self.server_started {
            return Err(CoreError::InvalidState(
                "Server not running. Execute 'start' command first.".to_string(),
            ));
        }

        let contact = self
            .contact_manager
            .as_ref()
            .unwrap()
            .get_contact_by_name(contact_name)
            .await
            .ok_or_else(|| CoreError::Contact(format!("Contact {} not found", contact_name)))?;

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

        let my_name = &self.network_manager.as_ref().unwrap().get_peer().await.name;

        let received_count = messages
            .iter()
            .filter(|msg| msg.from != *my_name && msg.to == *my_name)
            .count();

        Ok(received_count)
    }

    pub async fn shutdown(&mut self) -> Result<(), CoreError> {
        if !self.is_initialized {
            println!("System not initialized");
            return Ok(());
        }

        println!("Shutting down ShadowGhost...");

        if self.server_started {
            if let Some(nm) = &self.network_manager {
                nm.shutdown()
                    .await
                    .map_err(|e| CoreError::Network(e.to_string()))?;
            }
            self.server_started = false;
        }

        if let Some(cm) = &self.contact_manager {
            let contacts = cm.get_contacts_map().await;
            if let Err(e) = self.storage_manager.save_contacts(&contacts).await {
                println!("Warning: Failed to save contacts: {}", e);
            } else {
                println!("Contacts saved");
            }
        }

        if let Some(nm) = &self.network_manager {
            let chats = nm.get_chats().await;
            let mut saved_chats = 0;
            for (chat_key, messages) in chats.iter() {
                if let Err(e) = self
                    .storage_manager
                    .save_chat_history_with_cleanup(chat_key, messages)
                    .await
                {
                    println!("Warning: Failed to save chat {}: {}", chat_key, e);
                } else {
                    saved_chats += 1;
                }
            }
            if saved_chats > 0 {
                println!("Saved {} chats", saved_chats);
            }
        }

        self.network_manager = None;
        self.contact_manager = None;
        self.allocated_port = None;
        self.is_initialized = false;

        println!("ShadowGhost shutdown complete");
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
            let peer = nm.get_peer().await;
            let peer_name = peer.name.as_str();

            let chat_key = if peer_name < contact_name {
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
                "System not initialized".to_string(),
            ));
        }
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    pub async fn get_peer_info(&self) -> Option<String> {
        if let Some(nm) = &self.network_manager {
            let peer = nm.get_peer().await;
            Some(format!("{} ({})", peer.name, peer.address))
        } else {
            None
        }
    }

    pub async fn get_server_status(&self) -> String {
        if !self.is_initialized {
            return "Not initialized".to_string();
        }

        if let Some(nm) = &self.network_manager {
            if nm.is_running() {
                format!("Running on port {}", self.allocated_port.unwrap_or(0))
            } else if self.server_started {
                "Starting...".to_string()
            } else {
                "Stopped".to_string()
            }
        } else {
            "Network manager error".to_string()
        }
    }

    pub async fn get_network_stats(&self) -> Result<crate::network::NetworkStats, CoreError> {
        self.ensure_initialized()?;

        if let Some(nm) = &self.network_manager {
            Ok(nm.get_network_stats().await)
        } else {
            Err(CoreError::InvalidState(
                "Network manager unavailable".to_string(),
            ))
        }
    }

    pub async fn update_external_address(&self) -> Result<(), CoreError> {
        if !self.is_initialized {
            return Err(CoreError::InvalidState(
                "System not initialized".to_string(),
            ));
        }

        let old_external_ip = {
            let external_ip = self.external_ip.read().await;
            *external_ip
        };

        self.detect_external_ip().await?;

        let new_external_ip = {
            let external_ip = self.external_ip.read().await;
            *external_ip
        };

        if old_external_ip != new_external_ip {
            if let Some(ip) = new_external_ip {
                let port = self.allocated_port.unwrap();
                let new_address = format!("{}:{}", ip, port);

                if let Some(nm) = &self.network_manager {
                    nm.update_peer_address(new_address.clone()).await;
                }

                println!("Updated external address to: {}", new_address);
            } else {
                println!("External IP no longer available, using local address");
            }
        } else {
            println!("External IP unchanged");
        }

        Ok(())
    }

    pub async fn get_connection_info(&self) -> Result<String, CoreError> {
        self.ensure_initialized()?;

        if let Some(_nm) = &self.network_manager {
            let port = self.allocated_port.unwrap();
            let external_ip = self.external_ip.read().await;

            let info = if let Some(ip) = *external_ip {
                format!("External: {}:{}\nLocal: 127.0.0.1:{}", ip, port, port)
            } else {
                format!("Local only: 127.0.0.1:{}", port)
            };

            Ok(info)
        } else {
            Err(CoreError::InvalidState(
                "Network manager not available".to_string(),
            ))
        }
    }

    pub async fn check_contact_online(&self, contact_name: &str) -> bool {
        if !self.is_initialized {
            return false;
        }

        if let Some(cm) = &self.contact_manager {
            cm.check_contact_online(contact_name).await
        } else {
            false
        }
    }

    pub async fn update_user_name(&mut self, new_name: String) -> Result<(), CoreError> {
        self.config_manager
            .set_user_name(new_name.clone())
            .map_err(|e| CoreError::Config(e.to_string()))?;

        if self.is_initialized {
            if let Some(nm) = &self.network_manager {
                nm.update_peer_name(new_name).await;
            }
        }

        Ok(())
    }
}
