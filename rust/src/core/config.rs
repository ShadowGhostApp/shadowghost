use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;
use std::path::PathBuf;

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
    pub default_port: u16,
    pub max_peers: usize,
    pub connection_timeout: u64,
    pub enable_upnp: bool,
    pub enable_stun: bool,
    pub test_mode: bool,
    pub auto_detect_external_ip: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            port: 8080,
            default_port: 8080,
            max_peers: 50,
            connection_timeout: 30,
            enable_upnp: true,
            enable_stun: true,
            test_mode: false,
            auto_detect_external_ip: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub data_path: String,
    pub data_dir: String,
    pub backup_interval: u64,
    pub max_backups: usize,
    pub auto_backup: bool,
    pub enable_backup: bool,
    pub auto_cleanup_days: u32,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_path: "./data".to_string(),
            data_dir: "./data".to_string(),
            backup_interval: 86400,
            max_backups: 7,
            auto_backup: true,
            enable_backup: true,
            auto_cleanup_days: 30,
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
pub struct UserConfig {
    pub name: String,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            name: "user".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub user_name: String,
    pub user: UserConfig,
    pub network: NetworkConfig,
    pub storage: StorageConfig,
    pub security: SecurityConfig,
    pub version: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            user_name: "user".to_string(),
            user: UserConfig::default(),
            network: NetworkConfig::default(),
            storage: StorageConfig::default(),
            security: SecurityConfig::default(),
            version: "0.1.0".to_string(),
        }
    }
}

pub struct ConfigManager {
    config: AppConfig,
    config_path: PathBuf,
}

impl ConfigManager {
    pub fn new<P: Into<PathBuf>>(config_path: P) -> Result<Self, ConfigError> {
        let config_path = config_path.into();

        let mut manager = Self {
            config: AppConfig::default(),
            config_path,
        };

        if let Err(_) = manager.load() {
            manager.save()?;
        }

        Ok(manager)
    }

    pub fn load(&mut self) -> Result<(), ConfigError> {
        match std::fs::read_to_string(&self.config_path) {
            Ok(data) => {
                self.config = toml::from_str(&data)
                    .map_err(|e| ConfigError::SerializationError(e.to_string()))?;

                self.config.user_name = self.config.user.name.clone();
                self.config.network.default_port = self.config.network.port;
                self.config.storage.data_dir = self.config.storage.data_path.clone();
            }
            Err(_) => {
                self.config = AppConfig::default();
                self.save()?;
            }
        }
        Ok(())
    }

    pub fn save(&self) -> Result<(), ConfigError> {
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| ConfigError::IoError(e.to_string()))?;
        }

        let data = toml::to_string_pretty(&self.config)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))?;

        std::fs::write(&self.config_path, data).map_err(|e| ConfigError::IoError(e.to_string()))?;

        Ok(())
    }

    pub fn get_config(&self) -> &AppConfig {
        &self.config
    }

    pub fn get_config_mut(&mut self) -> &mut AppConfig {
        &mut self.config
    }

    pub fn set_user_name(&mut self, name: String) -> Result<(), ConfigError> {
        if name.trim().is_empty() {
            return Err(ConfigError::ValidationError(
                "Name cannot be empty".to_string(),
            ));
        }

        if name.len() > 1000 {
            return Err(ConfigError::ValidationError("Name too long".to_string()));
        }

        if name.contains('\x00') {
            return Err(ConfigError::ValidationError(
                "Name contains invalid characters".to_string(),
            ));
        }

        self.config.user_name = name.clone();
        self.config.user.name = name;
        self.save()
    }

    pub fn set_network_port(&mut self, port: u16) -> Result<(), ConfigError> {
        if port < 1024 {
            return Err(ConfigError::ValidationError(
                "Port must be >= 1024".to_string(),
            ));
        }
        self.config.network.port = port;
        self.config.network.default_port = port;
        self.save()
    }

    pub fn set_auto_cleanup_days(&mut self, days: u32) -> Result<(), ConfigError> {
        self.config.storage.auto_cleanup_days = days;
        self.save()
    }

    pub fn update_data_path(&mut self, path: String) -> Result<(), ConfigError> {
        if path.trim().is_empty() {
            return Err(ConfigError::ValidationError(
                "Path cannot be empty".to_string(),
            ));
        }
        self.config.storage.data_path = path.clone();
        self.config.storage.data_dir = path;
        self.save()
    }

    pub fn add_blocked_ip(&mut self, ip: String) -> Result<(), ConfigError> {
        if !self.config.security.blocked_ips.contains(&ip) {
            self.config.security.blocked_ips.push(ip);
            self.save()?;
        }
        Ok(())
    }

    pub fn remove_blocked_ip(&mut self, ip: &str) -> Result<(), ConfigError> {
        self.config.security.blocked_ips.retain(|x| x != ip);
        self.save()
    }

    pub fn enable_test_mode(&mut self) -> Result<(), ConfigError> {
        self.config.network.test_mode = true;
        self.config.network.auto_detect_external_ip = false;
        self.config.storage.enable_backup = false;
        self.save()
    }

    pub fn disable_test_mode(&mut self) -> Result<(), ConfigError> {
        self.config.network.test_mode = false;
        self.config.network.auto_detect_external_ip = true;
        self.config.storage.enable_backup = true;
        self.save()
    }

    pub fn validate_config(&self) -> Result<Vec<String>, ConfigError> {
        let mut issues = Vec::new();

        if self.config.user.name.is_empty() {
            issues.push("User name cannot be empty".to_string());
        }

        if self.config.user.name.len() > 1000 {
            issues.push("User name is too long".to_string());
        }

        if self.config.user.name.contains('\x00') {
            issues.push("User name contains invalid characters".to_string());
        }

        if self.config.network.port < 1024 {
            issues.push("Port must be >= 1024".to_string());
        }

        if self.config.network.max_peers == 0 {
            issues.push("Max peers must be > 0".to_string());
        }

        if self.config.storage.data_path.is_empty() {
            issues.push("Data path cannot be empty".to_string());
        }

        if self.config.storage.data_path.contains("..") {
            issues.push("Data path contains invalid sequences".to_string());
        }

        Ok(issues)
    }

    pub fn reset_to_defaults(&mut self) -> Result<(), ConfigError> {
        self.config = AppConfig::default();
        self.save()
    }

    pub fn update_config<F>(&mut self, updater: F) -> Result<(), ConfigError>
    where
        F: FnOnce(&mut AppConfig),
    {
        updater(&mut self.config);

        self.config.user_name = self.config.user.name.clone();
        self.config.network.default_port = self.config.network.port;
        self.config.storage.data_dir = self.config.storage.data_path.clone();

        self.save()
    }

    pub fn get_data_dir(&self) -> &str {
        &self.config.storage.data_path
    }

    pub fn get_user_name(&self) -> &str {
        &self.config.user.name
    }

    pub fn get_network_port(&self) -> u16 {
        self.config.network.port
    }

    pub fn is_test_mode(&self) -> bool {
        self.config.network.test_mode
    }

    pub fn export_config(&self) -> Result<String, ConfigError> {
        toml::to_string_pretty(&self.config)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))
    }

    pub fn import_config(&mut self, config_data: &str) -> Result<(), ConfigError> {
        let new_config: AppConfig = toml::from_str(config_data)
            .map_err(|e| ConfigError::SerializationError(e.to_string()))?;

        let temp_manager = Self {
            config: new_config.clone(),
            config_path: self.config_path.clone(),
        };

        let issues = temp_manager.validate_config()?;
        if !issues.is_empty() {
            return Err(ConfigError::ValidationError(format!(
                "Invalid config: {}",
                issues.join(", ")
            )));
        }

        self.config = new_config;
        self.save()
    }
}
