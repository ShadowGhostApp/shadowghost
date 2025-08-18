use crate::config::AppConfig;
use crate::events::{EventBus, StorageEvent};
use crate::network::{ChatMessage, Contact, ContactStatus};
use futures::Future;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::pin::Pin;
use tokio::fs as async_fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactDatabase {
    pub contacts: HashMap<String, Contact>,
    pub last_updated: u64,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatHistory {
    pub messages: Vec<ChatMessage>,
    pub participants: Vec<String>,
    pub created_at: u64,
    pub last_message_at: u64,
}

#[derive(Debug)]
pub struct StorageManager {
    config: AppConfig,
    data_dir: PathBuf,
    event_bus: EventBus,
}

impl StorageManager {
    pub fn new(config: AppConfig, event_bus: EventBus) -> Result<Self, Box<dyn std::error::Error>> {
        let data_dir = config.storage.data_dir.clone();

        fs::create_dir_all(&data_dir)?;
        fs::create_dir_all(data_dir.join("contacts"))?;
        fs::create_dir_all(data_dir.join("chats"))?;
        fs::create_dir_all(data_dir.join("keys"))?;
        fs::create_dir_all(data_dir.join("backups"))?;

        Ok(Self {
            config,
            data_dir,
            event_bus,
        })
    }

    pub async fn save_contacts(
        &self,
        contacts: &HashMap<String, Contact>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let database = ContactDatabase {
            contacts: contacts.clone(),
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            version: "1.0".to_string(),
        };

        let json_data = serde_json::to_string_pretty(&database)?;
        let contacts_file = self.data_dir.join("contacts").join("contacts.json");

        if contacts_file.exists() {
            let backup_file = self
                .data_dir
                .join("backups")
                .join(format!("contacts_backup_{}.json", database.last_updated));

            if let Err(e) = tokio::fs::copy(&contacts_file, &backup_file).await {
                self.event_bus.emit_storage(StorageEvent::Error {
                    error: e.to_string(),
                    operation: "backup_contacts".to_string(),
                });
            }
        }

        async_fs::write(&contacts_file, json_data).await?;

        self.event_bus.emit_storage(StorageEvent::ContactsSaved {
            count: contacts.len(),
        });

        Ok(())
    }

    pub async fn load_contacts(
        &self,
    ) -> Result<HashMap<String, Contact>, Box<dyn std::error::Error>> {
        let contacts_file = self.data_dir.join("contacts").join("contacts.json");

        if !contacts_file.exists() {
            self.event_bus
                .emit_storage(StorageEvent::ContactsLoaded { count: 0 });
            return Ok(HashMap::new());
        }

        let json_data = async_fs::read_to_string(&contacts_file).await?;
        let database: ContactDatabase = serde_json::from_str(&json_data)?;

        self.event_bus.emit_storage(StorageEvent::ContactsLoaded {
            count: database.contacts.len(),
        });

        Ok(database.contacts)
    }

    pub fn save_chat_history<'a>(
        &'a self,
        chat_id: &'a str,
        messages: &'a [ChatMessage],
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'a>> {
        Box::pin(async move {
            if messages.is_empty() {
                return Ok(());
            }

            let participants: Vec<String> = messages
                .iter()
                .flat_map(|m| vec![m.from.clone(), m.to.clone()])
                .collect::<std::collections::HashSet<_>>()
                .into_iter()
                .collect();

            let history = ChatHistory {
                messages: messages.to_vec(),
                participants,
                created_at: messages.first().unwrap().timestamp,
                last_message_at: messages.last().unwrap().timestamp,
            };

            let json_data = serde_json::to_string_pretty(&history)?;
            let chat_file = self
                .data_dir
                .join("chats")
                .join(format!("{}.json", chat_id));

            async_fs::write(&chat_file, json_data).await?;

            self.event_bus.emit_storage(StorageEvent::ChatHistorySaved {
                chat_id: chat_id.to_string(),
                message_count: messages.len(),
            });

            Ok(())
        })
    }

    pub async fn save_chat_history_with_cleanup(
        &self,
        chat_id: &str,
        messages: &[ChatMessage],
    ) -> Result<(), Box<dyn std::error::Error>> {
        if messages.is_empty() {
            return Ok(());
        }

        let cleaned_messages = self.apply_cleanup_rules(messages);

        let participants: Vec<String> = cleaned_messages
            .iter()
            .flat_map(|m| vec![m.from.clone(), m.to.clone()])
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        let history = ChatHistory {
            messages: cleaned_messages.clone(),
            participants,
            created_at: cleaned_messages.first().unwrap().timestamp,
            last_message_at: cleaned_messages.last().unwrap().timestamp,
        };

        let json_data = serde_json::to_string_pretty(&history)?;
        let chat_file = self
            .data_dir
            .join("chats")
            .join(format!("{}.json", chat_id));

        async_fs::write(&chat_file, json_data).await?;

        let original_count = messages.len();
        let final_count = cleaned_messages.len();

        if final_count != original_count {
            let removed = original_count - final_count;
            self.event_bus.emit_storage(StorageEvent::CleanupCompleted {
                removed_items: removed,
            });
        }

        self.event_bus.emit_storage(StorageEvent::ChatHistorySaved {
            chat_id: chat_id.to_string(),
            message_count: final_count,
        });

        Ok(())
    }

    fn apply_cleanup_rules(&self, messages: &[ChatMessage]) -> Vec<ChatMessage> {
        let mut cleaned = messages.to_vec();

        let cutoff_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0)
            - (self.config.storage.auto_cleanup_days as u64 * 24 * 60 * 60);

        cleaned.retain(|msg| msg.timestamp > cutoff_time);

        if cleaned.len() > self.config.storage.max_chat_history as usize {
            let excess = cleaned.len() - self.config.storage.max_chat_history as usize;
            cleaned.drain(0..excess);
        }

        cleaned
    }

    pub async fn load_chat_history(
        &self,
        chat_id: &str,
    ) -> Result<Vec<ChatMessage>, Box<dyn std::error::Error>> {
        let chat_file = self
            .data_dir
            .join("chats")
            .join(format!("{}.json", chat_id));

        if !chat_file.exists() {
            self.event_bus
                .emit_storage(StorageEvent::ChatHistoryLoaded {
                    chat_id: chat_id.to_string(),
                    message_count: 0,
                });
            return Ok(Vec::new());
        }

        let json_data = async_fs::read_to_string(&chat_file).await?;
        let history: ChatHistory = serde_json::from_str(&json_data)?;

        self.event_bus
            .emit_storage(StorageEvent::ChatHistoryLoaded {
                chat_id: chat_id.to_string(),
                message_count: history.messages.len(),
            });

        Ok(history.messages)
    }

    pub async fn append_message_to_chat(
        &self,
        chat_id: &str,
        message: &ChatMessage,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut messages = self.load_chat_history(chat_id).await?;
        messages.push(message.clone());

        self.save_chat_history_with_cleanup(chat_id, &messages)
            .await?;
        Ok(())
    }

    pub async fn cleanup_all_chats(&self) -> Result<(), Box<dyn std::error::Error>> {
        let chats_dir = self.data_dir.join("chats");

        if !chats_dir.exists() {
            return Ok(());
        }

        let mut total_removed = 0;
        let mut entries = async_fs::read_dir(&chats_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            if let Some(file_name) = entry.file_name().to_str() {
                if file_name.ends_with(".json") {
                    let chat_id = file_name.trim_end_matches(".json");
                    let before_messages = self.load_chat_history(chat_id).await?;
                    let before_count = before_messages.len();

                    if before_count > 0 {
                        self.save_chat_history_with_cleanup(chat_id, &before_messages)
                            .await?;
                        let after_messages = self.load_chat_history(chat_id).await?;
                        total_removed += before_count.saturating_sub(after_messages.len());
                    }
                }
            }
        }

        if total_removed > 0 {
            self.event_bus.emit_storage(StorageEvent::CleanupCompleted {
                removed_items: total_removed,
            });
        }

        Ok(())
    }

    pub async fn save_private_key(
        &self,
        key_data: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let key_file = self.data_dir.join("keys").join("private.key");

        if key_file.exists() {
            let timestamp = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs();

            let backup_file = self
                .data_dir
                .join("backups")
                .join(format!("private_key_backup_{}.key", timestamp));

            if let Err(e) = tokio::fs::copy(&key_file, &backup_file).await {
                self.event_bus.emit_storage(StorageEvent::Error {
                    error: e.to_string(),
                    operation: "backup_private_key".to_string(),
                });
            } else {
                self.event_bus.emit_storage(StorageEvent::BackupCreated {
                    file_path: backup_file.to_string_lossy().to_string(),
                });
            }
        }

        async_fs::write(&key_file, key_data).await?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = async_fs::metadata(&key_file).await?.permissions();
            perms.set_mode(0o600);
            async_fs::set_permissions(&key_file, perms).await?;
        }

        Ok(())
    }

    pub async fn load_private_key(&self) -> Result<Option<Vec<u8>>, Box<dyn std::error::Error>> {
        let key_file = self.data_dir.join("keys").join("private.key");

        if !key_file.exists() {
            return Ok(None);
        }

        let key_data = async_fs::read(&key_file).await?;
        Ok(Some(key_data))
    }

    pub async fn get_storage_stats(&self) -> Result<StorageStats, Box<dyn std::error::Error>> {
        let mut stats = StorageStats::default();

        if let Ok(contacts) = self.load_contacts().await {
            stats.total_contacts = contacts.len();
            stats.online_contacts = contacts
                .values()
                .filter(|c| matches!(c.status, ContactStatus::Online))
                .count();
        }

        let chats_dir = self.data_dir.join("chats");

        if chats_dir.exists() {
            let mut entries = async_fs::read_dir(&chats_dir).await?;

            while let Some(entry) = entries.next_entry().await? {
                if entry.file_type().await?.is_file() {
                    stats.total_chats += 1;

                    if let Some(file_name) = entry.file_name().to_str() {
                        if file_name.ends_with(".json") {
                            let chat_id = file_name.trim_end_matches(".json");
                            if let Ok(messages) = self.load_chat_history(chat_id).await {
                                stats.total_messages += messages.len();
                            }
                        }
                    }
                }
            }
        }

        stats.data_size_bytes = self.calculate_directory_size(&self.data_dir).await?;
        Ok(stats)
    }

    async fn calculate_directory_size(&self, dir: &PathBuf) -> Result<u64, std::io::Error> {
        let mut total_size = 0u64;

        if dir.exists() {
            let mut entries = async_fs::read_dir(dir).await?;

            while let Some(entry) = entries.next_entry().await? {
                let metadata = entry.metadata().await?;

                if metadata.is_file() {
                    total_size += metadata.len();
                } else if metadata.is_dir() {
                    let path = entry.path();
                    let recursive_size: Pin<
                        Box<dyn futures::Future<Output = Result<u64, std::io::Error>>>,
                    > = Box::pin(self.calculate_directory_size(&path));
                    total_size += recursive_size.await?;
                }
            }
        }

        Ok(total_size)
    }

    pub async fn export_data(
        &self,
        export_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let export_data = ExportData {
            contacts: self.load_contacts().await?,
            chats: self.export_all_chats().await?,
            exported_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            version: "1.0".to_string(),
        };

        let json_data = serde_json::to_string_pretty(&export_data)?;
        async_fs::write(export_path, json_data).await?;

        self.event_bus.emit_storage(StorageEvent::BackupCreated {
            file_path: export_path.to_string_lossy().to_string(),
        });

        Ok(())
    }

    async fn export_all_chats(
        &self,
    ) -> Result<HashMap<String, Vec<ChatMessage>>, Box<dyn std::error::Error>> {
        let mut all_chats = HashMap::new();
        let chats_dir = self.data_dir.join("chats");

        if chats_dir.exists() {
            let mut entries = async_fs::read_dir(&chats_dir).await?;

            while let Some(entry) = entries.next_entry().await? {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.ends_with(".json") {
                        let chat_id = file_name.trim_end_matches(".json");
                        if let Ok(messages) = self.load_chat_history(chat_id).await {
                            all_chats.insert(chat_id.to_string(), messages);
                        }
                    }
                }
            }
        }

        Ok(all_chats)
    }
}

#[derive(Debug, Default)]
pub struct StorageStats {
    pub total_contacts: usize,
    pub online_contacts: usize,
    pub total_chats: usize,
    pub total_messages: usize,
    pub data_size_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct ExportData {
    contacts: HashMap<String, Contact>,
    chats: HashMap<String, Vec<ChatMessage>>,
    exported_at: u64,
    version: String,
}

impl StorageStats {
    pub fn format_size(&self) -> String {
        let size = self.data_size_bytes as f64;

        if size < 1024.0 {
            format!("{} B", size)
        } else if size < 1024.0 * 1024.0 {
            format!("{:.1} KB", size / 1024.0)
        } else if size < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.1} MB", size / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", size / (1024.0 * 1024.0 * 1024.0))
        }
    }
}
