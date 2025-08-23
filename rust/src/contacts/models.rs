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
