use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Arc, RwLock};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub default_port: u16,
    pub use_fixed_port: bool,
    pub auto_detect_external_ip: bool,
    pub connection_timeout: u64,
    pub max_connections: u32,
    pub test_mode: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            default_port: 8080,
            use_fixed_port: false,
            auto_detect_external_ip: true,
            connection_timeout: 10,
            max_connections: 100,
            test_mode: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub name: String,
    pub auto_accept_contacts: bool,
    pub show_notifications: bool,
}

impl Default for UserConfig {
    fn default() -> Self {
        Self {
            name: "user".to_string(),
            auto_accept_contacts: false,
            show_notifications: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub data_dir: PathBuf,
    pub auto_cleanup_days: u32,
    pub max_chat_history: u32,
    pub enable_backup: bool,
    pub backup_interval_hours: u32,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("./data"),
            auto_cleanup_days: 30,
            max_chat_history: 1000,
            enable_backup: true,
            backup_interval_hours: 24,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoConfig {
    pub encryption_enabled: bool,
    pub key_rotation_days: u32,
    pub algorithm: String,
}

impl Default for CryptoConfig {
    fn default() -> Self {
        Self {
            encryption_enabled: true,
            key_rotation_days: 90,
            algorithm: "AES-256-GCM".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub network: NetworkConfig,
    pub user: UserConfig,
    pub storage: StorageConfig,
    pub crypto: CryptoConfig,
    pub version: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            network: NetworkConfig::default(),
            user: UserConfig::default(),
            storage: StorageConfig::default(),
            crypto: CryptoConfig::default(),
            version: "1.0.0".to_string(),
        }
    }
}

pub struct ConfigManager {
    config_path: PathBuf,
    config: Arc<RwLock<AppConfig>>,
}

impl ConfigManager {
    pub fn new(config_path: &PathBuf) -> Result<Self, Box<dyn std::error::Error>> {
        let config = if config_path.exists() {
            let config_data = std::fs::read_to_string(config_path)?;
            toml::from_str(&config_data).unwrap_or_default()
        } else {
            let default_config = AppConfig::default();
            let config_data = toml::to_string_pretty(&default_config)?;

            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            std::fs::write(config_path, config_data)?;
            default_config
        };

        Ok(Self {
            config_path: config_path.clone(),
            config: Arc::new(RwLock::new(config)),
        })
    }

    pub fn get_config(&self) -> AppConfig {
        let config = self.config.read().unwrap();
        config.clone()
    }

    pub fn update_config<F>(&mut self, updater: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: FnOnce(&mut AppConfig),
    {
        {
            let mut config = self.config.write().unwrap();
            updater(&mut config);
        }

        self.save_config()
    }

    pub fn set_user_name(&mut self, name: String) -> Result<(), Box<dyn std::error::Error>> {
        self.update_config(|config| {
            config.user.name = name;
        })
    }

    pub fn enable_test_mode(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.update_config(|config| {
            config.network.test_mode = true;
            config.network.auto_detect_external_ip = false;
            config.network.use_fixed_port = false;
            config.storage.enable_backup = false;
        })
    }

    pub fn set_network_port(&mut self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        self.update_config(|config| {
            config.network.default_port = port;
        })
    }

    pub fn set_data_directory(&mut self, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        self.update_config(|config| {
            config.storage.data_dir = path;
        })
    }

    pub fn toggle_encryption(&mut self, enabled: bool) -> Result<(), Box<dyn std::error::Error>> {
        self.update_config(|config| {
            config.crypto.encryption_enabled = enabled;
        })
    }

    pub fn set_auto_cleanup_days(&mut self, days: u32) -> Result<(), Box<dyn std::error::Error>> {
        self.update_config(|config| {
            config.storage.auto_cleanup_days = days;
        })
    }

    pub fn set_max_chat_history(&mut self, max: u32) -> Result<(), Box<dyn std::error::Error>> {
        self.update_config(|config| {
            config.storage.max_chat_history = max;
        })
    }

    fn save_config(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config = self.config.read().unwrap();
        let config_data = toml::to_string_pretty(&*config)?;
        std::fs::write(&self.config_path, config_data)?;
        Ok(())
    }

    pub fn reload_config(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if self.config_path.exists() {
            let config_data = std::fs::read_to_string(&self.config_path)?;
            let new_config: AppConfig = toml::from_str(&config_data)?;

            let mut config = self.config.write().unwrap();
            *config = new_config;
        }

        Ok(())
    }

    pub fn reset_to_defaults(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut config = self.config.write().unwrap();
            *config = AppConfig::default();
        }

        self.save_config()
    }

    pub fn validate_config(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let config = self.config.read().unwrap();
        let mut issues = Vec::new();

        if config.network.default_port < 1024 {
            issues.push("Invalid network port range".to_string());
        }

        if config.network.connection_timeout == 0 {
            issues.push("Connection timeout cannot be zero".to_string());
        }

        if config.network.max_connections == 0 {
            issues.push("Max connections cannot be zero".to_string());
        }

        if config.user.name.is_empty() {
            issues.push("User name cannot be empty".to_string());
        }

        if config.storage.auto_cleanup_days == 0 {
            issues.push("Auto cleanup days cannot be zero".to_string());
        }

        if config.storage.max_chat_history == 0 {
            issues.push("Max chat history cannot be zero".to_string());
        }

        if config.crypto.key_rotation_days == 0 {
            issues.push("Key rotation days cannot be zero".to_string());
        }

        Ok(issues)
    }

    pub fn export_config(&self, export_path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let config = self.config.read().unwrap();
        let config_data = toml::to_string_pretty(&*config)?;
        std::fs::write(export_path, config_data)?;
        Ok(())
    }

    pub fn import_config(
        &mut self,
        import_path: &PathBuf,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !import_path.exists() {
            return Err("Import file does not exist".into());
        }

        let config_data = std::fs::read_to_string(import_path)?;
        let imported_config: AppConfig = toml::from_str(&config_data)?;

        {
            let mut config = self.config.write().unwrap();
            *config = imported_config;
        }

        self.save_config()
    }

    pub fn get_config_summary(&self) -> String {
        let config = self.config.read().unwrap();

        format!(
            "ShadowGhost Configuration:\n\
             User: {}\n\
             Network Port: {}\n\
             Data Directory: {}\n\
             Encryption: {}\n\
             Test Mode: {}",
            config.user.name,
            config.network.default_port,
            config.storage.data_dir.display(),
            if config.crypto.encryption_enabled {
                "Enabled"
            } else {
                "Disabled"
            },
            if config.network.test_mode {
                "Enabled"
            } else {
                "Disabled"
            }
        )
    }
}
