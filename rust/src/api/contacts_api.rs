use super::core_api::CORE;
use crate::data::contacts::ContactStats;
use crate::network::manager::{Contact, ContactStatus, TrustLevel};
use chrono::Utc;
use flutter_rust_bridge::frb;

#[frb]
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

// ✅ Убираем block_on, делаем полностью async
#[frb]
pub async fn get_contacts() -> Result<Vec<Contact>, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        match core.lock().await.get_contacts().await {
            Ok(contacts) => Ok(contacts),
            Err(e) => Err(format!("Failed to get contacts: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
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

#[frb]
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

#[frb]
pub async fn get_contact_by_id(contact_id: String) -> Result<Contact, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        match core.lock().await.get_contact_by_id(&contact_id) {
            Some(contact) => Ok(contact),
            None => Err("Contact not found".to_string()),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn search_contacts(query: String) -> Result<Vec<Contact>, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        match core.lock().await.get_contacts().await {
            Ok(contacts) => {
                let filtered: Vec<Contact> = contacts
                    .into_iter()
                    .filter(|c| c.name.to_lowercase().contains(&query.to_lowercase()))
                    .collect();
                Ok(filtered)
            }
            Err(e) => Err(format!("Failed to search contacts: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn get_contact_stats() -> Result<ContactStats, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        match core.lock().await.get_contacts().await {
            Ok(contacts) => {
                let total = contacts.len();
                let online = contacts
                    .iter()
                    .filter(|c| matches!(c.status, ContactStatus::Online))
                    .count();
                let trusted = contacts
                    .iter()
                    .filter(|c| matches!(c.trust_level, TrustLevel::Trusted))
                    .count();
                let blocked = 0; // Now blocked contacts in contacts manager

                Ok(ContactStats {
                    total_contacts: total,
                    online_contacts: online,
                    trusted_contacts: trusted,
                    blocked_contacts: blocked,
                    pending_contacts: contacts
                        .iter()
                        .filter(|c| matches!(c.trust_level, TrustLevel::Pending))
                        .count(),
                })
            }
            Err(e) => Err(format!("Failed to get contact stats: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}
