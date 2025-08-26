impl ContactManager {
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
        if let Some(contact) = self.contact_book.get_contact(contact_id) {
            let mut updated_contact = contact.clone();
            updated_contact.last_seen = Some(chrono::Utc::now());
            updated_contact.status = ContactStatus::Online;
            self.contact_book.add_contact(updated_contact)?;
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
        let all_contacts = self.contact_book.get_contacts();
        let mut updated_count = 0;

        for contact in all_contacts {
            if matches!(contact.status, ContactStatus::Online) {
                if let Some(last_seen) = contact.last_seen {
                    if last_seen < cutoff_time {
                        let mut updated_contact = contact.clone();
                        updated_contact.status = ContactStatus::Offline;
                        self.contact_book.add_contact(updated_contact)?;
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

    pub async fn batch_update_trust_level(
        &mut self,
        contact_ids: Vec<String>,
        trust_level: TrustLevel,
    ) -> Result<u32, ContactError> {
        let mut updated_count = 0;

        for contact_id in contact_ids {
            if self
                .set_trust_level(&contact_id, trust_level.clone())
                .is_ok()
            {
                updated_count += 1;
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

    pub async fn optimize_contact_storage(
        &mut self,
    ) -> Result<ContactOptimizationResult, ContactError> {
        let original_count = self.contact_book.get_contacts().len();

        let duplicate_count = self.remove_duplicate_contacts().await?;
        let orphaned_blocked_count = self.cleanup_orphaned_blocked_entries().await?;

        let final_count = self.contact_book.get_contacts().len();

        if duplicate_count > 0 || orphaned_blocked_count > 0 {
            self.save_contacts().await?;
        }

        Ok(ContactOptimizationResult {
            original_contact_count: original_count,
            final_contact_count: final_count,
            duplicates_removed: duplicate_count,
            orphaned_blocks_cleaned: orphaned_blocked_count,
            optimization_time: chrono::Utc::now(),
        })
    }

    async fn remove_duplicate_contacts(&mut self) -> Result<u32, ContactError> {
        let contacts = self.contact_book.get_contacts();
        let mut seen_addresses = std::collections::HashSet::new();
        let mut duplicates_to_remove = Vec::new();

        for contact in contacts {
            let key = format!("{}:{}", contact.name.to_lowercase(), contact.address);
            if seen_addresses.contains(&key) {
                duplicates_to_remove.push(contact.id);
            } else {
                seen_addresses.insert(key);
            }
        }

        let duplicate_count = duplicates_to_remove.len() as u32;
        for contact_id in duplicates_to_remove {
            let _ = self.contact_book.remove_contact(&contact_id);
        }

        Ok(duplicate_count)
    }

    async fn cleanup_orphaned_blocked_entries(&mut self) -> Result<u32, ContactError> {
        let blocked_ids = self.contact_book.get_blocked_contact_ids();
        let existing_contacts: std::collections::HashSet<String> = self
            .contact_book
            .get_contacts()
            .into_iter()
            .map(|c| c.id)
            .collect();

        let orphaned_blocks: Vec<String> = blocked_ids
            .into_iter()
            .filter(|id| !existing_contacts.contains(id))
            .collect();

        let orphaned_count = orphaned_blocks.len() as u32;
        for id in orphaned_blocks {
            let _ = self.contact_book.unblock_contact(&id);
        }

        Ok(orphaned_count)
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
}

#[derive(Debug, Clone)]
pub struct ContactInteractionStats {
    pub contact_id: String,
    pub days_since_last_seen: u32,
    pub is_trusted: bool,
    pub is_blocked: bool,
    pub current_status: ContactStatus,
    pub trust_level: TrustLevel,
}

#[derive(Debug, Clone)]
pub struct ContactOptimizationResult {
    pub original_contact_count: usize,
    pub final_contact_count: usize,
    pub duplicates_removed: u32,
    pub orphaned_blocks_cleaned: u32,
    pub optimization_time: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone)]
pub struct ContactValidationIssue {
    pub contact_id: String,
    pub issue_type: ContactIssueType,
    pub description: String,
    pub severity: IssueSeverity,
}

#[derive(Debug, Clone)]
pub enum ContactIssueType {
    EmptyName,
    EmptyAddress,
    InvalidAddress,
    DuplicateContact,
    OrphanedBlock,
}

#[derive(Debug, Clone)]
pub enum IssueSeverity {
    Low,
    Medium,
    High,
    Critical,
}
