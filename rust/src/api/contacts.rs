use super::core::CORE;
use crate::network::{Contact, ContactStatus, TrustLevel};
use chrono::Utc;
use flutter_rust_bridge::frb;

pub async fn add_contact(name: String, address: String) -> Result<Contact, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        let contact = Contact {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            address,
            status: ContactStatus::Offline,
            last_seen: Some(Utc::now()),
            trust_level: TrustLevel::Unknown,
        };

        match core.lock().await.add_contact_manual(contact.clone()).await {
            Ok(_) => Ok(contact),
            Err(e) => Err(format!("Failed to add contact: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb(sync)]
pub fn get_contacts() -> Result<Vec<Contact>, String> {
    let rt = tokio::runtime::Handle::current();
    let core_guard = rt.block_on(CORE.lock());
    if let Some(core) = core_guard.as_ref() {
        match rt.block_on(core.lock()).get_contacts_sync() {
            Ok(contacts) => Ok(contacts),
            Err(e) => Err(format!("Failed to get contacts: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

pub async fn remove_contact(contact_id: String) -> Result<String, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        match core.lock().await.remove_contact_by_id(&contact_id).await {
            Ok(_) => Ok("Contact removed successfully".to_string()),
            Err(e) => Err(format!("Failed to remove contact: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

pub async fn update_contact_trust_level(
    contact_id: String,
    trust_level: TrustLevel,
) -> Result<String, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        match core
            .lock()
            .await
            .update_contact_trust_level(&contact_id, trust_level)
            .await
        {
            Ok(_) => Ok("Contact trust level updated".to_string()),
            Err(e) => Err(format!("Failed to update contact trust: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb(sync)]
pub fn get_contact_by_id(contact_id: String) -> Result<Contact, String> {
    let rt = tokio::runtime::Handle::current();
    let core_guard = rt.block_on(CORE.lock());
    if let Some(core) = core_guard.as_ref() {
        match rt.block_on(core.lock()).get_contact_by_id(&contact_id) {
            Some(contact) => Ok(contact),
            None => Err("Contact not found".to_string()),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}
