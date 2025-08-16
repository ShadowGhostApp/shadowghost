use serde::{Deserialize, Serialize};

use std::fs;

use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct AppConfig {
    pub user: UserConfig,

    pub network: NetworkConfig,

    pub security: SecurityConfig,

    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct UserConfig {
    pub name: String,

    pub language: String,

    pub auto_start_server: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct NetworkConfig {
    pub default_port: u16,

    pub max_connections: u32,

    pub connection_timeout_ms: u64,

    pub heartbeat_interval_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct SecurityConfig {
    pub auto_accept_contacts: bool,

    pub require_encryption: bool,

    pub allow_anonymous_contacts: bool,

    pub max_message_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct StorageConfig {
    pub data_dir: PathBuf,

    pub max_chat_history: u32,

    pub auto_cleanup_days: u32,

    pub compress_old_messages: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            user: UserConfig {
                name: "user".to_string(),

                language: "en".to_string(),

                auto_start_server: true,
            },

            network: NetworkConfig {
                default_port: 8000,

                max_connections: 100,

                connection_timeout_ms: 30000,

                heartbeat_interval_ms: 60000,
            },

            security: SecurityConfig {
                auto_accept_contacts: false,

                require_encryption: false,

                allow_anonymous_contacts: false,

                max_message_size: 1024 * 1024,
            },

            storage: StorageConfig {
                data_dir: PathBuf::from("./data"),

                max_chat_history: 1000,

                auto_cleanup_days: 90,

                compress_old_messages: false,
            },
        }
    }
}

impl AppConfig {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        if path.as_ref().exists() {
            let content = fs::read_to_string(path)?;

            let config: AppConfig = toml::from_str(&content)?;

            Ok(config)
        } else {
            let default = AppConfig::default();

            default.save_to_file(path)?;

            Ok(default)
        }
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), Box<dyn std::error::Error>> {
        let content = toml::to_string_pretty(self)?;

        fs::write(path, content)?;

        Ok(())
    }
}

pub struct ConfigManager {
    config: AppConfig,

    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new<P: AsRef<Path>>(config_path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = config_path.as_ref().to_path_buf();

        let config = AppConfig::load_from_file(&config_path)?;

        Ok(Self {
            config,

            config_path,
        })
    }

    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }

    pub fn update_config<F>(&mut self, updater: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce(&mut AppConfig),
    {
        updater(&mut self.config);

        self.save_config()
    }

    pub fn set_user_name(&mut self, name: String) -> Result<(), Box<dyn std::error::Error>> {
        self.config.user.name = name;

        self.save_config()
    }

    pub fn set_default_port(&mut self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        self.config.network.default_port = port;

        self.save_config()
    }

    pub fn toggle_auto_accept_contacts(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.config.security.auto_accept_contacts = !self.config.security.auto_accept_contacts;

        self.save_config()
    }

    fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.config.save_to_file(&self.config_path)
    }
}
