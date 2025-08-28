// Добавить эти методы в struct ContactManager для поддержки batch-операций

impl ContactManager {
    // ... существующие методы ...

    // Метод для пакетной блокировки контактов
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

    // Метод для обновления активности контакта
    pub async fn update_contact_activity(&self, contact_id: &str) -> Result<(), ContactError> {
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

    // Метод для получения статистики взаимодействия с контактом
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

    // Метод для очистки заблокированных контактов
    pub async fn cleanup_blocked_contacts(&self, days: u32) -> Result<u32, ContactError> {
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

    // Метод для сохранения контактов
    pub async fn save_contacts(&self) -> Result<(), ContactError> {
        // Реализация зависит от вашей конкретной логики сохранения
        // Пример:
        // self.storage.save_contacts(&self.contact_book).await
        //     .map_err(|e| ContactError::StorageError(e.to_string()))

        // Заглушка для примера
        Ok(())
    }

    // Метод для загрузки контактов
    pub async fn load_contacts(&self) -> Result<(), ContactError> {
        // Реализация зависит от вашей конкретной логики загрузки
        // Пример:
        // let loaded_contacts = self.storage.load_contacts().await
        //     .map_err(|e| ContactError::StorageError(e.to_string()))?;
        // self.contact_book = loaded_contacts;

        // Заглушка для примера
        Ok(())
    }
}
