use crate::core::peer::Peer;
use crate::network::{Contact, ContactStatus, PeerData, TrustLevel};
use chrono::Utc;

use super::error::ContactError;
use super::models::{ContactBook, ContactStats};
use super::operations;

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
        operations::generate_sg_link(&self.peer)
    }

    pub async fn add_contact_by_sg_link(&self, sg_link: &str) -> Result<Contact, ContactError> {
        operations::parse_sg_link(sg_link, &self.peer.name)
    }

    pub fn add_contact_from_peer_data(&mut self, peer_data: &PeerData) -> Result<(), ContactError> {
        let contact = operations::create_contact_from_peer_data(peer_data);
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
        self.contact_book.get_contact(contact_id).is_some()
    }

    pub fn get_contact_by_name_ref(&self, name: &str) -> Option<&Contact> {
        self.contact_book
            .get_contacts()
            .iter()
            .find(|c| c.name == name);
    }

    pub fn get_blocked_contacts(&self) -> Vec<Contact> {
        self.contact_book
            .get_contacts()
            .into_iter()
            .filter(|c| self.contact_book.is_blocked(&c.id))
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
        if let Some(contact) = self.contact_book.get_contact(contact_id) {
            // Обновляем последнее время контакта
            // Это потребует изменения в ContactBook, так как get_contact не даёт мутабельный доступ
            // Для простоты, мы просто удаляем и вставляем обновлённый контакт
            let mut updated_contact = contact.clone();
            updated_contact.last_seen = Some(Utc::now());
            self.contact_book.add_contact(updated_contact)?;
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
        if let Some(contact) = self.contact_book.get_contact(contact_id) {
            // Подобно update_last_seen, обновляем уровень доверия
            let mut updated_contact = contact.clone();
            updated_contact.trust_level = trust_level;
            self.contact_book.add_contact(updated_contact)?;
            Ok(())
        } else {
            Err(ContactError::ContactNotFound(format!(
                "Contact with ID {} not found",
                contact_id
            )))
        }
    }

    pub fn get_contact_count(&self) -> usize {
        self.contact_book.get_contacts().len()
    }

    pub fn clear_all_contacts(&mut self) {
        self.contact_book = ContactBook::new();
    }

    pub fn export_contacts(&self) -> Result<String, ContactError> {
        serde_json::to_string_pretty(&self.contact_book)
            .map_err(|e| ContactError::SerializationError(e.to_string()))
    }

    pub fn import_contacts(&mut self, data: &str) -> Result<usize, ContactError> {
        let imported_book: ContactBook = serde_json::from_str(data)
            .map_err(|e| ContactError::SerializationError(e.to_string()))?;

        let mut imported_count = 0;
        for contact in imported_book.get_contacts() {
            if !self.contact_exists(&contact.id) {
                self.contact_book.add_contact(contact.clone())?;
                imported_count += 1;
            }
        }

        Ok(imported_count)
    }

    pub fn create_contact_from_sg_link(&self, sg_link_data: &str) -> Result<Contact, ContactError> {
        use base64::{engine::general_purpose, Engine as _};
        let decoded_data = general_purpose::STANDARD
            .decode(sg_link_data)
            .map_err(|e| ContactError::InvalidContact(format!("Decode error: {}", e)))?;

        let data_str = String::from_utf8(decoded_data)
            .map_err(|_| ContactError::InvalidContact("UTF-8 conversion failed".to_string()))?;

        let peer_data: PeerData = serde_json::from_str(&data_str)
            .map_err(|_| ContactError::InvalidContact("JSON parse failed".to_string()))?;

        Ok(operations::create_contact_from_peer_data(&peer_data))
    }
}
