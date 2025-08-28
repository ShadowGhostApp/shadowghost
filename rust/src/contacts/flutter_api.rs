use crate::contacts::{
    ContactExportOptions, ContactGroup, ContactImportResult, ContactInteractionStats,
    ContactSearchQuery, ContactStats, ContactValidationIssue,
};
use crate::core::ENGINE;
use crate::network::{Contact, ContactStatus, TrustLevel};

#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

#[cfg_attr(feature = "flutter", frb)]
pub async fn add_contact(contact: Contact) -> Result<(), String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .contacts()
        .add_contact(contact)
        .await
        .map_err(|e| e.to_string())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn remove_contact(contact_id: String) -> Result<(), String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .contacts()
        .remove_contact(&contact_id)
        .map_err(|e| e.to_string())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn get_contact(contact_id: String) -> Result<Contact, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .contacts()
        .get_contact(&contact_id)
        .ok_or_else(|| "Contact not found".to_string())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn get_all_contacts() -> Result<Vec<Contact>, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    Ok(engine.contacts().get_contacts())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn update_contact_status(
    contact_id: String,
    status: ContactStatus,
) -> Result<(), String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .contacts()
        .update_contact_status(&contact_id, status)
        .map_err(|e| e.to_string())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn set_trust_level(contact_id: String, trust_level: TrustLevel) -> Result<(), String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .contacts()
        .set_trust_level(&contact_id, trust_level)
        .map_err(|e| e.to_string())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn block_contact(contact_id: String) -> Result<(), String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .contacts()
        .block_contact(&contact_id)
        .map_err(|e| e.to_string())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn unblock_contact(contact_id: String) -> Result<(), String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .contacts()
        .unblock_contact(&contact_id)
        .map_err(|e| e.to_string())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn is_contact_blocked(contact_id: String) -> Result<bool, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    Ok(engine.contacts().is_contact_blocked(&contact_id))
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn get_contact_stats() -> Result<ContactStats, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    Ok(engine.contacts().get_contact_stats())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn find_contacts_by_name(name: String) -> Result<Vec<Contact>, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    Ok(engine.contacts().find_contacts_by_name(&name))
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn find_contacts_by_address(address: String) -> Result<Vec<Contact>, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    Ok(engine.contacts().find_contacts_by_address(&address))
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn get_contacts_by_trust_level(trust_level: TrustLevel) -> Result<Vec<Contact>, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    Ok(engine.contacts().get_contacts_by_trust_level(trust_level))
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn get_contacts_by_status(status: ContactStatus) -> Result<Vec<Contact>, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    Ok(engine.contacts().get_contacts_by_status(status))
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn batch_block_contacts(contact_ids: Vec<String>) -> Result<u32, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .contacts()
        .batch_block_contacts(contact_ids)
        .await
        .map_err(|e| e.to_string())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn batch_unblock_contacts(contact_ids: Vec<String>) -> Result<u32, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .contacts()
        .batch_unblock_contacts(contact_ids)
        .await
        .map_err(|e| e.to_string())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn save_contacts() -> Result<(), String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .contacts()
        .save_contacts()
        .await
        .map_err(|e| e.to_string())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn load_contacts() -> Result<(), String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .contacts()
        .load_contacts()
        .await
        .map_err(|e| e.to_string())
}
