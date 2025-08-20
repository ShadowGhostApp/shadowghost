use super::core::CORE;
use crate::prelude::*;
use flutter_rust_bridge::frb;

pub async fn add_contact(name: String, address: String) -> Result<Contact, String> {
    let core_guard = CORE.lock().unwrap();
    if let Some(core) = core_guard.clone() {
        drop(core_guard);

        let contact = Contact {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            address,
            status: ContactStatus::Offline,
            last_seen: chrono::Utc::now().timestamp() as u64,
            trust_level: TrustLevel::Unknown,
        };

        match core.add_contact(contact.clone()).await {
            Ok(_) => Ok(contact),
            Err(e) => Err(format!("Failed to add contact: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb(sync)]
pub fn get_contacts() -> Result<Vec<Contact>, String> {
    let core_guard = CORE.lock().unwrap();
    if let Some(core) = core_guard.as_ref() {
        match core.get_contacts() {
            Ok(contacts) => Ok(contacts),
            Err(e) => Err(format!("Failed to get contacts: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

pub async fn remove_contact(contact_id: String) -> Result<String, String> {
    let core_guard = CORE.lock().unwrap();
    if let Some(core) = core_guard.clone() {
        drop(core_guard);
        match core.remove_contact(&contact_id).await {
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
    let core_guard = CORE.lock().unwrap();
    if let Some(core) = core_guard.clone() {
        drop(core_guard);
        match core.update_contact_trust(&contact_id, trust_level).await {
            Ok(_) => Ok("Contact trust level updated".to_string()),
            Err(e) => Err(format!("Failed to update contact trust: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb(sync)]
pub fn get_contact_by_id(contact_id: String) -> Result<Contact, String> {
    let core_guard = CORE.lock().unwrap();
    if let Some(core) = core_guard.as_ref() {
        match core.get_contact(&contact_id) {
            Ok(Some(contact)) => Ok(contact),
            Ok(None) => Err("Contact not found".to_string()),
            Err(e) => Err(format!("Failed to get contact: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}
