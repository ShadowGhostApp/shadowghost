use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ConfigError {
    IoError(String),
    SerializationError(String),
    ValidationError(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::IoError(msg) => write!(f, "IO error: {}", msg),
            ConfigError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            ConfigError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
        }
    }
}

impl Error for ConfigError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub port: u16,
    pub max_peers: usize,
    pub connection_timeout: u64,
    pub enable_upnp: bool,
    pub enable_stun: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            max_peers: 50,
            connection_timeout: 30,
            enable_upnp: true,
            enable_stun: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub data_path: String,
    pub backup_interval: u64,
    pub max_backups: usize,
    pub auto_backup: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_path: "./data".to_string(),
            backup_interval: 86400,
            max_backups: 7,
            auto_backup: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub encryption_enabled: bool,
    pub require_verification: bool,
    pub auto_accept_contacts: bool,
    pub blocked_ips: Vec<String>,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            encryption_enabled: true,
            require_verification: true,
            auto_accept_contacts: false,
            blocked_ips: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub user_name: String,
    pub network: NetworkConfig,
    pub storage: StorageConfig,
    pub security: SecurityConfig,
    pub version: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            user_name: "user".to_string(),
            network: NetworkConfig::default(),
            storage: StorageConfig::default(),
            security: SecurityConfig::default(),
            version: "0.1.0".to_string(),
        }
    }
}

pub struct ConfigManager {
    config: AppConfig,
    config_path: String,
}

impl ConfigManager {
    pub fn new(config_path: String) -> Self {
        Self {
            config: AppConfig::default(),
            config_path,
        }
    }

    pub async fn load(&mut self) -> Result<(), ConfigError> {
        match tokio::fs::read_to_string(&self.config_path).await {
            Ok(data) => {
                self.config = serde_json::from_str(&data)
                    .map_err(|e| ConfigError::SerializationError(e.to_string()))?;
            }
            Err(_) => {
                self.config = AppConfig::default();
                self.save().await?;
            }
        }
        Ok(())
    }

    pub async fn save(&self) -> Result<(), ConfigError> {
        let data = serde_json::to_string_pretty(&self.config)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))?;

        tokio::fs::write(&self.config_path, data)
            .await
            .map_err(|e| ConfigError::IoError(e.to_string()))?;

        Ok(())
    }

    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }

    pub fn update_user_name(&mut self, name: String) -> Result<(), ConfigError> {
        if name.trim().is_empty() {
            return Err(ConfigError::ValidationError(
                "Name cannot be empty".to_string(),
            ));
        }
        self.config.user_name = name;
        Ok(())
    }

    pub fn update_network_port(&mut self, port: u16) -> Result<(), ConfigError> {
        if port < 1024 {
            return Err(ConfigError::ValidationError(
                "Port must be >= 1024".to_string(),
            ));
        }
        self.config.network.port = port;
        Ok(())
    }

    pub fn update_data_path(&mut self, path: String) -> Result<(), ConfigError> {
        if path.trim().is_empty() {
            return Err(ConfigError::ValidationError(
                "Path cannot be empty".to_string(),
            ));
        }
        self.config.storage.data_path = path;
        Ok(())
    }

    pub fn add_blocked_ip(&mut self, ip: String) -> Result<(), ConfigError> {
        if !self.config.security.blocked_ips.contains(&ip) {
            self.config.security.blocked_ips.push(ip);
        }
        Ok(())
    }

    pub fn remove_blocked_ip(&mut self, ip: &str) -> Result<(), ConfigError> {
        self.config.security.blocked_ips.retain(|x| x != ip);
        Ok(())
    }

    pub fn reset_to_defaults(&mut self) {
        self.config = AppConfig::default();
    }
}
