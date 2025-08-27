use crate::contacts::types::*;
use crate::network::{Contact, ContactStatus, PeerData, TrustLevel};
use base64::{engine::general_purpose, Engine as _};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::path::PathBuf;

#[derive(Debug)]
pub enum ContactError {
    InvalidContact(String),
    ContactNotFound(String),
    ContactExists(String),
    SerializationError(String),
    IoError(String),
}

impl fmt::Display for ContactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContactError::InvalidContact(msg) => write!(f, "Invalid contact: {}", msg),
            ContactError::ContactNotFound(msg) => write!(f, "Contact not found: {}", msg),
            ContactError::ContactExists(msg) => write!(f, "Contact exists: {}", msg),
            ContactError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            ContactError::IoError(msg) => write!(f, "IO error: {}", msg),
        }
    }
}

impl Error for ContactError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactBook {
    contacts: HashMap<String, Contact>,
    blocked_contacts: HashMap<String, BlockedContactInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockedContactInfo {
    pub blocked_at: chrono::DateTime<chrono::Utc>,
    pub reason: String,
    pub blocked_by_user: bool,
}

impl ContactBook {
    pub fn new() -> Self {
        Self {
            contacts: HashMap::new(),
            blocked_contacts: HashMap::new(),
        }
    }

    pub fn add_contact(&mut self, contact: Contact) -> Result<(), ContactError> {
        self.contacts.insert(contact.id.clone(), contact);
        Ok(())
    }

    pub fn remove_contact(&mut self, contact_id: &str) -> Result<(), ContactError> {
        if !self.contacts.contains_key(contact_id) {
            return Err(ContactError::ContactNotFound(format!(
                "Contact with ID {} not found",
                contact_id
            )));
        }

        self.contacts.remove(contact_id);
        self.blocked_contacts.remove(contact_id);
        Ok(())
    }

    pub fn get_contact(&self, contact_id: &str) -> Option<&Contact> {
        self.contacts.get(contact_id)
    }

    pub fn get_contacts(&self) -> Vec<Contact> {
        self.contacts.values().cloned().collect()
    }

    pub fn update_contact_status(
        &mut self,
        contact_id: &str,
        status: ContactStatus,
    ) -> Result<(), ContactError> {
        if let Some(contact) = self.contacts.get_mut(contact_id) {
            contact.status = status;
            Ok(())
        } else {
            Err(ContactError::ContactNotFound(format!(
                "Contact with ID {} not found",
                contact_id
            )))
        }
    }

    pub fn block_contact(&mut self, contact_id: &str) -> Result<(), ContactError> {
        if !self.contacts.contains_key(contact_id) {
            return Err(ContactError::ContactNotFound(format!(
                "Contact with ID {} not found",
                contact_id
            )));
        }

        let blocked_info = BlockedContactInfo {
            blocked_at: chrono::Utc::now(),
            reason: "Blocked by user".to_string(),
            blocked_by_user: true,
        };

        self.blocked_contacts
            .insert(contact_id.to_string(), blocked_info);
        Ok(())
    }

    pub fn unblock_contact(&mut self, contact_id: &str) -> Result<(), ContactError> {
        if self.blocked_contacts.remove(contact_id).is_some() {
            Ok(())
        } else {
            Err(ContactError::ContactNotFound(format!(
                "Blocked contact with ID {} not found",
                contact_id
            )))
        }
    }

    pub fn is_blocked(&self, contact_id: &str) -> bool {
        self.blocked_contacts.contains_key(contact_id)
    }

    pub fn get_blocked_contact_ids(&self) -> Vec<String> {
        self.blocked_contacts.keys().cloned().collect()
    }

    pub fn find_contacts_by_name(&self, name: &str) -> Vec<Contact> {
        let name_lower = name.to_lowercase();
        self.contacts
            .values()
            .filter(|contact| contact.name.to_lowercase().contains(&name_lower))
            .cloned()
            .collect()
    }

    pub fn find_contacts_by_address(&self, address: &str) -> Vec<Contact> {
        let address_lower = address.to_lowercase();
        self.contacts
            .values()
            .filter(|contact| contact.address.to_lowercase().contains(&address_lower))
            .cloned()
            .collect()
    }
}

pub struct ContactManager {
    contact_book: ContactBook,
    data_path: PathBuf,
}

impl ContactManager {
    pub fn new(data_path: &PathBuf) -> Result<Self, String> {
        Ok(Self {
            contact_book: ContactBook::new(),
            data_path: data_path.clone(),
        })
    }

    pub fn add_contact(&mut self, contact: Contact) -> Result<(), ContactError> {
        self.contact_book.add_contact(contact)
    }

    pub fn remove_contact(&mut self, contact_id: &str) -> Result<(), ContactError> {
        self.contact_book.remove_contact(contact_id)
    }

    pub fn get_contact(&self, contact_id: &str) -> Option<Contact> {
        self.contact_book.get_contact(contact_id).cloned()
    }

    pub fn get_contacts(&self) -> Vec<Contact> {
        self.contact_book.get_contacts()
    }

    pub fn update_contact_status(
        &mut self,
        contact_id: &str,
        status: ContactStatus,
    ) -> Result<(), ContactError> {
        self.contact_book.update_contact_status(contact_id, status)
    }

    pub fn set_trust_level(
        &mut self,
        contact_id: &str,
        trust_level: TrustLevel,
    ) -> Result<(), ContactError> {
        if let Some(contact) = self.contact_book.contacts.get_mut(contact_id) {
            contact.trust_level = trust_level;
            Ok(())
        } else {
            Err(ContactError::ContactNotFound(format!(
                "Contact with ID {} not found",
                contact_id
            )))
        }
    }

    pub fn block_contact(&mut self, contact_id: &str) -> Result<(), ContactError> {
        self.contact_book.block_contact(contact_id)
    }

    pub fn unblock_contact(&mut self, contact_id: &str) -> Result<(), ContactError> {
        self.contact_book.unblock_contact(contact_id)
    }

    pub fn is_contact_blocked(&self, contact_id: &str) -> bool {
        self.contact_book.is_blocked(contact_id)
    }

    pub fn get_contact_stats(&self) -> ContactStats {
        let all_contacts = self.contact_book.get_contacts();
        let online_count = all_contacts
            .iter()
            .filter(|c| matches!(c.status, ContactStatus::Online))
            .count();
        let trusted_count = all_contacts
            .iter()
            .filter(|c| matches!(c.trust_level, TrustLevel::Trusted))
            .count();
        let blocked_count = all_contacts
            .iter()
            .filter(|c| self.contact_book.is_blocked(&c.id))
            .count();

        ContactStats {
            total_contacts: all_contacts.len(),
            online_contacts: online_count,
            trusted_contacts: trusted_count,
            blocked_contacts: blocked_count,
            pending_contacts: all_contacts
                .iter()
                .filter(|c| matches!(c.trust_level, TrustLevel::Pending))
                .count(),
        }
    }

    pub fn find_contacts_by_name(&self, name: &str) -> Vec<Contact> {
        self.contact_book.find_contacts_by_name(name)
    }

    pub fn find_contacts_by_address(&self, address: &str) -> Vec<Contact> {
        self.contact_book.find_contacts_by_address(address)
    }

    pub fn get_contacts_by_trust_level(&self, trust_level: TrustLevel) -> Vec<Contact> {
        self.contact_book
            .get_contacts()
            .into_iter()
            .filter(|contact| contact.trust_level == trust_level)
            .collect()
    }

    pub fn get_contacts_by_status(&self, status: ContactStatus) -> Vec<Contact> {
        self.contact_book
            .get_contacts()
            .into_iter()
            .filter(|contact| contact.status == status)
            .collect()
    }

    pub async fn batch_block_contacts(
        &mut self,
        contact_ids: Vec<String>,
    ) -> Result<u32, ContactError> {
        let mut blocked_count = 0;
        for contact_id in contact_ids {
            if self.contact_book.block_contact(&contact_id).is_ok() {
                blocked_count += 1;
            }
        }
        if blocked_count > 0 {
            self.save_contacts().await?;
        }
        Ok(blocked_count)
    }

    pub async fn batch_unblock_contacts(
        &mut self,
        contact_ids: Vec<String>,
    ) -> Result<u32, ContactError> {
        let mut unblocked_count = 0;
        for contact_id in contact_ids {
            if self.contact_book.unblock_contact(&contact_id).is_ok() {
                unblocked_count += 1;
            }
        }
        if unblocked_count > 0 {
            self.save_contacts().await?;
        }
        Ok(unblocked_count)
    }

    pub fn get_blocked_contact_ids(&self) -> Vec<String> {
        self.contact_book.get_blocked_contact_ids()
    }

    pub async fn cleanup_blocked_contacts(&mut self, days: u32) -> Result<u32, ContactError> {
        let cutoff_time = chrono::Utc::now() - chrono::Duration::days(days as i64);
        let all_contacts = self.contact_book.get_contacts();
        let mut cleanup_count = 0;

        let contacts_to_remove: Vec<String> = all_contacts
            .iter()
            .filter(|contact| {
                if self.contact_book.is_blocked(&contact.id) {
                    if let Some(last_seen) = contact.last_seen {
                        last_seen < cutoff_time
                    } else {
                        true
                    }
                } else {
                    false
                }
            })
            .map(|contact| contact.id.clone())
            .collect();

        for contact_id in contacts_to_remove {
            if self.contact_book.remove_contact(&contact_id).is_ok() {
                cleanup_count += 1;
            }
        }

        if cleanup_count > 0 {
            self.save_contacts().await?;
        }

        Ok(cleanup_count)
    }

    pub fn is_contact_allowed(&self, contact_id: &str) -> bool {
        !self.is_contact_blocked(contact_id)
    }

    pub async fn update_contact_activity(&mut self, contact_id: &str) -> Result<(), ContactError> {
        if let Some(contact) = self.contact_book.contacts.get_mut(contact_id) {
            contact.last_seen = Some(chrono::Utc::now());
            contact.status = ContactStatus::Online;
            Ok(())
        } else {
            Err(ContactError::ContactNotFound(format!(
                "Contact with ID {} not found",
                contact_id
            )))
        }
    }

    pub async fn mark_contacts_offline(
        &mut self,
        older_than_minutes: u32,
    ) -> Result<u32, ContactError> {
        let cutoff_time = chrono::Utc::now() - chrono::Duration::minutes(older_than_minutes as i64);
        let mut updated_count = 0;

        for contact in self.contact_book.contacts.values_mut() {
            if matches!(contact.status, ContactStatus::Online) {
                if let Some(last_seen) = contact.last_seen {
                    if last_seen < cutoff_time {
                        contact.status = ContactStatus::Offline;
                        updated_count += 1;
                    }
                }
            }
        }

        if updated_count > 0 {
            self.save_contacts().await?;
        }

        Ok(updated_count)
    }

    pub fn get_contact_interaction_stats(
        &self,
        contact_id: &str,
    ) -> Result<ContactInteractionStats, ContactError> {
        if let Some(contact) = self.contact_book.get_contact(contact_id) {
            let days_since_added = if let Some(last_seen) = contact.last_seen {
                let now = chrono::Utc::now();
                let duration = now.signed_duration_since(last_seen);
                duration.num_days().max(0) as u32
            } else {
                0
            };

            Ok(ContactInteractionStats {
                contact_id: contact_id.to_string(),
                days_since_last_seen: days_since_added,
                is_trusted: matches!(contact.trust_level, TrustLevel::Trusted),
                is_blocked: self.is_contact_blocked(contact_id),
                current_status: contact.status.clone(),
                trust_level: contact.trust_level.clone(),
            })
        } else {
            Err(ContactError::ContactNotFound(format!(
                "Contact with ID {} not found",
                contact_id
            )))
        }
    }

    pub fn validate_contact_data(&self) -> Vec<ContactValidationIssue> {
        let mut issues = Vec::new();
        let contacts = self.contact_book.get_contacts();

        for contact in contacts {
            if contact.name.trim().is_empty() {
                issues.push(ContactValidationIssue {
                    contact_id: contact.id.clone(),
                    issue_type: ContactIssueType::EmptyName,
                    description: "Contact has empty name".to_string(),
                    severity: IssueSeverity::Medium,
                });
            }

            if contact.address.trim().is_empty() {
                issues.push(ContactValidationIssue {
                    contact_id: contact.id.clone(),
                    issue_type: ContactIssueType::EmptyAddress,
                    description: "Contact has empty address".to_string(),
                    severity: IssueSeverity::High,
                });
            }

            if !self.is_valid_address(&contact.address) {
                issues.push(ContactValidationIssue {
                    contact_id: contact.id.clone(),
                    issue_type: ContactIssueType::InvalidAddress,
                    description: format!("Invalid address format: {}", contact.address),
                    severity: IssueSeverity::High,
                });
            }
        }

        issues
    }

    fn is_valid_address(&self, address: &str) -> bool {
        if address.is_empty() {
            return false;
        }

        if !address.contains(':') {
            return false;
        }

        let parts: Vec<&str> = address.split(':').collect();
        if parts.len() != 2 {
            return false;
        }

        parts[1].parse::<u16>().is_ok()
    }

    pub async fn save_contacts(&self) -> Result<(), ContactError> {
        let contacts_file = self.data_path.join("contacts.json");
        let content = serde_json::to_string_pretty(&self.contact_book)
            .map_err(|e| ContactError::SerializationError(e.to_string()))?;

        tokio::fs::write(&contacts_file, content)
            .await
            .map_err(|e| ContactError::IoError(e.to_string()))?;

        Ok(())
    }

    pub async fn load_contacts(&mut self) -> Result<(), ContactError> {
        let contacts_file = self.data_path.join("contacts.json");

        if contacts_file.exists() {
            let content = tokio::fs::read_to_string(&contacts_file)
                .await
                .map_err(|e| ContactError::IoError(e.to_string()))?;

            self.contact_book = serde_json::from_str(&content)
                .map_err(|e| ContactError::SerializationError(e.to_string()))?;
        }

        Ok(())
    }
}

// SG Link operations
pub fn generate_sg_link(peer: &crate::core::Peer) -> Result<String, ContactError> {
    let peer_data = PeerData {
        id: peer.id.clone(),
        name: peer.name.clone(),
        address: peer.get_full_address(),
        public_key: peer.public_key.clone(),
        connected_at: Utc::now(),
        last_seen: Utc::now(),
        bytes_sent: 0,
        bytes_received: 0,
    };

    let json_data = serde_json::to_string(&peer_data)
        .map_err(|e| ContactError::SerializationError(e.to_string()))?;

    let encoded = general_purpose::STANDARD.encode(json_data);
    Ok(format!("sg://{}", encoded))
}

pub fn parse_sg_link(sg_link: &str, current_peer_name: &str) -> Result<Contact, ContactError> {
    if !sg_link.starts_with("sg://") {
        return Err(ContactError::InvalidContact(
            "Invalid SG link format".to_string(),
        ));
    }

    let link_data = &sg_link[5..];

    let decoded_data = general_purpose::STANDARD
        .decode(link_data)
        .map_err(|e| ContactError::InvalidContact(format!("Decode error: {}", e)))?;

    let data_str = String::from_utf8(decoded_data)
        .map_err(|_| ContactError::InvalidContact("UTF-8 conversion failed".to_string()))?;

    let peer_data: PeerData = serde_json::from_str(&data_str)
        .map_err(|_| ContactError::InvalidContact("JSON parse failed".to_string()))?;

    if peer_data.name == current_peer_name {
        return Err(ContactError::InvalidContact(
            "Cannot add yourself as contact".to_string(),
        ));
    }

    let contact = Contact {
        id: peer_data.id,
        name: peer_data.name,
        address: peer_data.address,
        status: ContactStatus::Offline,
        trust_level: TrustLevel::Pending,
        last_seen: Some(peer_data.last_seen),
    };

    Ok(contact)
}