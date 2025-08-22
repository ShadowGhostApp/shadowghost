use crate::core::peer::Peer;
use crate::network::{Contact, ContactStatus, PeerData, TrustLevel};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt;

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
pub struct ContactStats {
    pub total_contacts: usize,
    pub online_contacts: usize,
    pub trusted_contacts: usize,
    pub blocked_contacts: usize,
    pub pending_contacts: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactBook {
    contacts: HashMap<String, Contact>,
    blocked_contacts: HashMap<String, bool>,
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

        self.blocked_contacts.insert(contact_id.to_string(), true);
        Ok(())
    }

    pub fn unblock_contact(&mut self, contact_id: &str) -> Result<(), ContactError> {
        self.blocked_contacts.remove(contact_id);
        Ok(())
    }

    pub fn is_blocked(&self, contact_id: &str) -> bool {
        self.blocked_contacts
            .get(contact_id)
            .copied()
            .unwrap_or(false)
    }
}

pub struct ContactManager {
    peer: Peer,
    contact_book: ContactBook,
    storage_path: Option<String>,
}

impl ContactManager {
    pub fn new(peer: Peer) -> Self {
        Self {
            peer,
            contact_book: ContactBook::new(),
            storage_path: None,
        }
    }

    pub fn new_with_storage(storage_path: String) -> Result<Self, ContactError> {
        let peer = Peer::new("default_user".to_string(), "127.0.0.1:8080".to_string());

        Ok(Self {
            peer,
            contact_book: ContactBook::new(),
            storage_path: Some(storage_path),
        })
    }

    pub async fn load_contacts(&mut self) -> Result<(), ContactError> {
        if let Some(ref storage_path) = self.storage_path {
            match tokio::fs::read_to_string(storage_path).await {
                Ok(data) => match serde_json::from_str::<ContactBook>(&data) {
                    Ok(book) => {
                        self.contact_book = book;
                        Ok(())
                    }
                    Err(e) => Err(ContactError::SerializationError(e.to_string())),
                },
                Err(_) => Ok(()),
            }
        } else {
            Ok(())
        }
    }

    // Remove the duplicate load_contacts method - keeping only the one above
    pub async fn save_contacts(&self) -> Result<(), ContactError> {
        if let Some(ref storage_path) = self.storage_path {
            let data = serde_json::to_string_pretty(&self.contact_book)
                .map_err(|e| ContactError::SerializationError(e.to_string()))?;

            tokio::fs::write(storage_path, data)
                .await
                .map_err(|e| ContactError::IoError(e.to_string()))?;
        }
        Ok(())
    }

    pub async fn generate_sg_link(&self) -> Result<String, ContactError> {
        let peer_data = PeerData {
            id: self.peer.id.clone(),
            name: self.peer.name.clone(),
            address: self.peer.get_full_address(),
            public_key: self.peer.public_key.clone(),
            connected_at: Utc::now(),
            last_seen: Utc::now(),
            bytes_sent: 0,
            bytes_received: 0,
        };

        let json_data = serde_json::to_string(&peer_data)
            .map_err(|e| ContactError::SerializationError(e.to_string()))?;

        use base64::{Engine as _, engine::general_purpose};
        let encoded = general_purpose::STANDARD.encode(json_data);
        Ok(format!("sg://{}", encoded))
    }

    pub async fn add_contact_by_sg_link(&self, sg_link: &str) -> Result<Contact, ContactError> {
        if !sg_link.starts_with("sg://") {
            return Err(ContactError::InvalidContact(
                "Invalid SG link format".to_string(),
            ));
        }

        let link_data = &sg_link[5..];

        use base64::{Engine as _, engine::general_purpose};
        let decoded_data = general_purpose::STANDARD
            .decode(link_data)
            .map_err(|e| ContactError::InvalidContact(format!("Decode error: {}", e)))?;

        let data_str = String::from_utf8(decoded_data)
            .map_err(|_| ContactError::InvalidContact("UTF-8 conversion failed".to_string()))?;

        let peer_data: PeerData = serde_json::from_str(&data_str)
            .map_err(|_| ContactError::InvalidContact("JSON parse failed".to_string()))?;

        if peer_data.name == self.peer.name {
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

    pub fn add_contact_from_peer_data(&mut self, peer_data: &PeerData) -> Result<(), ContactError> {
        let contact = Contact {
            id: peer_data.id.clone(),
            name: peer_data.name.clone(),
            address: peer_data.address.clone(),
            status: ContactStatus::Offline,
            trust_level: TrustLevel::Pending,
            last_seen: Some(peer_data.last_seen),
        };

        self.contact_book.add_contact(contact)
    }

    pub fn add_contact(&mut self, contact: Contact) -> Result<(), ContactError> {
        self.contact_book.add_contact(contact)
    }

    pub async fn get_contacts(&self) -> Vec<Contact> {
        self.contact_book.get_contacts()
    }

    pub async fn get_contact_by_name(&self, name: &str) -> Option<Contact> {
        self.contact_book
            .get_contacts()
            .into_iter()
            .find(|c| c.name == name)
    }

    pub async fn get_contact_by_id(&self, contact_id: &str) -> Option<Contact> {
        self.contact_book.get_contact(contact_id).cloned()
    }

    pub fn get_contact(&self, contact_id: &str) -> Option<Contact> {
        self.contact_book.get_contact(contact_id).cloned()
    }

    pub fn get_contact_cloned(&self, contact_id: &str) -> Option<Contact> {
        self.contact_book.get_contact(contact_id).cloned()
    }

    pub fn contact_exists(&self, contact_id: &str) -> bool {
        self.contact_book.contacts.contains_key(contact_id)
    }

    pub fn get_contact_by_name_ref(&self, name: &str) -> Option<&Contact> {
        self.contact_book.contacts.values().find(|c| c.name == name)
    }

    pub fn get_blocked_contacts(&self) -> Vec<Contact> {
        self.contact_book
            .blocked_contacts
            .keys()
            .filter_map(|id| self.contact_book.get_contact(id))
            .cloned()
            .collect()
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
        let blocked_count = self.contact_book.blocked_contacts.len();

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

    pub fn remove_contact(&mut self, contact_id: &str) -> Result<(), ContactError> {
        self.contact_book.remove_contact(contact_id)
    }

    pub fn update_contact_status(
        &mut self,
        contact_id: &str,
        status: ContactStatus,
    ) -> Result<(), ContactError> {
        self.contact_book.update_contact_status(contact_id, status)
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

    pub fn search_contacts(&self, query: &str) -> Vec<Contact> {
        let query = query.to_lowercase();
        self.contact_book
            .get_contacts()
            .into_iter()
            .filter(|contact| {
                contact.name.to_lowercase().contains(&query)
                    || contact.address.to_lowercase().contains(&query)
            })
            .collect()
    }

    pub fn get_trusted_contacts(&self) -> Vec<Contact> {
        self.contact_book
            .get_contacts()
            .into_iter()
            .filter(|contact| matches!(contact.trust_level, TrustLevel::Trusted))
            .collect()
    }

    pub fn get_online_contacts(&self) -> Vec<Contact> {
        self.contact_book
            .get_contacts()
            .into_iter()
            .filter(|contact| matches!(contact.status, ContactStatus::Online))
            .collect()
    }

    pub fn update_last_seen(&mut self, contact_id: &str) -> Result<(), ContactError> {
        if let Some(contact) = self.contact_book.contacts.get_mut(contact_id) {
            contact.last_seen = Some(Utc::now());
            Ok(())
        } else {
            Err(ContactError::ContactNotFound(format!(
                "Contact with ID {} not found",
                contact_id
            )))
        }
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

    pub fn get_contact_count(&self) -> usize {
        self.contact_book.contacts.len()
    }

    pub fn clear_all_contacts(&mut self) {
        self.contact_book.contacts.clear();
        self.contact_book.blocked_contacts.clear();
    }

    pub fn export_contacts(&self) -> Result<String, ContactError> {
        serde_json::to_string_pretty(&self.contact_book)
            .map_err(|e| ContactError::SerializationError(e.to_string()))
    }

    pub fn import_contacts(&mut self, data: &str) -> Result<usize, ContactError> {
        let imported_book: ContactBook = serde_json::from_str(data)
            .map_err(|e| ContactError::SerializationError(e.to_string()))?;

        let mut imported_count = 0;
        for contact in imported_book.contacts.values() {
            if !self.contact_book.contacts.contains_key(&contact.id) {
                self.contact_book
                    .contacts
                    .insert(contact.id.clone(), contact.clone());
                imported_count += 1;
            }
        }

        Ok(imported_count)
    }

    pub fn create_contact_from_sg_link(&self, sg_link_data: &str) -> Result<Contact, ContactError> {
        use base64::{Engine as _, engine::general_purpose};
        let decoded_data = general_purpose::STANDARD
            .decode(sg_link_data)
            .map_err(|e| ContactError::InvalidContact(format!("Decode error: {}", e)))?;

        let data_str = String::from_utf8(decoded_data)
            .map_err(|_| ContactError::InvalidContact("UTF-8 conversion failed".to_string()))?;

        let peer_data: PeerData = serde_json::from_str(&data_str)
            .map_err(|_| ContactError::InvalidContact("JSON parse failed".to_string()))?;

        Ok(Contact {
            id: peer_data.id,
            name: peer_data.name,
            address: peer_data.address,
            status: ContactStatus::Offline,
            trust_level: TrustLevel::Pending,
            last_seen: Some(peer_data.last_seen),
        })
    }
}
