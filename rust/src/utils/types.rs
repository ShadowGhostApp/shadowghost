use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

// Connection testing types
#[derive(Debug, Clone)]
pub struct ConnectionTestResult {
    pub address: String,
    pub is_reachable: bool,
    pub response_time: Option<Duration>,
    pub error: Option<String>,
}

impl ConnectionTestResult {
    pub fn success(address: String, response_time: Duration) -> Self {
        Self {
            address,
            is_reachable: true,
            response_time: Some(response_time),
            error: None,
        }
    }

    pub fn failure(address: String, error: String) -> Self {
        Self {
            address,
            is_reachable: false,
            response_time: None,
            error: Some(error),
        }
    }
}

// Path configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConfiguration {
    pub app_data_dir: PathBuf,
    pub config_dir: PathBuf,
    pub chats_dir: PathBuf,
    pub contacts_file: PathBuf,
    pub peer_config_file: PathBuf,
    pub keys_dir: PathBuf,
    pub temp_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub backups_dir: PathBuf,
}

impl PathConfiguration {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let app_data_dir = crate::utils::paths::DataPaths::get_app_data_dir()?;
        
        Ok(Self {
            config_dir: crate::utils::paths::DataPaths::get_config_dir()?,
            chats_dir: crate::utils::paths::DataPaths::get_chats_dir()?,
            contacts_file: crate::utils::paths::DataPaths::get_contacts_file()?,
            peer_config_file: crate::utils::paths::DataPaths::get_peer_config_file()?,
            keys_dir: crate::utils::paths::DataPaths::get_keys_dir()?,
            temp_dir: crate::utils::paths::DataPaths::get_temp_dir()?,
            logs_dir: crate::utils::paths::DataPaths::get_logs_dir()?,
            backups_dir: crate::utils::paths::DataPaths::get_backups_dir()?,
            app_data_dir,
        })
    }

    pub fn validate_paths(&self) -> Vec<String> {
        let mut issues = Vec::new();

        let paths_to_check = vec![
            ("App Data", &self.app_data_dir),
            ("Config", &self.config_dir),
            ("Chats", &self.chats_dir),
            ("Keys", &self.keys_dir),
            ("Temp", &self.temp_dir),
            ("Logs", &self.logs_dir),
            ("Backups", &self.backups_dir),
        ];

        for (name, path) in paths_to_check {
            if !path.exists() {
                issues.push(format!("{} directory does not exist: {}", name, path.display()));
            } else if !path.is_dir() {
                issues.push(format!("{} path is not a directory: {}", name, path.display()));
            }
        }

        // Check file paths
        let parent = self.contacts_file.parent();
        if let Some(parent_dir) = parent {
            if !parent_dir.exists() {
                issues.push(format!("Contacts file parent directory does not exist: {}", parent_dir.display()));
            }
        }

        let parent = self.peer_config_file.parent();
        if let Some(parent_dir) = parent {
            if !parent_dir.exists() {
                issues.push(format!("Peer config file parent directory does not exist: {}", parent_dir.display()));
            }
        }

        issues
    }
}

// Validation types
#[derive(Debug, Clone)]
pub enum ValidationError {
    Empty(String),
    TooLong(String, usize),
    InvalidCharacters(String),
    InvalidFormat(String),
}

impl ValidationError {
    pub fn message(&self) -> String {
        match self {
            ValidationError::Empty(field) => format!("{} cannot be empty", field),
            ValidationError::TooLong(field, max_len) => {
                format!("{} is too long (maximum {} characters)", field, max_len)
            }
            ValidationError::InvalidCharacters(field) => {
                format!("{} contains invalid characters", field)
            }
            ValidationError::InvalidFormat(field) => format!("{} has invalid format", field),
        }
    }
}

// Address parsing types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedAddress {
    pub host: String,
    pub port: u16,
    pub is_local: bool,
    pub is_valid: bool,
}

impl ParsedAddress {
    pub fn new(host: String, port: u16) -> Self {
        let is_local = host == "127.0.0.1" 
            || host == "localhost" 
            || host == "::1" 
            || host.starts_with("192.168.")
            || host.starts_with("10.")
            || host.starts_with("172.16.");

        Self {
            host,
            port,
            is_local,
            is_valid: port > 0,
        }
    }

    pub fn full_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

// Message processing types
#[derive(Debug, Clone)]
pub struct MessageDeliveryStats {
    pub total_messages: usize,
    pub delivered_messages: usize,
    pub pending_messages: usize,
    pub failed_messages: usize,
    pub success_rate: f64,
}

impl MessageDeliveryStats {
    pub fn calculate(messages: &[crate::network::ChatMessage]) -> Self {
        let total = messages.len();
        let delivered = messages.iter().filter(|m| matches!(m.delivery_status, crate::network::DeliveryStatus::Delivered)).count();
        let pending = messages.iter().filter(|m| matches!(m.delivery_status, crate::network::DeliveryStatus::Pending)).count();
        let failed = messages.iter().filter(|m| matches!(m.delivery_status, crate::network::DeliveryStatus::Failed)).count();
        
        let success_rate = if total > 0 {
            delivered as f64 / total as f64
        } else {
            0.0
        };

        Self {
            total_messages: total,
            delivered_messages: delivered,
            pending_messages: pending,
            failed_messages: failed,
            success_rate,
        }
    }
}

// Utility result types
pub type UtilsResult<T> = Result<T, UtilsError>;

#[derive(Debug, Clone)]
pub enum UtilsError {
    InvalidInput(String),
    NetworkError(String),
    PathError(String),
    ValidationError(ValidationError),
}

impl std::fmt::Display for UtilsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UtilsError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            UtilsError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            UtilsError::PathError(msg) => write!(f, "Path error: {}", msg),
            UtilsError::ValidationError(err) => write!(f, "Validation error: {}", err.message()),
        }
    }
}

impl std::error::Error for UtilsError {}

impl From<ValidationError> for UtilsError {
    fn from(err: ValidationError) -> Self {
        UtilsError::ValidationError(err)
    }
}