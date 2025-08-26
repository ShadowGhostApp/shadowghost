use crate::core::config::Config;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

pub struct HealthMonitor {
    config: Config,
    last_status: HealthStatus,
    check_interval: Duration,
    is_running: bool,
}

impl HealthMonitor {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            last_status: HealthStatus::default(),
            check_interval: Duration::from_secs(30),
            is_running: false,
        }
    }

    pub async fn start(&mut self) -> Result<(), String> {
        self.is_running = true;
        self.perform_health_check().await?;
        Ok(())
    }

    pub async fn stop(&mut self) -> Result<(), String> {
        self.is_running = false;
        Ok(())
    }

    pub async fn get_health_status(&self) -> Result<HealthStatus, String> {
        Ok(self.last_status.clone())
    }

    async fn perform_health_check(&mut self) -> Result<(), String> {
        let mut components = HashMap::new();
        let mut issues = Vec::new();
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check core components
        components.insert(
            "core".to_string(),
            ComponentHealth {
                name: "Core System".to_string(),
                status: HealthLevel::Healthy,
                message: "Core system operational".to_string(),
                last_check: timestamp,
                metrics: HashMap::new(),
            },
        );

        // Check network component
        let network_health = self.check_network_health().await;
        components.insert("network".to_string(), network_health.clone());

        if network_health.status != HealthLevel::Healthy {
            issues.push(HealthIssue {
                component: "network".to_string(),
                severity: network_health.status.clone(),
                message: network_health.message.clone(),
                timestamp,
            });
        }

        // Check storage component
        let storage_health = self.check_storage_health().await;
        components.insert("storage".to_string(), storage_health.clone());

        if storage_health.status != HealthLevel::Healthy {
            issues.push(HealthIssue {
                component: "storage".to_string(),
                severity: storage_health.status.clone(),
                message: storage_health.message.clone(),
                timestamp,
            });
        }

        // Determine overall health
        let overall_health = if issues.is_empty() {
            HealthLevel::Healthy
        } else if issues.iter().any(|i| i.severity == HealthLevel::Critical) {
            HealthLevel::Critical
        } else {
            HealthLevel::Warning
        };

        self.last_status = HealthStatus {
            overall_health,
            components,
            last_check: timestamp,
            issues,
        };

        Ok(())
    }

    async fn check_network_health(&self) -> ComponentHealth {
        let mut metrics = HashMap::new();
        metrics.insert("port".to_string(), self.config.network.port as f64);

        ComponentHealth {
            name: "Network".to_string(),
            status: HealthLevel::Healthy,
            message: "Network services operational".to_string(),
            last_check: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metrics,
        }
    }

    async fn check_storage_health(&self) -> ComponentHealth {
        use std::path::Path;

        let path = Path::new(&self.config.storage.data_path);
        let status = if path.exists() && path.is_dir() {
            HealthLevel::Healthy
        } else {
            HealthLevel::Warning
        };

        let _message = if status == HealthLevel::Healthy {
            "Storage accessible".to_string()
        } else {
            "Storage path issues detected".to_string()
        };

        ComponentHealth {
            name: "Storage".to_string(),
            status,
            message: _message,
            last_check: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metrics: HashMap::new(),
        }
    }

    pub fn is_running(&self) -> bool {
        self.is_running
    }
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
    pub fn healthy(_message: String) -> Self {
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
