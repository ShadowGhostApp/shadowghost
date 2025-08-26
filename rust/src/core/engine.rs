use super::{Config, Profile, ProfileManager};
use crate::events::EventBus;
use crate::{chats, contacts, crypto, network, storage};
use std::path::PathBuf;
use std::sync::OnceLock;

pub static ENGINE: OnceLock<Engine> = OnceLock::new();

pub struct Engine {
    profile: Profile,
    profile_path: PathBuf,
    chats_manager: chats::Manager,
    contacts_manager: contacts::ContactManager,
    network_manager: network::NetworkManager,
    crypto_manager: crypto::SecurityManager,
    storage_manager: storage::StorageManager,
    config: Config,
    event_bus: EventBus,
}

#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error("Initialization error: {0}")]
    Initialization(String),
    #[error("Profile error: {0}")]
    Profile(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Manager error: {0}")]
    Manager(String),
}

impl Engine {
    pub fn new(profile: Profile, profile_path: PathBuf) -> Result<Self, EngineError> {
        let config = Config::load(&profile_path).map_err(|e| EngineError::Config(e))?;
        let event_bus = EventBus::new();

        // Create managers in correct dependency order
        let storage_manager = storage::StorageManager::new(&profile_path, event_bus.clone())
            .map_err(|e| EngineError::Manager(e.to_string()))?;

        let crypto_manager = crypto::SecurityManager::new(config.clone(), event_bus.clone())
            .map_err(|e| EngineError::Manager(e))?;

        let network_manager = network::NetworkManager::new_default()
            .map_err(|e| EngineError::Manager(e.to_string()))?;

        let contacts_manager = contacts::ContactManager::new(&profile_path)
            .map_err(|e| EngineError::Manager(e.to_string()))?;

        let chats_manager = chats::Manager::new(storage_manager.clone(), event_bus.clone())
            .map_err(|e| EngineError::Manager(e))?;

        Ok(Self {
            profile,
            profile_path,
            chats_manager,
            contacts_manager,
            network_manager,
            crypto_manager,
            storage_manager,
            config,
            event_bus,
        })
    }

    pub async fn initialize(&mut self, user_name: &str) -> Result<(), EngineError> {
        self.config.user_name = user_name.to_string();
        self.config
            .save(&self.profile_path)
            .map_err(|e| EngineError::Config(e))?;

        // Initialize all managers
        self.storage_manager
            .initialize()
            .await
            .map_err(|e| EngineError::Initialization(e))?;

        self.crypto_manager
            .initialize()
            .await
            .map_err(|e| EngineError::Initialization(e))?;

        self.network_manager
            .start()
            .map_err(|e| EngineError::Initialization(e.to_string()))?;

        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<(), EngineError> {
        self.network_manager
            .stop()
            .map_err(|e| EngineError::Manager(e.to_string()))?;
        Ok(())
    }

    pub fn chats(&self) -> &chats::Manager {
        &self.chats_manager
    }

    pub fn get_current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    pub fn format_chat_message(
        from: &str,
        to: &str,
        content: &str,
        msg_type: crate::network::ChatMessageType,
    ) -> ChatMessage {
        ChatMessage {
            id: Self::create_message_id(),
            from: from.to_string(),
            to: to.to_string(),
            content: content.to_string(),
            msg_type,
            timestamp: Self::get_current_timestamp(),
            delivery_status: crate::network::DeliveryStatus::Pending,
        }
    }
}
