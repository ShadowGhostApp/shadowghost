use crate::contacts::{
    ContactError, ContactInteractionStats, ContactIssueType, ContactValidationIssue, IssueSeverity,
};
use crate::network::{Contact, ContactStatus, TrustLevel};
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ContactStats {
    pub total_contacts: usize,
    pub online_contacts: usize,
    pub trusted_contacts: usize,
    pub blocked_contacts: usize,
    pub pending_contacts: usize,
}

#[derive(Debug, Clone)]
pub struct ContactBook {
    pub contacts: HashMap<String, Contact>,
    pub blocked_contacts: HashMap<String, BlockedContactInfo>,
}

#[derive(Debug, Clone)]
pub struct BlockedContactInfo {
    pub blocked_at: chrono::DateTime<chrono::Utc>,
    pub reason: String,
    pub blocked_by_user: bool,
}

pub struct ContactManager {
    contact_book: ContactBook,
    data_path: PathBuf,
}

impl ContactManager {
    pub fn new(data_path: &Path) -> Result<Self, ContactError> {
        Ok(Self {
            contact_book: ContactBook::new(),
            data_path: data_path.to_path_buf(),
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
        let contacts = self.get_contacts();
        let total = contacts.len();
        let online = contacts
            .iter()
            .filter(|c| matches!(c.status, ContactStatus::Online))
            .count();
        let trusted = contacts
            .iter()
            .filter(|c| matches!(c.trust_level, TrustLevel::Trusted))
            .count();
        let blocked = self.contact_book.get_blocked_count();
        let pending = contacts
            .iter()
            .filter(|c| matches!(c.trust_level, TrustLevel::Pending))
            .count();

        ContactStats {
            total_contacts: total,
            online_contacts: online,
            trusted_contacts: trusted,
            blocked_contacts: blocked,
            pending_contacts: pending,
        }
    }

    pub fn find_contacts_by_name(&self, name: &str) -> Vec<Contact> {
        self.contact_book.find_contacts_by_name(name)
    }

    pub fn find_contacts_by_address(&self, address: &str) -> Vec<Contact> {
        self.contact_book.find_contacts_by_address(address)
    }

    pub fn get_contacts_by_trust_level(&self, trust_level: TrustLevel) -> Vec<Contact> {
        self.get_contacts()
            .into_iter()
            .filter(|c| c.trust_level == trust_level)
            .collect()
    }

    pub fn get_contacts_by_status(&self, status: ContactStatus) -> Vec<Contact> {
        self.get_contacts()
            .into_iter()
            .filter(|c| c.status == status)
            .collect()
    }

    pub async fn batch_block_contacts(
        &self,
        contact_ids: Vec<String>,
    ) -> Result<u32, ContactError> {
        let mut blocked_count = 0;
        for contact_id in contact_ids {
            if self.block_contact(&contact_id).is_ok() {
                blocked_count += 1;
            }
        }
        if blocked_count > 0 {
            // Если нужно сохранить изменения в хранилище после массовой блокировки
            // Предполагается, что у нас есть метод save_contacts
            self.save_contacts().await?;
        }
        Ok(blocked_count)
    }

    // Метод для пакетной разблокировки контактов
    pub async fn batch_unblock_contacts(
        &self,
        contact_ids: Vec<String>,
    ) -> Result<u32, ContactError> {
        let mut unblocked_count = 0;
        for contact_id in contact_ids {
            if self.unblock_contact(&contact_id).is_ok() {
                unblocked_count += 1;
            }
        }
        if unblocked_count > 0 {
            // Если нужно сохранить изменения в хранилище после массовой разблокировки
            self.save_contacts().await?;
        }
        Ok(unblocked_count)
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

    // Метод для пометки контактов как offline
    pub async fn mark_contacts_offline(
        &self,
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

    pub async fn cleanup_blocked_contacts(&mut self, days: u32) -> Result<u32, ContactError> {
        let cutoff_time = chrono::Utc::now() - chrono::Duration::days(days as i64);
        let all_contacts = self.get_contacts();
        let mut cleanup_count = 0;

        let contacts_to_remove: Vec<String> = all_contacts
            .iter()
            .filter(|contact| {
                if self.is_contact_blocked(&contact.id) {
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
            if self.remove_contact(&contact_id).is_ok() {
                cleanup_count += 1;
            }
        }

        if cleanup_count > 0 {
            self.save_contacts().await?;
        }

        Ok(cleanup_count)
    }

    // Проверка валидности данных контактов
    pub fn validate_contact_data(&self) -> Vec<ContactValidationIssue> {
        let mut issues = Vec::new();
        let contacts = self.get_contacts();

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

    // Вспомогательный метод для проверки валидности адреса
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
        Ok(())
    }

    pub async fn load_contacts(&self) -> Result<(), ContactError> {
        Ok(())
    }
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

    pub fn get_blocked_count(&self) -> usize {
        self.blocked_contacts.len()
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

pub fn generate_sg_link(peer: &crate::core::Peer) -> Result<String, ContactError> {
    use crate::network::PeerData;
    use base64::{engine::general_purpose, Engine as _};

    let peer_data = PeerData {
        id: peer.id.clone(),
        name: peer.name.clone(),
        address: peer.get_full_address(),
        public_key: peer.public_key.clone(),
        connected_at: chrono::Utc::now(),
        last_seen: chrono::Utc::now(),
        bytes_sent: 0,
        bytes_received: 0,
    };

    let json_data = serde_json::to_string(&peer_data)
        .map_err(|e| ContactError::SerializationError(e.to_string()))?;

    let encoded = general_purpose::STANDARD.encode(json_data);
    Ok(format!("sg://{}", encoded))
}

pub fn parse_sg_link(sg_link: &str, current_peer_name: &str) -> Result<Contact, ContactError> {
    use crate::network::PeerData;
    use base64::{engine::general_purpose, Engine as _};

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
