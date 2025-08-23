pub mod config;
pub mod engine;
pub mod peer;

pub use config::*;
pub use engine::*;
pub use peer::*;

use crate::contacts::ContactManager;
use crate::crypto::CryptoManager;
use crate::data::StorageManager;
use crate::events::EventBus;
use crate::network::NetworkManager;
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug)]
pub enum CoreError {
    InvalidState(String),
    Network(String),
    Storage(String),
    Contact(String),
    Config(String),
    Crypto(String),
    Initialization(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
            CoreError::Network(msg) => write!(f, "Network error: {}", msg),
            CoreError::Storage(msg) => write!(f, "Storage error: {}", msg),
            CoreError::Contact(msg) => write!(f, "Contact error: {}", msg),
            CoreError::Config(msg) => write!(f, "Config error: {}", msg),
            CoreError::Crypto(msg) => write!(f, "Crypto error: {}", msg),
            CoreError::Initialization(msg) => write!(f, "Initialization error: {}", msg),
        }
    }
}

impl Error for CoreError {}

pub struct ShadowGhostCore {
    config_manager: ConfigManager,
    network_manager: Arc<RwLock<NetworkManager>>,
    storage_manager: Arc<RwLock<StorageManager>>,
    contact_manager: Arc<RwLock<ContactManager>>,
    crypto_manager: Arc<RwLock<CryptoManager>>,
    event_bus: EventBus,
    is_initialized: bool,
    user_name: Option<String>,
}

impl ShadowGhostCore {
    pub fn new() -> Result<Self, CoreError> {
        let config_manager =
            ConfigManager::new("./config.toml").map_err(|e| CoreError::Config(e.to_string()))?;

        let event_bus = EventBus::new();

        let peer = Peer::new("default_user".to_string(), "127.0.0.1:8080".to_string());

        let crypto_manager = Arc::new(RwLock::new(
            CryptoManager::new().map_err(|e| CoreError::Crypto(e.to_string()))?,
        ));

        let network_manager = Arc::new(RwLock::new(
            NetworkManager::new(peer.clone(), event_bus.clone())
                .map_err(|e| CoreError::Network(e.to_string()))?,
        ));

        let storage_manager = Arc::new(RwLock::new(
            StorageManager::new(config_manager.get_config().clone(), event_bus.clone())
                .map_err(|e| CoreError::Storage(e.to_string()))?,
        ));

        let contact_manager = Arc::new(RwLock::new(ContactManager::new(peer)));

        Ok(Self {
            config_manager,
            network_manager,
            storage_manager,
            contact_manager,
            crypto_manager,
            event_bus,
            is_initialized: false,
            user_name: None,
        })
    }

    pub fn new_for_test(test_id: &str) -> Result<Self, CoreError> {
        let temp_dir = std::env::temp_dir().join("shadowghost_test").join(test_id);
        std::fs::create_dir_all(&temp_dir).map_err(|e| CoreError::Config(e.to_string()))?;

        let config_path = temp_dir.join("config.toml");
        let config_manager =
            ConfigManager::new(config_path).map_err(|e| CoreError::Config(e.to_string()))?;

        let event_bus = EventBus::new();

        let peer = Peer::new("test_user".to_string(), "127.0.0.1:8080".to_string());

        let crypto_manager = Arc::new(RwLock::new(
            CryptoManager::new().map_err(|e| CoreError::Crypto(e.to_string()))?,
        ));

        let network_manager = Arc::new(RwLock::new(
            NetworkManager::new(peer.clone(), event_bus.clone())
                .map_err(|e| CoreError::Network(e.to_string()))?,
        ));

        let storage_manager = Arc::new(RwLock::new(
            StorageManager::new(config_manager.get_config().clone(), event_bus.clone())
                .map_err(|e| CoreError::Storage(e.to_string()))?,
        ));

        let contact_manager = Arc::new(RwLock::new(ContactManager::new(peer)));

        Ok(Self {
            config_manager,
            network_manager,
            storage_manager,
            contact_manager,
            crypto_manager,
            event_bus,
            is_initialized: false,
            user_name: None,
        })
    }

    pub async fn initialize(&mut self, user_name: Option<String>) -> Result<(), CoreError> {
        if self.is_initialized {
            return Ok(());
        }

        if let Some(name) = user_name {
            self.config_manager
                .set_user_name(name.clone())
                .map_err(|e| CoreError::Config(e.to_string()))?;
            self.user_name = Some(name);
        } else {
            self.user_name = Some(self.config_manager.get_user_name().to_string());
        }

        self.storage_manager
            .write()
            .await
            .initialize()
            .await
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        self.contact_manager
            .write()
            .await
            .load_contacts()
            .await
            .map_err(|e| CoreError::Contact(e.to_string()))?;

        self.is_initialized = true;
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    pub async fn start_server(&mut self) -> Result<(), CoreError> {
        if !self.is_initialized {
            return Err(CoreError::InvalidState("Core not initialized".to_string()));
        }

        self.network_manager
            .write()
            .await
            .start_server()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        Ok(())
    }

    pub async fn stop_server(&mut self) -> Result<(), CoreError> {
        self.network_manager
            .write()
            .await
            .shutdown()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        Ok(())
    }

    pub async fn restart_server(&mut self) -> Result<(), CoreError> {
        self.stop_server().await?;
        self.start_server().await?;
        Ok(())
    }

    pub fn is_server_started(&self) -> bool {
        true
    }

    pub async fn get_server_status(&self) -> String {
        if self.network_manager.read().await.is_running() {
            "🟢 Running".to_string()
        } else {
            "🔴 Stopped".to_string()
        }
    }

    pub async fn get_peer_info(&self) -> Option<String> {
        if let Some(ref name) = self.user_name {
            let peer = self.network_manager.read().await.get_peer().await;
            Some(format!("{} ({})", name, peer.get_full_address()))
        } else {
            None
        }
    }

    pub async fn generate_sg_link(&self) -> Result<String, CoreError> {
        self.contact_manager
            .read()
            .await
            .generate_sg_link()
            .await
            .map_err(|e| CoreError::Contact(e.to_string()))
    }

    pub async fn add_contact_by_sg_link(&self, sg_link: &str) -> Result<(), CoreError> {
        let contact = self
            .contact_manager
            .read()
            .await
            .add_contact_by_sg_link(sg_link)
            .await
            .map_err(|e| CoreError::Contact(e.to_string()))?;

        self.contact_manager
            .write()
            .await
            .add_contact(contact)
            .map_err(|e| CoreError::Contact(e.to_string()))?;

        Ok(())
    }

    pub async fn get_contacts(&self) -> Result<Vec<crate::network::Contact>, CoreError> {
        Ok(self.contact_manager.read().await.get_contacts().await)
    }

    pub async fn get_contact_count(&self) -> usize {
        self.contact_manager.read().await.get_contact_count()
    }

    pub async fn send_message(&self, contact_name: &str, content: &str) -> Result<(), CoreError> {
        if !self.is_initialized {
            return Err(CoreError::InvalidState("Core not initialized".to_string()));
        }

        let contact = self
            .contact_manager
            .read()
            .await
            .get_contact_by_name(contact_name)
            .await
            .ok_or_else(|| CoreError::Contact(format!("Contact '{}' not found", contact_name)))?;

        let message_id = self
            .network_manager
            .read()
            .await
            .send_chat_message(&contact, content)
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;

        let message = crate::network::ChatMessage {
            id: message_id,
            from: self
                .user_name
                .as_ref()
                .unwrap_or(&"user".to_string())
                .clone(),
            to: contact_name.to_string(),
            content: content.to_string(),
            msg_type: crate::network::ChatMessageType::Text,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            delivery_status: crate::network::DeliveryStatus::Sent,
        };

        self.storage_manager
            .write()
            .await
            .save_message(contact_name, &message)
            .await
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        Ok(())
    }

    pub async fn get_chat_messages(
        &self,
        contact_name: &str,
    ) -> Result<Vec<crate::network::ChatMessage>, CoreError> {
        self.storage_manager
            .read()
            .await
            .get_messages(contact_name)
            .await
            .map_err(|e| CoreError::Storage(e.to_string()))
    }

    pub async fn check_contact_online(&self, _contact_name: &str) -> bool {
        false
    }

    pub async fn get_unread_message_count(&self, contact_name: &str) -> Result<usize, CoreError> {
        let count = self
            .storage_manager
            .read()
            .await
            .get_unread_message_count(contact_name)
            .await
            .map_err(|e| CoreError::Storage(e.to_string()))?;
        Ok(count as usize)
    }

    pub async fn get_network_stats(&self) -> Result<crate::network::NetworkStats, CoreError> {
        self.network_manager
            .read()
            .await
            .get_network_stats()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))
    }

    pub async fn update_user_name(&mut self, new_name: String) -> Result<(), CoreError> {
        self.config_manager
            .set_user_name(new_name.clone())
            .map_err(|e| CoreError::Config(e.to_string()))?;
        self.user_name = Some(new_name);
        Ok(())
    }

    pub async fn get_connection_info(&self) -> Result<String, CoreError> {
        let peer = self.network_manager.read().await.get_peer().await;
        Ok(format!("Address: {}\nPort: {}", peer.address, peer.port))
    }

    pub async fn update_external_address(&self) -> Result<(), CoreError> {
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), CoreError> {
        self.network_manager
            .write()
            .await
            .shutdown()
            .await
            .map_err(|e| CoreError::Network(e.to_string()))?;
        Ok(())
    }

    pub fn get_event_bus(&self) -> EventBus {
        self.event_bus.clone()
    }

    pub async fn add_contact_manual(
        &self,
        contact: crate::network::Contact,
    ) -> Result<(), CoreError> {
        self.contact_manager
            .write()
            .await
            .add_contact(contact)
            .map_err(|e| CoreError::Contact(e.to_string()))?;
        Ok(())
    }

    pub fn get_contacts_sync(&self) -> Result<Vec<crate::network::Contact>, CoreError> {
        Ok(vec![])
    }

    pub async fn remove_contact_by_id(&self, contact_id: &str) -> Result<(), CoreError> {
        self.contact_manager
            .write()
            .await
            .remove_contact(contact_id)
            .map_err(|e| CoreError::Contact(e.to_string()))?;
        Ok(())
    }

    pub async fn update_contact_trust_level(
        &self,
        contact_id: &str,
        trust_level: crate::network::TrustLevel,
    ) -> Result<(), CoreError> {
        self.contact_manager
            .write()
            .await
            .set_trust_level(contact_id, trust_level)
            .map_err(|e| CoreError::Contact(e.to_string()))?;
        Ok(())
    }

    pub fn get_contact_by_id(&self, _contact_id: &str) -> Option<crate::network::Contact> {
        None
    }

    pub async fn send_chat_message(&self, to: &str, content: &str) -> Result<(), CoreError> {
        self.send_message(to, content).await
    }

    pub fn get_unread_count(&self, _contact_id: &str) -> Result<usize, CoreError> {
        Ok(0)
    }
}
