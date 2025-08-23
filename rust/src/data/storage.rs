impl StorageManager {
    pub async fn delete_message(&mut self, message_id: &str) -> Result<(), StorageError> {
        let mut message_found = false;

        for (chat_id, messages) in self.chat_storage.messages.iter_mut() {
            if let Some(pos) = messages.iter().position(|m| m.id == message_id) {
                messages.remove(pos);
                message_found = true;
                break;
            }
        }

        if !message_found {
            return Err(StorageError::NotFound(format!(
                "Message {} not found",
                message_id
            )));
        }

        self.save_chats().await?;
        self.update_stats().await?;

        use crate::events::{AppEvent, StorageEvent};
        self.event_bus
            .emit(AppEvent::Storage(StorageEvent::ChatHistorySaved {
                chat_id: "various".to_string(),
                message_count: 1,
            }));

        Ok(())
    }

    pub async fn update_message_status(
        &mut self,
        message_id: &str,
        new_status: crate::network::DeliveryStatus,
    ) -> Result<(), StorageError> {
        let mut message_found = false;

        for (_, messages) in self.chat_storage.messages.iter_mut() {
            if let Some(message) = messages.iter_mut().find(|m| m.id == message_id) {
                message.delivery_status = new_status;
                message_found = true;
                break;
            }
        }

        if !message_found {
            return Err(StorageError::NotFound(format!(
                "Message {} not found",
                message_id
            )));
        }

        self.save_chats().await?;
        Ok(())
    }

    pub async fn get_messages_by_status(
        &self,
        chat_id: &str,
        status: crate::network::DeliveryStatus,
    ) -> Result<Vec<ChatMessage>, StorageError> {
        if let Some(messages) = self.chat_storage.messages.get(chat_id) {
            let filtered_messages: Vec<ChatMessage> = messages
                .iter()
                .filter(|m| m.delivery_status == status)
                .cloned()
                .collect();
            Ok(filtered_messages)
        } else {
            Ok(vec![])
        }
    }

    pub async fn get_failed_messages(&self) -> Result<Vec<ChatMessage>, StorageError> {
        let mut failed_messages = Vec::new();

        for (_, messages) in &self.chat_storage.messages {
            for message in messages {
                if matches!(
                    message.delivery_status,
                    crate::network::DeliveryStatus::Failed
                ) {
                    failed_messages.push(message.clone());
                }
            }
        }

        Ok(failed_messages)
    }

    pub async fn cleanup_old_messages(&mut self, days: u32) -> Result<u32, StorageError> {
        let cutoff_time = chrono::Utc::now().timestamp() as u64 - (days as u64 * 24 * 60 * 60);
        let mut removed_count = 0;

        for (_, messages) in self.chat_storage.messages.iter_mut() {
            let original_len = messages.len();
            messages.retain(|m| m.timestamp >= cutoff_time);
            removed_count += (original_len - messages.len()) as u32;
        }

        if removed_count > 0 {
            self.save_chats().await?;
            self.update_stats().await?;

            use crate::events::{AppEvent, StorageEvent};
            self.event_bus
                .emit(AppEvent::Storage(StorageEvent::CleanupCompleted {
                    removed_items: removed_count as usize,
                }));
        }

        Ok(removed_count)
    }

    pub async fn get_chat_size(&self, chat_id: &str) -> Result<u64, StorageError> {
        if let Some(messages) = self.chat_storage.messages.get(chat_id) {
            let size = serde_json::to_string(messages)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?
                .len() as u64;
            Ok(size)
        } else {
            Ok(0)
        }
    }

    pub async fn optimize_storage(&mut self) -> Result<StorageOptimizationResult, StorageError> {
        let original_size = self.stats.storage_size_bytes;

        self.compact_messages().await?;
        self.remove_duplicate_messages().await?;
        self.update_stats().await?;

        let new_size = self.stats.storage_size_bytes;
        let space_saved = original_size.saturating_sub(new_size);

        Ok(StorageOptimizationResult {
            original_size_bytes: original_size,
            optimized_size_bytes: new_size,
            space_saved_bytes: space_saved,
            messages_deduplicated: 0,
            optimization_time: chrono::Utc::now(),
        })
    }

    async fn compact_messages(&mut self) -> Result<(), StorageError> {
        for (_, messages) in self.chat_storage.messages.iter_mut() {
            messages.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));
        }
        Ok(())
    }

    async fn remove_duplicate_messages(&mut self) -> Result<u32, StorageError> {
        let mut removed_count = 0;

        for (_, messages) in self.chat_storage.messages.iter_mut() {
            let mut seen_ids = std::collections::HashSet::new();
            let original_len = messages.len();

            messages.retain(|message| {
                if seen_ids.contains(&message.id) {
                    false
                } else {
                    seen_ids.insert(message.id.clone());
                    true
                }
            });

            removed_count += (original_len - messages.len()) as u32;
        }

        if removed_count > 0 {
            self.save_chats().await?;
        }

        Ok(removed_count)
    }

    pub async fn export_chat_data(
        &self,
        chat_id: &str,
        format: &str,
    ) -> Result<String, StorageError> {
        let messages = self.get_messages(chat_id).await?;

        match format.to_lowercase().as_str() {
            "json" => serde_json::to_string_pretty(&messages)
                .map_err(|e| StorageError::SerializationError(e.to_string())),
            "csv" => {
                let mut csv_data = String::from("timestamp,from,to,content,type,status\n");
                for message in messages {
                    csv_data.push_str(&format!(
                        "{},{},{},{},{:?},{:?}\n",
                        message.timestamp,
                        message.from,
                        message.to,
                        message.content.replace(',', ";"),
                        message.msg_type,
                        message.delivery_status
                    ));
                }
                Ok(csv_data)
            }
            "txt" => {
                let mut text_data = String::new();
                for message in messages {
                    let time = chrono::DateTime::from_timestamp(message.timestamp as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                        .unwrap_or_else(|| "Unknown time".to_string());

                    text_data.push_str(&format!(
                        "[{}] {}: {}\n",
                        time, message.from, message.content
                    ));
                }
                Ok(text_data)
            }
            _ => Err(StorageError::SerializationError(
                "Unsupported format".to_string(),
            )),
        }
    }

    pub async fn import_chat_data(
        &mut self,
        chat_id: &str,
        data: &str,
        format: &str,
    ) -> Result<u32, StorageError> {
        let imported_messages: Vec<ChatMessage> = match format.to_lowercase().as_str() {
            "json" => serde_json::from_str(data)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?,
            _ => {
                return Err(StorageError::SerializationError(
                    "Unsupported import format".to_string(),
                ))
            }
        };

        let import_count = imported_messages.len() as u32;

        for message in imported_messages {
            self.save_message(chat_id, &message).await?;
        }

        Ok(import_count)
    }

    pub async fn get_storage_health(&self) -> Result<StorageHealth, StorageError> {
        let issues = self.validate_all_data().await?;
        let total_size = self.stats.storage_size_bytes;
        let fragmentation = self.calculate_fragmentation().await?;

        Ok(StorageHealth {
            is_healthy: issues.is_empty(),
            total_size_bytes: total_size,
            fragmentation_percent: fragmentation,
            corruption_issues: issues.len() as u32,
            last_check: chrono::Utc::now(),
            recommendations: self.get_health_recommendations(fragmentation, &issues),
        })
    }

    async fn validate_all_data(&self) -> Result<Vec<String>, StorageError> {
        let mut issues = Vec::new();

        issues.extend(self.validate_contacts().await?);
        issues.extend(self.validate_chats().await?);

        Ok(issues)
    }

    async fn calculate_fragmentation(&self) -> Result<f64, StorageError> {
        let mut total_messages = 0;
        let mut fragmented_chats = 0;

        for (_, messages) in &self.chat_storage.messages {
            total_messages += messages.len();

            let mut timestamps: Vec<u64> = messages.iter().map(|m| m.timestamp).collect();
            timestamps.sort();

            let mut gaps = 0;
            for window in timestamps.windows(2) {
                if window[1] - window[0] > 3600 {
                    gaps += 1;
                }
            }

            if gaps > messages.len() / 10 {
                fragmented_chats += 1;
            }
        }

        if self.chat_storage.messages.len() > 0 {
            Ok((fragmented_chats as f64 / self.chat_storage.messages.len() as f64) * 100.0)
        } else {
            Ok(0.0)
        }
    }

    fn get_health_recommendations(&self, fragmentation: f64, issues: &[String]) -> Vec<String> {
        let mut recommendations = Vec::new();

        if fragmentation > 30.0 {
            recommendations
                .push("Consider running storage optimization to reduce fragmentation".to_string());
        }

        if !issues.is_empty() {
            recommendations.push("Fix data corruption issues found during validation".to_string());
        }

        if self.stats.storage_size_bytes > 100 * 1024 * 1024 {
            recommendations
                .push("Consider archiving old messages to reduce storage size".to_string());
        }

        if recommendations.is_empty() {
            recommendations.push("Storage is healthy, no actions needed".to_string());
        }

        recommendations
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageOptimizationResult {
    pub original_size_bytes: u64,
    pub optimized_size_bytes: u64,
    pub space_saved_bytes: u64,
    pub messages_deduplicated: u32,
    pub optimization_time: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageHealth {
    pub is_healthy: bool,
    pub total_size_bytes: u64,
    pub fragmentation_percent: f64,
    pub corruption_issues: u32,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub recommendations: Vec<String>,
}
