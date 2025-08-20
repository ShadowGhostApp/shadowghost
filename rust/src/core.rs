use crate::contact_manager::{ContactError, ContactManager, ContactStats};
use crate::events::EventBus;
use crate::network::{
    ChatMessage, Contact, NetworkError, NetworkManager, NetworkStats, TrustLevel,
};
use crate::storage::{StorageError, StorageManager};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
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
    unread_cache: Arc<RwLock<HashMap<String, AtomicU64>>>,
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
            unread_cache: Arc::new(RwLock::new(HashMap::new())),
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
        true
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
            "Running".to_string()
        } else {
            "Stopped".to_string()
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
        let contact = contacts.get_contact_by_name(contact_name);
        if contact.is_none() {
            return Err(CoreError::Contact(format!(
                "Contact '{}' not found",
                contact_name
            )));
        }
        drop(contacts);

        let message_id = network.send_chat_message(contact_name, content).await?;

        let message = ChatMessage {
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

        let mut storage = self.storage_manager.write().await;
        storage.save_message(contact_name, &message).await?;

        Ok(())
    }

    pub async fn send_chat_message(
        &self,
        contact_name: &str,
        content: &str,
    ) -> Result<(), CoreError> {
        self.send_message(contact_name, content).await
    }

    pub async fn get_chat_messages(
        &self,
        contact_name: &str,
    ) -> Result<Vec<ChatMessage>, CoreError> {
        let storage = self.storage_manager.read().await;
        Ok(storage.get_messages(contact_name).await?)
    }

    pub async fn get_unread_message_count(&self, contact_name: &str) -> Result<u64, CoreError> {
        let storage = self.storage_manager.read().await;
        Ok(storage.get_unread_message_count(contact_name).await?)
    }

    pub async fn mark_messages_as_read(&self, contact_name: &str) -> Result<(), CoreError> {
        let mut storage = self.storage_manager.write().await;
        storage.mark_messages_as_read(contact_name).await?;
        self.reset_unread_count(contact_name).await;
        Ok(())
    }

    pub async fn update_unread_cache(&self, contact_name: &str) -> Result<(), CoreError> {
        let count = self.get_unread_message_count(contact_name).await?;
        let mut cache = self.unread_cache.write().await;
        cache
            .entry(contact_name.to_string())
            .or_insert_with(|| AtomicU64::new(0))
            .store(count, Ordering::Relaxed);
        Ok(())
    }

    pub fn get_unread_count_cached(&self, contact_name: &str) -> u64 {
        if let Ok(cache) = self.unread_cache.try_read() {
            cache
                .get(contact_name)
                .map(|atomic| atomic.load(Ordering::Relaxed))
                .unwrap_or(0)
        } else {
            0
        }
    }

    pub async fn get_unread_count_async(&self, contact_name: &str) -> Result<u64, CoreError> {
        let storage = self.storage_manager.read().await;
        Ok(storage.get_unread_message_count(contact_name).await?)
    }

    pub fn get_unread_count(&self, contact_name: &str) -> Result<u64, CoreError> {
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                let storage_manager = self.storage_manager.clone();
                let contact_name = contact_name.to_string();

                match handle.block_on(async move {
                    let storage = storage_manager.read().await;
                    storage.get_unread_message_count(&contact_name).await
                }) {
                    Ok(count) => Ok(count),
                    Err(e) => Err(CoreError::Storage(e.to_string())),
                }
            }
            Err(_) => Ok(0),
        }
    }

    pub async fn get_contacts(&self) -> Result<Vec<Contact>, CoreError> {
        let contacts = self.contact_manager.read().await;
        Ok(contacts.get_contacts())
    }

    pub fn get_contacts_sync(&self) -> Result<Vec<Contact>, CoreError> {
        if let Ok(contacts) = self.contact_manager.try_read() {
            Ok(contacts.get_contacts())
        } else {
            Ok(vec![])
        }
    }

    pub fn get_contact_by_id(&self, contact_id: &str) -> Option<Contact> {
        if let Ok(contacts) = self.contact_manager.try_read() {
            contacts.get_contact(contact_id)
        } else {
            None
        }
    }

    pub async fn get_contact_by_id_async(&self, contact_id: &str) -> Option<Contact> {
        let contacts = self.contact_manager.read().await;
        contacts.get_contact(contact_id)
    }

    pub async fn get_contact_by_id_from_storage(
        &self,
        contact_id: &str,
    ) -> Result<Option<Contact>, CoreError> {
        let storage = self.storage_manager.read().await;
        Ok(storage.get_contact(contact_id).await?)
    }

    pub async fn get_contact_stats(&self) -> Result<ContactStats, CoreError> {
        let contacts = self.contact_manager.read().await;
        Ok(contacts.get_contact_stats())
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

        contacts.add_contact(contact.clone())?;
        contacts.save_contacts().await?;
        drop(contacts);

        let mut storage = self.storage_manager.write().await;
        storage.save_contact(&contact).await?;

        Ok(())
    }

    pub async fn add_contact_manual(&self, contact: Contact) -> Result<(), CoreError> {
        let mut contacts = self.contact_manager.write().await;
        contacts.add_contact(contact.clone())?;
        contacts.save_contacts().await?;
        drop(contacts);

        let mut storage = self.storage_manager.write().await;
        storage.save_contact(&contact).await?;

        Ok(())
    }

    pub async fn remove_contact_by_id(&self, contact_id: &str) -> Result<(), CoreError> {
        let mut contacts = self.contact_manager.write().await;
        contacts.remove_contact(contact_id)?;
        contacts.save_contacts().await?;
        drop(contacts);

        let mut storage = self.storage_manager.write().await;
        storage.delete_contact(contact_id).await?;

        Ok(())
    }

    pub async fn update_contact_trust_level(
        &self,
        contact_id: &str,
        trust_level: TrustLevel,
    ) -> Result<(), CoreError> {
        let mut contacts = self.contact_manager.write().await;
        contacts.set_trust_level(contact_id, trust_level)?;
        contacts.save_contacts().await?;

        if let Some(contact) = contacts.get_contact(contact_id) {
            drop(contacts);
            let mut storage = self.storage_manager.write().await;
            storage.save_contact(&contact).await?;
        }

        Ok(())
    }

    pub async fn block_contact(&self, contact_id: &str) -> Result<(), CoreError> {
        let mut contacts = self.contact_manager.write().await;
        contacts.block_contact(contact_id)?;
        contacts.save_contacts().await?;
        Ok(())
    }

    pub async fn unblock_contact(&self, contact_id: &str) -> Result<(), CoreError> {
        let mut contacts = self.contact_manager.write().await;
        contacts.unblock_contact(contact_id)?;
        contacts.save_contacts().await?;
        Ok(())
    }

    pub async fn search_contacts(&self, query: &str) -> Result<Vec<Contact>, CoreError> {
        let contacts = self.contact_manager.read().await;
        Ok(contacts.search_contacts(query))
    }

    pub async fn get_trusted_contacts(&self) -> Result<Vec<Contact>, CoreError> {
        let contacts = self.contact_manager.read().await;
        Ok(contacts.get_trusted_contacts())
    }

    pub async fn get_online_contacts(&self) -> Result<Vec<Contact>, CoreError> {
        let contacts = self.contact_manager.read().await;
        Ok(contacts.get_online_contacts())
    }

    pub async fn get_blocked_contacts(&self) -> Result<Vec<Contact>, CoreError> {
        let contacts = self.contact_manager.read().await;
        Ok(contacts.get_blocked_contacts())
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

    pub async fn get_storage_stats(&self) -> Result<crate::storage::StorageStats, CoreError> {
        let storage = self.storage_manager.read().await;
        Ok(storage.get_stats().await?)
    }

    pub async fn update_user_name(&mut self, new_name: String) -> Result<(), CoreError> {
        self.user_name = Some(new_name.clone());
        let network = self.network_manager.read().await;
        network.update_peer_name(new_name).await?;
        Ok(())
    }

    pub async fn clear_unread_cache(&self) {
        let mut cache = self.unread_cache.write().await;
        cache.clear();
    }

    pub async fn increment_unread_count(&self, contact_name: &str) {
        let mut cache = self.unread_cache.write().await;
        let counter = cache
            .entry(contact_name.to_string())
            .or_insert_with(|| AtomicU64::new(0));
        counter.fetch_add(1, Ordering::Relaxed);
    }

    pub async fn reset_unread_count(&self, contact_name: &str) {
        let cache = self.unread_cache.write().await;
        if let Some(counter) = cache.get(contact_name) {
            counter.store(0, Ordering::Relaxed);
        }
    }

    pub async fn export_contacts(&self) -> Result<String, CoreError> {
        let contacts = self.contact_manager.read().await;
        Ok(contacts.export_contacts()?)
    }

    pub async fn import_contacts(&self, data: &str) -> Result<usize, CoreError> {
        let mut contacts = self.contact_manager.write().await;
        let imported_count = contacts.import_contacts(data)?;
        contacts.save_contacts().await?;
        Ok(imported_count)
    }

    pub async fn get_contact_count(&self) -> usize {
        let contacts = self.contact_manager.read().await;
        contacts.get_contact_count()
    }

    pub async fn clear_all_contacts(&self) -> Result<(), CoreError> {
        let mut contacts = self.contact_manager.write().await;
        contacts.clear_all_contacts();
        contacts.save_contacts().await?;
        drop(contacts);

        let mut storage = self.storage_manager.write().await;
        storage.clear_all_data().await?;

        self.clear_unread_cache().await;
        Ok(())
    }

    pub async fn backup_data(&self) -> Result<String, CoreError> {
        let mut storage = self.storage_manager.write().await;
        Ok(storage.backup().await?)
    }

    pub async fn restore_data(&self, backup_path: &str) -> Result<(), CoreError> {
        let mut storage = self.storage_manager.write().await;
        storage.restore_from_backup(backup_path).await?;

        let mut contacts = self.contact_manager.write().await;
        contacts.load_contacts().await?;

        self.clear_unread_cache().await;
        Ok(())
    }

    pub async fn delete_chat(&self, contact_name: &str) -> Result<(), CoreError> {
        let mut storage = self.storage_manager.write().await;
        storage.delete_chat(contact_name).await?;
        self.reset_unread_count(contact_name).await;
        Ok(())
    }

    pub async fn get_all_chats(&self) -> Result<HashMap<String, Vec<ChatMessage>>, CoreError> {
        let storage = self.storage_manager.read().await;
        Ok(storage.get_all_chats().await?)
    }
}
