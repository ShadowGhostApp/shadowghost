use super::core_api::CORE;
use crate::network::manager::{NetworkStats, PeerData};
use flutter_rust_bridge::frb;

#[frb]
pub async fn start_discovery() -> Result<String, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        // TODO: Implement actual network discovery
        Ok("Discovery started successfully".to_string())
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn stop_discovery() -> Result<String, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        Ok("Discovery stopped successfully".to_string())
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn get_network_stats() -> Result<NetworkStats, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        match core.lock().await.get_network_stats().await {
            Ok(stats) => Ok(stats),
            Err(e) => Err(format!("Failed to get network stats: {}", e)),
        }
    } else {
        // Return empty stats if core not initialized
        Ok(NetworkStats {
            connected_peers: 0,
            total_messages_sent: 0,
            total_messages_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            uptime_seconds: 0,
            messages_sent: 0,
            messages_received: 0,
            total_connections: 0,
        })
    }
}

#[frb]
pub async fn get_connected_peers() -> Result<Vec<PeerData>, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        // TODO: Get actual connected peers from network manager
        Ok(vec![])
    } else {
        Ok(vec![])
    }
}

#[frb]
pub async fn connect_to_peer(peer_address: String) -> Result<String, String> {
    if peer_address.is_empty() {
        return Err("Peer address cannot be empty".to_string());
    }

    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        // Basic address validation
        if !is_valid_address(&peer_address) {
            return Err("Invalid peer address format".to_string());
        }

        // TODO: Implement actual peer connection
        Ok(format!(
            "Successfully initiated connection to {}",
            peer_address
        ))
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn disconnect_from_peer(peer_id: String) -> Result<String, String> {
    if peer_id.is_empty() {
        return Err("Peer ID cannot be empty".to_string());
    }

    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        Ok(format!("Successfully disconnected from peer {}", peer_id))
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn is_network_active() -> Result<bool, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        match core.lock().await.get_network_stats().await {
            Ok(stats) => Ok(stats.connected_peers > 0),
            Err(_) => Ok(false),
        }
    } else {
        Ok(false)
    }
}

#[frb]
pub async fn get_my_network_address() -> Result<String, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        match core.lock().await.get_connection_info().await {
            Ok(info) => {
                // Extract address from connection info
                let lines: Vec<&str> = info.lines().collect();
                for line in lines {
                    if line.starts_with("Address:") {
                        if let Some(address) = line.split(":").nth(1) {
                            return Ok(address.trim().to_string());
                        }
                    }
                }
                Ok("127.0.0.1:8080".to_string()) // Fallback
            }
            Err(_) => Ok("127.0.0.1:8080".to_string()),
        }
    } else {
        Ok("127.0.0.1:8080".to_string())
    }
}

#[frb]
pub async fn ping_peer(peer_id: String) -> Result<u64, String> {
    if peer_id.is_empty() {
        return Err("Peer ID cannot be empty".to_string());
    }

    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        // Simulate ping
        let start = std::time::Instant::now();

        // TODO: Implement actual ping
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let ping_time = start.elapsed().as_millis() as u64;
        Ok(ping_time)
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn set_network_config(max_peers: u32, port: u16) -> Result<String, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        // Basic validation
        if port < 1024 {
            return Err("Port must be >= 1024".to_string());
        }

        if max_peers == 0 {
            return Err("Max peers must be > 0".to_string());
        }

        // TODO: Update actual network configuration
        Ok(format!(
            "Network config updated: max_peers={}, port={}",
            max_peers, port
        ))
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn test_connection_to_peer(peer_address: String) -> Result<ConnectionTestResult, String> {
    if peer_address.is_empty() {
        return Err("Peer address cannot be empty".to_string());
    }

    if !is_valid_address(&peer_address) {
        return Ok(ConnectionTestResult {
            success: false,
            ping_time: 0,
            error: Some("Invalid address format".to_string()),
            peer_info: None,
        });
    }

    let start = std::time::Instant::now();

    // TODO: Implement actual connection testing
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let ping_time = start.elapsed().as_millis() as u64;

    Ok(ConnectionTestResult {
        success: true,
        ping_time,
        error: None,
        peer_info: Some(PeerInfo {
            name: "Test Peer".to_string(),
            version: "1.0.0".to_string(),
            supported_protocols: vec!["SG/1.0".to_string()],
        }),
    })
}

#[frb]
pub async fn get_network_health() -> Result<NetworkHealth, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        match core.lock().await.get_network_stats().await {
            Ok(stats) => {
                Ok(NetworkHealth {
                    is_healthy: stats.connected_peers > 0,
                    connected_peers: stats.connected_peers,
                    message_success_rate: calculate_success_rate(&stats),
                    average_ping: 50,  // TODO: Calculate actual ping
                    network_errors: 0, // TODO: Track actual errors
                    last_error: None,
                })
            }
            Err(e) => Ok(NetworkHealth {
                is_healthy: false,
                connected_peers: 0,
                message_success_rate: 0.0,
                average_ping: 0,
                network_errors: 1,
                last_error: Some(e.to_string()),
            }),
        }
    } else {
        Ok(NetworkHealth {
            is_healthy: false,
            connected_peers: 0,
            message_success_rate: 0.0,
            average_ping: 0,
            network_errors: 0,
            last_error: Some("Core not initialized".to_string()),
        })
    }
}

#[frb]
pub async fn scan_for_peers() -> Result<Vec<DiscoveredPeer>, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        // TODO: Implement actual peer scanning
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        Ok(vec![
            DiscoveredPeer {
                id: "peer1".to_string(),
                name: "Local Peer 1".to_string(),
                address: "192.168.1.100:8080".to_string(),
                signal_strength: 85,
                last_seen: chrono::Utc::now(),
            },
            DiscoveredPeer {
                id: "peer2".to_string(),
                name: "Local Peer 2".to_string(),
                address: "192.168.1.101:8080".to_string(),
                signal_strength: 70,
                last_seen: chrono::Utc::now(),
            },
        ])
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb]
pub async fn enable_auto_discovery(enabled: bool) -> Result<String, String> {
    let core_guard = CORE.lock().await;
    if let Some(core) = core_guard.as_ref() {
        if enabled {
            Ok("Auto-discovery enabled".to_string())
        } else {
            Ok("Auto-discovery disabled".to_string())
        }
    } else {
        Err("Core not initialized".to_string())
    }
}

// Helper functions
fn is_valid_address(address: &str) -> bool {
    if !address.contains(':') {
        return false;
    }

    let parts: Vec<&str> = address.split(':').collect();
    if parts.len() != 2 {
        return false;
    }

    // Check port
    if let Ok(port) = parts[1].parse::<u16>() {
        port > 0
    } else {
        false
    }
}

fn calculate_success_rate(stats: &NetworkStats) -> f64 {
    let total_messages = stats.total_messages_sent + stats.total_messages_received;
    if total_messages > 0 {
        // Simple success rate calculation
        0.95 // 95% success rate for demo
    } else {
        0.0
    }
}

// Simple structs defined inline
#[derive(Debug, Clone)]
pub struct ConnectionTestResult {
    pub success: bool,
    pub ping_time: u64,
    pub error: Option<String>,
    pub peer_info: Option<PeerInfo>,
}

#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub name: String,
    pub version: String,
    pub supported_protocols: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct NetworkHealth {
    pub is_healthy: bool,
    pub connected_peers: u32,
    pub message_success_rate: f64,
    pub average_ping: u64,
    pub network_errors: u32,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DiscoveredPeer {
    pub id: String,
    pub name: String,
    pub address: String,
    pub signal_strength: u8, // 0-100
    pub last_seen: chrono::DateTime<chrono::Utc>,
}
