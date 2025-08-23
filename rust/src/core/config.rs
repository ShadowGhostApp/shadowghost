use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub user_name: String,
    pub profile_id: String,
    pub network: NetworkConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub port: u16,
    pub max_peers: usize,
    pub enable_discovery: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub data_path: PathBuf,
    pub enable_encryption: bool,
}

impl Config {
    pub fn load(profile_path: &PathBuf) -> Result<Self, String> {
        let config_file = profile_path.join("config.toml");
        if config_file.exists() {
            let content = std::fs::read_to_string(&config_file)
                .map_err(|e| format!("Failed to read config: {}", e))?;
            toml::from_str(&content).map_err(|e| format!("Failed to parse config: {}", e))
        } else {
            Ok(Self::default_for_profile(profile_path))
        }
    }

    pub fn save(&self, profile_path: &PathBuf) -> Result<(), String> {
        let config_file = profile_path.join("config.toml");
        let content = toml::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        std::fs::write(&config_file, content).map_err(|e| format!("Failed to write config: {}", e))
    }

    fn default_for_profile(profile_path: &PathBuf) -> Self {
        Self {
            user_name: "User".to_string(),
            profile_id: uuid::Uuid::new_v4().to_string(),
            network: NetworkConfig {
                port: 8080 + rand::random::<u16>() % 1000, // Random port 8080-9080
                max_peers: 50,
                enable_discovery: true,
            },
            storage: StorageConfig {
                data_path: profile_path.clone(),
                enable_encryption: true,
            },
        }
    }
}
