use crate::network::{Contact, ContactStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::error::ContactError;

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

    pub fn block_contact_with_reason(
        &mut self,
        contact_id: &str,
        reason: String,
    ) -> Result<(), ContactError> {
        if !self.contacts.contains_key(contact_id) {
            return Err(ContactError::ContactNotFound(format!(
                "Contact with ID {} not found",
                contact_id
            )));
        }

        let blocked_info = BlockedContactInfo {
            blocked_at: chrono::Utc::now(),
            reason,
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

    pub fn get_blocked_contacts(&self) -> Vec<Contact> {
        self.blocked_contacts
            .keys()
            .filter_map(|id| self.contacts.get(id).cloned())
            .collect()
    }

    pub fn get_blocked_contact_info(&self, contact_id: &str) -> Option<&BlockedContactInfo> {
        self.blocked_contacts.get(contact_id)
    }

    pub fn get_all_blocked_info(&self) -> HashMap<String, BlockedContactInfo> {
        self.blocked_contacts.clone()
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

    pub fn get_contacts_count(&self) -> usize {
        self.contacts.len()
    }

    pub fn get_blocked_count(&self) -> usize {
        self.blocked_contacts.len()
    }

    pub fn has_contact(&self, contact_id: &str) -> bool {
        self.contacts.contains_key(contact_id)
    }

    pub fn update_contact(&mut self, contact: Contact) -> Result<(), ContactError> {
        if !self.contacts.contains_key(&contact.id) {
            return Err(ContactError::ContactNotFound(format!(
                "Contact with ID {} not found",
                contact.id
            )));
        }

        self.contacts.insert(contact.id.clone(), contact);
        Ok(())
    }

    pub fn cleanup_old_blocked_contacts(&mut self, days: u32) -> u32 {
        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(days as i64);
        let mut removed_count = 0;

        let contacts_to_remove: Vec<String> = self
            .blocked_contacts
            .iter()
            .filter(|(_, info)| info.blocked_at < cutoff_date)
            .map(|(id, _)| id.clone())
            .collect();

        for contact_id in contacts_to_remove {
            if self.blocked_contacts.remove(&contact_id).is_some() {
                removed_count += 1;
            }
        }

        removed_count
    }

    pub fn bulk_block_contacts(&mut self, contact_ids: Vec<String>, reason: String) -> u32 {
        let mut blocked_count = 0;

        for contact_id in contact_ids {
            if self.contacts.contains_key(&contact_id) {
                let blocked_info = BlockedContactInfo {
                    blocked_at: chrono::Utc::now(),
                    reason: reason.clone(),
                    blocked_by_user: true,
                };

                self.blocked_contacts.insert(contact_id, blocked_info);
                blocked_count += 1;
            }
        }

        blocked_count
    }

    pub fn bulk_unblock_contacts(&mut self, contact_ids: Vec<String>) -> u32 {
        let mut unblocked_count = 0;

        for contact_id in contact_ids {
            if self.blocked_contacts.remove(&contact_id).is_some() {
                unblocked_count += 1;
            }
        }

        unblocked_count
    }

    pub fn get_contacts_by_criteria<F>(&self, predicate: F) -> Vec<Contact>
    where
        F: Fn(&Contact) -> bool,
    {
        self.contacts
            .values()
            .filter(|c| predicate(c))
            .cloned()
            .collect()
    }

    pub fn merge_contact_book(&mut self, other: &ContactBook) -> Result<MergeResult, ContactError> {
        let mut added = 0;
        let mut updated = 0;
        let mut conflicts = Vec::new();

        for (id, contact) in &other.contacts {
            if let Some(existing_contact) = self.contacts.get(id) {
                if existing_contact.name != contact.name
                    || existing_contact.address != contact.address
                {
                    conflicts.push(ContactMergeConflict {
                        contact_id: id.clone(),
                        existing_contact: existing_contact.clone(),
                        incoming_contact: contact.clone(),
                    });
                } else {
                    self.contacts.insert(id.clone(), contact.clone());
                    updated += 1;
                }
            } else {
                self.contacts.insert(id.clone(), contact.clone());
                added += 1;
            }
        }

        for (id, blocked_info) in &other.blocked_contacts {
            if !self.blocked_contacts.contains_key(id) {
                self.blocked_contacts
                    .insert(id.clone(), blocked_info.clone());
            }
        }

        Ok(MergeResult {
            contacts_added: added,
            contacts_updated: updated,
            conflicts,
            merge_time: chrono::Utc::now(),
        })
    }

    pub fn export_blocked_list(&self) -> HashMap<String, BlockedContactInfo> {
        self.blocked_contacts.clone()
    }

    pub fn import_blocked_list(
        &mut self,
        blocked_list: HashMap<String, BlockedContactInfo>,
    ) -> u32 {
        let mut imported_count = 0;

        for (contact_id, blocked_info) in blocked_list {
            if self.contacts.contains_key(&contact_id) {
                self.blocked_contacts.insert(contact_id, blocked_info);
                imported_count += 1;
            }
        }

        imported_count
    }

    pub fn validate_integrity(&self) -> Vec<String> {
        let mut issues = Vec::new();

        for (id, contact) in &self.contacts {
            if contact.id != *id {
                issues.push(format!("Contact ID mismatch: {} vs {}", contact.id, id));
            }

            if contact.name.trim().is_empty() {
                issues.push(format!("Contact {} has empty name", id));
            }

            if contact.address.trim().is_empty() {
                issues.push(format!("Contact {} has empty address", id));
            }
        }

        for blocked_id in self.blocked_contacts.keys() {
            if !self.contacts.contains_key(blocked_id) {
                issues.push(format!(
                    "Blocked contact {} not found in contacts",
                    blocked_id
                ));
            }
        }

        issues
    }
}

#[derive(Debug, Clone)]
pub struct ContactMergeConflict {
    pub contact_id: String,
    pub existing_contact: Contact,
    pub incoming_contact: Contact,
}

#[derive(Debug, Clone)]
pub struct MergeResult {
    pub contacts_added: u32,
    pub contacts_updated: u32,
    pub conflicts: Vec<ContactMergeConflict>,
    pub merge_time: chrono::DateTime<chrono::Utc>,
}
