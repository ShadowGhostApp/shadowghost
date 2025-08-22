use crate::core::ShadowGhostCore;
use flutter_rust_bridge::frb;
use std::sync::{Arc, LazyLock};
use tokio::sync::Mutex;

pub static CORE: LazyLock<Mutex<Option<Arc<Mutex<ShadowGhostCore>>>>> =
    LazyLock::new(|| Mutex::const_new(None));

#[frb]
pub async fn initialize_core() -> Result<String, String> {
    let app_data_dir = std::env::temp_dir().join("shadowghost_flutter");
    std::fs::create_dir_all(&app_data_dir).map_err(|e| e.to_string())?;

    match ShadowGhostCore::new() {
        Ok(mut core) => match core.initialize(Some("Flutter User".to_string())).await {
            Ok(_) => {
                *CORE.lock().await = Some(Arc::new(Mutex::new(core)));
                Ok("Core initialized successfully".to_string())
            }
            Err(e) => Err(format!("Failed to initialize core: {}", e)),
        },
        Err(e) => Err(format!("Failed to create core: {}", e)),
    }
}

#[frb]
pub async fn initialize_core_with_config(
    user_name: String,
    data_path: String,
) -> Result<String, String> {
    let app_data_dir = std::path::Path::new(&data_path);
    std::fs::create_dir_all(&app_data_dir).map_err(|e| e.to_string())?;

    match ShadowGhostCore::new() {
        Ok(mut core) => match core.initialize(Some(user_name)).await {
            Ok(_) => {
                *CORE.lock().await = Some(Arc::new(Mutex::new(core)));
                Ok("Core initialized successfully with custom config".to_string())
            }
            Err(e) => Err(format!("Failed to initialize core: {}", e)),
        },
        Err(e) => Err(format!("Failed to create core: {}", e)),
    }
}

#[frb]
pub async fn shutdown_core() -> Result<String, String> {
    let mut core_guard = CORE.lock().await;
    if let Some(core) = core_guard.take() {
        // Graceful shutdown
        if let Err(e) = core.lock().await.shutdown().await {
            return Err(format!("Error during shutdown: {}", e));
        }
        Ok("Core shutdown successfully".to_string())
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn is_core_initialized() -> Result<bool, String> {
    let core_guard = CORE.lock().await;
    Ok(core_guard.is_some())
}

#[frb]
pub async fn get_core_status() -> Result<CoreStatus, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        let core_lock = core.lock().await;

        Ok(CoreStatus {
            initialized: core_lock.is_initialized(),
            server_running: core_lock.is_server_started(),
            user_name: core_lock
                .get_peer_info()
                .await
                .unwrap_or_else(|| "Unknown".to_string()),
            contact_count: core_lock.get_contact_count().await,
            server_status: core_lock.get_server_status().await,
        })
    } else {
        Ok(CoreStatus {
            initialized: false,
            server_running: false,
            user_name: "Not initialized".to_string(),
            contact_count: 0,
            server_status: "🔴 Not running".to_string(),
        })
    }
}

#[frb]
pub async fn generate_my_link() -> Result<String, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        match core.lock().await.generate_sg_link().await {
            Ok(link) => Ok(link),
            Err(e) => Err(format!("Failed to generate link: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn start_server() -> Result<String, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        match core.lock().await.start_server().await {
            Ok(_) => Ok("Server started successfully".to_string()),
            Err(e) => Err(format!("Failed to start server: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn stop_server() -> Result<String, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        match core.lock().await.stop_server().await {
            Ok(_) => Ok("Server stopped successfully".to_string()),
            Err(e) => Err(format!("Failed to stop server: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn restart_server() -> Result<String, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        match core.lock().await.restart_server().await {
            Ok(_) => Ok("Server restarted successfully".to_string()),
            Err(e) => Err(format!("Failed to restart server: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn update_user_name(new_name: String) -> Result<String, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        match core.lock().await.update_user_name(new_name.clone()).await {
            Ok(_) => Ok(format!("User name updated to '{}'", new_name)),
            Err(e) => Err(format!("Failed to update user name: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn get_connection_info() -> Result<ConnectionInfo, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        match core.lock().await.get_connection_info().await {
            Ok(info) => {
                let lines: Vec<String> = info.lines().map(|s| s.to_string()).collect();
                Ok(ConnectionInfo {
                    raw_info: info,
                    details: lines,
                })
            }
            Err(e) => Err(format!("Failed to get connection info: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn update_external_address() -> Result<String, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        match core.lock().await.update_external_address().await {
            Ok(_) => Ok("External address updated successfully".to_string()),
            Err(e) => Err(format!("Failed to update external address: {}", e)),
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn validate_sg_link(sg_link: String) -> Result<SgLinkInfo, String> {
    // Basic SG link validation without adding contact
    if !sg_link.starts_with("sg://") {
        return Err("Invalid SG link format".to_string());
    }

    let link_data = &sg_link[5..];
    use base64::{engine::general_purpose, Engine as _};

    match general_purpose::STANDARD.decode(link_data) {
        Ok(decoded_data) => match String::from_utf8(decoded_data) {
            Ok(data_str) => match serde_json::from_str::<serde_json::Value>(&data_str) {
                Ok(data) => Ok(SgLinkInfo {
                    valid: true,
                    peer_name: data
                        .get("name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown")
                        .to_string(),
                    peer_address: data
                        .get("address")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown")
                        .to_string(),
                    error: None,
                }),
                Err(_) => Ok(SgLinkInfo {
                    valid: false,
                    peer_name: "Unknown".to_string(),
                    peer_address: "Unknown".to_string(),
                    error: Some("Invalid JSON data".to_string()),
                }),
            },
            Err(_) => Ok(SgLinkInfo {
                valid: false,
                peer_name: "Unknown".to_string(),
                peer_address: "Unknown".to_string(),
                error: Some("Invalid UTF-8 data".to_string()),
            }),
        },
        Err(_) => Ok(SgLinkInfo {
            valid: false,
            peer_name: "Unknown".to_string(),
            peer_address: "Unknown".to_string(),
            error: Some("Invalid base64 encoding".to_string()),
        }),
    }
}

#[frb]
pub async fn get_system_health() -> Result<SystemHealth, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        let core_lock = core.lock().await;

        // Collect system health information
        let network_stats =
            core_lock
                .get_network_stats()
                .await
                .unwrap_or_else(|_| crate::network::NetworkStats {
                    connected_peers: 0,
                    total_messages_sent: 0,
                    total_messages_received: 0,
                    bytes_sent: 0,
                    bytes_received: 0,
                    uptime_seconds: 0,
                    messages_sent: 0,
                    messages_received: 0,
                    total_connections: 0,
                });
        let contact_count = core_lock.get_contact_count().await;

        Ok(SystemHealth {
            core_initialized: core_lock.is_initialized(),
            server_running: core_lock.is_server_started(),
            network_active: network_stats.connected_peers > 0,
            contact_count,
            message_count: network_stats.total_messages_sent
                + network_stats.total_messages_received,
            last_activity: chrono::Utc::now(),
            memory_usage: get_memory_usage(),
            error_count: 0, // TODO: Implement error tracking
        })
    } else {
        Ok(SystemHealth {
            core_initialized: false,
            server_running: false,
            network_active: false,
            contact_count: 0,
            message_count: 0,
            last_activity: chrono::Utc::now(),
            memory_usage: 0,
            error_count: 0,
        })
    }
}

// Simple helper functions
fn get_memory_usage() -> u64 {
    // TODO: Implement actual memory usage calculation
    0
}

// Simple structs defined inline
#[derive(Debug, Clone)]
pub struct CoreStatus {
    pub initialized: bool,
    pub server_running: bool,
    pub user_name: String,
    pub contact_count: usize,
    pub server_status: String,
}

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub raw_info: String,
    pub details: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SgLinkInfo {
    pub valid: bool,
    pub peer_name: String,
    pub peer_address: String,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct SystemHealth {
    pub core_initialized: bool,
    pub server_running: bool,
    pub network_active: bool,
    pub contact_count: usize,
    pub message_count: u64,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub memory_usage: u64,
    pub error_count: u32,
}
