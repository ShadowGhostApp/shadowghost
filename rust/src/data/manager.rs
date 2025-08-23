use crate::core::config::AppConfig;
use crate::data::{ContactManager, StorageManager};
use crate::events::EventBus;
use crate::network::Contact;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct DataManager {
    pub contacts: Arc<RwLock<ContactManager>>,
    pub storage: Arc<RwLock<StorageManager>>,
    config: AppConfig,
    event_bus: EventBus,
}

impl DataManager {
    pub fn new(config: AppConfig, event_bus: EventBus) -> Result<Self, crate::core::CoreError> {
        let storage_path = config.storage.data_path.clone();

        let storage_manager = StorageManager::new(config.clone(), event_bus.clone())
            .map_err(|e| crate::core::CoreError::Storage(e.to_string()))?;

        let contact_manager =
            ContactManager::new_with_storage(format!("{}/contacts.json", storage_path))
                .map_err(|e| crate::core::CoreError::Contact(e.to_string()))?;

        Ok(Self {
            contacts: Arc::new(RwLock::new(contact_manager)),
            storage: Arc::new(RwLock::new(storage_manager)),
            config,
            event_bus,
        })
    }

    pub async fn initialize(&self) -> Result<(), String> {
        // Initialize storage
        self.storage
            .write()
            .await
            .initialize()
            .await
            .map_err(|e| format!("Failed to initialize storage: {}", e))?;

        // Load contacts
        self.contacts
            .write()
            .await
            .load_contacts()
            .await
            .map_err(|e| format!("Failed to load contacts: {}", e))?;

        println!("Data manager initialized successfully");
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), String> {
        // Save contacts before shutdown
        self.contacts
            .read()
            .await
            .save_contacts()
            .await
            .map_err(|e| format!("Failed to save contacts: {}", e))?;

        println!("Data manager shutdown completed");
        Ok(())
    }

    pub async fn add_contact(&self, contact: Contact) -> Result<(), String> {
        self.contacts
            .write()
            .await
            .add_contact(contact.clone())
            .map_err(|e| e.to_string())?;

        // Save to persistent storage
        self.contacts
            .read()
            .await
            .save_contacts()
            .await
            .map_err(|e| e.to_string())?;

        // Emit event
        use crate::events::{AppEvent, NetworkEvent};
        self.event_bus
            .emit(AppEvent::Network(NetworkEvent::ContactAdded { contact }));

        Ok(())
    }

    pub async fn get_all_contacts(&self) -> Result<Vec<Contact>, String> {
        self.contacts
            .read()
            .await
            .get_contacts()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn remove_contact(&self, contact_id: &str) -> Result<(), String> {
        self.contacts
            .write()
            .await
            .remove_contact(contact_id)
            .map_err(|e| e.to_string())?;

        // Save changes
        self.contacts
            .read()
            .await
            .save_contacts()
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn update_contact_status(
        &self,
        contact_id: &str,
        status: crate::network::ContactStatus,
    ) -> Result<(), String> {
        self.contacts
            .write()
            .await
            .update_contact_status(contact_id, status)
            .map_err(|e| e.to_string())?;

        // Save changes
        self.contacts
            .read()
            .await
            .save_contacts()
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn backup_data(&self) -> Result<String, String> {
        self.storage
            .write()
            .await
            .backup()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn restore_from_backup(&self, backup_path: &str) -> Result<(), String> {
        self.storage
            .write()
            .await
            .restore_from_backup(backup_path)
            .await
            .map_err(|e| e.to_string())?;

        // Reload contacts after restore
        self.contacts
            .write()
            .await
            .load_contacts()
            .await
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub async fn get_storage_stats(&self) -> Result<crate::data::StorageStats, String> {
        self.storage
            .read()
            .await
            .get_stats()
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn validate_data_integrity(&self) -> Result<Vec<String>, String> {
        let mut issues = Vec::new();

        // Validate contacts
        match self.contacts.read().await.validate_contacts().await {
            Ok(contact_issues) => issues.extend(contact_issues),
            Err(e) => issues.push(format!("Contact validation failed: {}", e)),
        }

        // Validate storage
        match self.storage.read().await.validate_chats().await {
            Ok(chat_issues) => issues.extend(chat_issues),
            Err(e) => issues.push(format!("Chat validation failed: {}", e)),
        }

        Ok(issues)
    }

    pub async fn cleanup_old_data(&self, days: u32) -> Result<usize, String> {
        // This would implement cleanup logic for old messages, logs, etc.
        // For now, return 0 items cleaned
        println!("Data cleanup completed (simulated)");
        Ok(0)
    }

    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }

    pub fn get_event_bus(&self) -> &EventBus {
        &self.event_bus
    }
}
