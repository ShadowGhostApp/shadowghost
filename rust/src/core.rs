use crate::contact_manager::{ContactError, ContactManager};
use crate::events::EventBus;
use crate::network::{ChatMessage, Contact, NetworkError, NetworkManager, NetworkStats};
use crate::storage::{StorageError, StorageManager};
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug)]
pub enum CoreError {
    Network(String),
    Storage(String),
    Contact(String),
    InvalidState(String),
    Initialization(String),
}

impl fmt::Display for CoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CoreError::Network(msg) => write!(f, "Network error: {}", msg),
            CoreError::Storage(msg) => write!(f, "Storage error: {}", msg),
            CoreError::Contact(msg) => write!(f, "Contact error: {}", msg),
            CoreError::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
            CoreError::Initialization(msg) => write!(f, "Initialization error: {}", msg),
        }
    }
}

impl Error for CoreError {}

impl From<NetworkError> for CoreError {
    fn from(error: NetworkError) -> Self {
        CoreError::Network(error.to_string())
    }
}

impl From<StorageError> for CoreError {
    fn from(error: StorageError) -> Self {
        CoreError::Storage(error.to_string())
    }
}

impl From<ContactError> for CoreError {
    fn from(error: ContactError) -> Self {
        CoreError::Contact(error.to_string())
    }
}

pub struct ShadowGhostCore {
    network_manager: Arc<RwLock<NetworkManager>>,
    storage_manager: Arc<RwLock<StorageManager>>,
    contact_manager: Arc<RwLock<ContactManager>>,
    event_bus: EventBus,
    is_initialized: bool,
    user_name: Option<String>,
}

impl ShadowGhostCore {
    pub fn new() -> Result<Self, CoreError> {
        let event_bus = EventBus::new();

        let network_manager =
            NetworkManager::new().map_err(|e| CoreError::Network(e.to_string()))?;

        let storage_manager = StorageManager::new("./data".to_string())
            .map_err(|e| CoreError::Storage(e.to_string()))?;

        let contact_manager = ContactManager::new("./data/contacts.json".to_string())
            .map_err(|e| CoreError::Contact(e.to_string()))?;

        Ok(Self {
            network_manager: Arc::new(RwLock::new(network_manager)),
            storage_manager: Arc::new(RwLock::new(storage_manager)),
            contact_manager: Arc::new(RwLock::new(contact_manager)),
            event_bus,
            is_initialized: false,
            user_name: None,
        })
    }

    pub fn get_event_bus(&self) -> EventBus {
        self.event_bus.clone()
    }

    pub async fn initialize(&mut self, user_name: Option<String>) -> Result<(), CoreError> {
        self.user_name = user_name.or_else(|| Some("user".to_string()));

        let mut storage = self.storage_manager.write().await;
        storage.initialize().await?;
        drop(storage);

        let mut contacts = self.contact_manager.write().await;
        contacts.load_contacts().await?;
        drop(contacts);

        self.is_initialized = true;
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.is_initialized
    }

    pub async fn get_peer_info(&self) -> Option<String> {
        if let Some(name) = &self.user_name {
            Some(format!("{} (127.0.0.1:8080)", name))
        } else {
            None
        }
    }

    pub fn is_server_started(&self) -> bool {
        false
    }

    pub async fn start_server(&mut self) -> Result<(), CoreError> {
        if !self.is_initialized {
            return Err(CoreError::InvalidState("Core not initialized".to_string()));
        }

        let mut network = self.network_manager.write().await;
        network.start_server().await?;
        Ok(())
    }

    pub async fn stop_server(&mut self) -> Result<(), CoreError> {
        let network = self.network_manager.read().await;
        network.shutdown().await?;
        Ok(())
    }

    pub async fn restart_server(&mut self) -> Result<(), CoreError> {
        self.stop_server().await?;
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        self.start_server().await?;
        Ok(())
    }

    pub async fn get_server_status(&self) -> String {
        let network = self.network_manager.read().await;
        if network.is_running() {
            "🟢 Running".to_string()
        } else {
            "🔴 Stopped".to_string()
        }
    }

    pub async fn shutdown(&self) -> Result<(), CoreError> {
        let network = self.network_manager.read().await;
        network.shutdown().await?;
        Ok(())
    }

    pub async fn send_message(&self, contact_name: &str, content: &str) -> Result<(), CoreError> {
        if !self.is_initialized {
            return Err(CoreError::InvalidState("Core not initialized".to_string()));
        }

        let network = self.network_manager.read().await;
        if !network.is_running() {
            return Err(CoreError::InvalidState("Server not running".to_string()));
        }

        let contacts = self.contact_manager.read().await;
        if contacts.get_contact_by_name(contact_name).is_none() {
            return Err(CoreError::Contact(format!(
                "Contact '{}' not found",
                contact_name
            )));
        }
        drop(contacts);

        network.send_chat_message(contact_name, content).await?;
        Ok(())
    }

    pub async fn get_chat_messages(
        &self,
        contact_name: &str,
    ) -> Result<Vec<ChatMessage>, CoreError> {
        let network = self.network_manager.read().await;
        Ok(network.get_chat_messages(contact_name).await?)
    }

    pub async fn get_unread_message_count(&self, contact_name: &str) -> Result<u64, CoreError> {
        let storage = self.storage_manager.read().await;
        Ok(storage.get_unread_message_count(contact_name).await?)
    }

    pub async fn get_contacts(&self) -> Result<Vec<Contact>, CoreError> {
        let contacts = self.contact_manager.read().await;
        Ok(contacts.get_contacts())
    }

    pub async fn add_contact_by_sg_link(&self, sg_link: &str) -> Result<(), CoreError> {
        if !self.is_initialized {
            return Err(CoreError::InvalidState("Core not initialized".to_string()));
        }

        if !sg_link.starts_with("sg://") {
            return Err(CoreError::Contact("Invalid SG link format".to_string()));
        }

        let link_data = &sg_link[5..];

        let mut contacts = self.contact_manager.write().await;
        let contact = contacts.create_contact_from_sg_link(link_data)?;

        let user_name = self
            .user_name
            .as_ref()
            .unwrap_or(&"user".to_string())
            .clone();
        if contact.name == user_name {
            return Err(CoreError::Contact(
                "Cannot add yourself as contact".to_string(),
            ));
        }

        contacts.add_contact(contact)?;
        contacts.save_contacts().await?;
        Ok(())
    }

    pub async fn generate_sg_link(&self) -> Result<String, CoreError> {
        if !self.is_initialized {
            return Err(CoreError::InvalidState("Core not initialized".to_string()));
        }
        let default_name = "user".to_string();

        let user_name = self.user_name.as_ref().unwrap_or(&default_name);
        let peer_data = format!(
            r#"{{"id":"{}","name":"{}","address":"127.0.0.1:8080","public_key":[],"connected_at":"{}","last_seen":"{}"}}"#,
            uuid::Uuid::new_v4(),
            user_name,
            chrono::Utc::now().to_rfc3339(),
            chrono::Utc::now().to_rfc3339()
        );

        use base64::{Engine as _, engine::general_purpose};
        let encoded = general_purpose::STANDARD.encode(peer_data);
        Ok(format!("sg://{}", encoded))
    }

    pub async fn check_contact_online(&self, contact_name: &str) -> bool {
        let contacts = self.contact_manager.read().await;
        if let Some(_contact) = contacts.get_contact_by_name(contact_name) {
            drop(contacts);
            true
        } else {
            false
        }
    }

    pub async fn get_connection_info(&self) -> Result<String, CoreError> {
        let info = format!(
            "Local Address: 127.0.0.1:8080\nExternal Address: Not available\nServer Status: {}",
            self.get_server_status().await
        );
        Ok(info)
    }

    pub async fn update_external_address(&self) -> Result<(), CoreError> {
        Ok(())
    }

    pub async fn get_network_stats(&self) -> Result<NetworkStats, CoreError> {
        let network = self.network_manager.read().await;
        Ok(network.get_network_stats().await?)
    }

    pub async fn update_user_name(&mut self, new_name: String) -> Result<(), CoreError> {
        self.user_name = Some(new_name.clone());
        let network = self.network_manager.read().await;
        network.update_peer_name(new_name).await?;
        Ok(())
    }
}
