use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

// Configuration types
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

// Profile types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used: chrono::DateTime<chrono::Utc>,
}

// Peer types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peer {
    pub id: String,
    pub name: String,
    pub address: String,
    pub public_key: Vec<u8>,
    pub port: u16,
}

impl Peer {
    pub fn new(name: String, address: String) -> Self {
        let (host, port) = Self::parse_address(&address);
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            address: host,
            public_key: vec![],
            port,
        }
    }

    pub fn new_with_entropy(name: String, address: String) -> Self {
        let (host, port) = Self::parse_address(&address);

        let entropy = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        let id = format!("{}_{}", uuid::Uuid::new_v4(), entropy);

        Self {
            id,
            name,
            address: host,
            public_key: vec![],
            port,
        }
    }

    pub fn with_address(name: String, address: String, port: u16) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            address,
            public_key: vec![],
            port,
        }
    }

    pub fn get_full_address(&self) -> String {
        format!("{}:{}", self.address, self.port)
    }

    pub fn update_address(&mut self, address: String, port: u16) {
        self.address = address;
        self.port = port;
    }

    pub fn set_public_key(&mut self, key: Vec<u8>) {
        self.public_key = key;
    }

    pub fn get_short_id(&self) -> String {
        if self.id.len() > 8 {
            self.id[..8].to_string()
        } else {
            self.id.clone()
        }
    }

    pub fn get_info(&self) -> String {
        format!("{} ({}:{})", self.name, self.address, self.port)
    }

    fn parse_address(address: &str) -> (String, u16) {
        if let Some(colon_pos) = address.rfind(':') {
            let host = &address[..colon_pos];
            let port_str = &address[colon_pos + 1..];

            if let Ok(port) = port_str.parse::<u16>() {
                (host.to_string(), port)
            } else {
                (address.to_string(), 8080)
            }
        } else {
            (address.to_string(), 8080)
        }
    }
}

impl fmt::Display for Peer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({}:{})", self.name, self.address, self.port)
    }
}

// Error types
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("Initialization error: {0}")]
    Initialization(String),
    #[error("Profile error: {0}")]
    Profile(String),
    #[error("Configuration error: {0}")]
    Config(String),
    #[error("Manager error: {0}")]
    Manager(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Storage error: {0}")]
    Storage(String),
    #[error("Contact error: {0}")]
    Contact(String),
    #[error("Invalid state: {0}")]
    InvalidState(String),
}

// Health monitoring types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub overall_health: HealthLevel,
    pub components: HashMap<String, ComponentHealth>,
    pub last_check: u64,
    pub issues: Vec<HealthIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthLevel {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthLevel,
    pub message: String,
    pub last_check: u64,
    pub metrics: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIssue {
    pub component: String,
    pub severity: HealthLevel,
    pub message: String,
    pub timestamp: u64,
}

impl Default for HealthStatus {
    fn default() -> Self {
        Self {
            overall_health: HealthLevel::Unknown,
            components: HashMap::new(),
            last_check: 0,
            issues: Vec::new(),
        }
    }
}

impl HealthStatus {
    pub fn healthy(message: String) -> Self {
        Self {
            overall_health: HealthLevel::Healthy,
            components: HashMap::new(),
            last_check: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            issues: Vec::new(),
        }
    }

    pub fn unhealthy(message: String) -> Self {
        Self {
            overall_health: HealthLevel::Critical,
            components: HashMap::new(),
            last_check: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            issues: vec![HealthIssue {
                component: "system".to_string(),
                severity: HealthLevel::Critical,
                message,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            }],
        }
    }
}

// Metrics types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub network_bytes_sent: u64,
    pub network_bytes_received: u64,
    pub active_connections: u32,
    pub message_count: u64,
    pub error_count: u32,
    pub uptime_seconds: u64,
    pub timestamp: u64,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0,
            network_bytes_sent: 0,
            network_bytes_received: 0,
            active_connections: 0,
            message_count: 0,
            error_count: 0,
            uptime_seconds: 0,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    pub timestamp: u64,
    pub value: f64,
}