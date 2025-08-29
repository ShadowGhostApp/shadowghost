use crate::core::types::*;
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

impl Engine {
    pub fn new(profile: Profile, profile_path: PathBuf) -> Result<Self, CoreError> {
        let config = Config::load(&profile_path).map_err(|e| CoreError::Config(e))?;
        let event_bus = EventBus::new();

        let storage_manager_for_chats =
            storage::StorageManager::new(&profile_path, event_bus.clone())
                .map_err(|e| CoreError::Manager(e.to_string()))?;

        let storage_manager = storage::StorageManager::new(&profile_path, event_bus.clone())
            .map_err(|e| CoreError::Manager(e.to_string()))?;

        let crypto_manager = crypto::SecurityManager::new(config.clone(), event_bus.clone())
            .map_err(|e| CoreError::Manager(e))?;

        let network_manager = network::NetworkManager::new_default()
            .map_err(|e| CoreError::Manager(e.to_string()))?;

        let contacts_manager = contacts::ContactManager::new(&profile_path)
            .map_err(|e| CoreError::Manager(e.to_string()))?;

        let chats_manager = chats::Manager::new(
            std::sync::Arc::new(tokio::sync::RwLock::new(storage_manager_for_chats)),
            event_bus.clone(),
        )
        .map_err(|e| CoreError::Manager(e))?;

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

    pub async fn initialize(&mut self, user_name: &str) -> Result<(), CoreError> {
        self.config.user_name = user_name.to_string();
        self.config
            .save(&self.profile_path)
            .map_err(|e| CoreError::Config(e))?;

        self.storage_manager
            .initialize()
            .await
            .map_err(|e| CoreError::Initialization(e))?;

        self.crypto_manager
            .initialize()
            .await
            .map_err(|e| CoreError::Initialization(e))?;

        self.network_manager
            .start()
            .map_err(|e| CoreError::Initialization(e.to_string()))?;

        Ok(())
    }

    pub async fn shutdown(&mut self) -> Result<(), CoreError> {
        self.network_manager
            .stop()
            .map_err(|e| CoreError::Manager(e.to_string()))?;
        Ok(())
    }

    pub fn chats(&self) -> &chats::Manager {
        &self.chats_manager
    }

    pub fn contacts(&self) -> &contacts::ContactManager {
        &self.contacts_manager
    }

    pub fn network(&self) -> &network::NetworkManager {
        &self.network_manager
    }

    pub fn storage(&self) -> &storage::StorageManager {
        &self.storage_manager
    }

    pub fn crypto(&self) -> &crypto::SecurityManager {
        &self.crypto_manager
    }

    pub fn get_current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    pub fn create_message_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    pub fn format_chat_message(
        from: &str,
        to: &str,
        content: &str,
        msg_type: crate::network::ChatMessageType,
    ) -> crate::network::ChatMessage {
        crate::network::ChatMessage {
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