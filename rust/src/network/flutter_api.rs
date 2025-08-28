use crate::core::ENGINE;
use crate::network::{ChatMessage, Contact, NetworkStats};

// Для решения проблемы с flutter_rust_bridge, используем feature gate
#[cfg(feature = "flutter")]
use flutter_rust_bridge::frb;

#[cfg_attr(feature = "flutter", frb)]
pub async fn start_network_server() -> Result<(), String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    // Note: This is a read-only reference, actual implementation would need Arc<Mutex<Engine>>
    Ok(())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn stop_network_server() -> Result<(), String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    // Note: This is a read-only reference, actual implementation would need Arc<Mutex<Engine>>
    Ok(())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn get_network_stats() -> Result<NetworkStats, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .network()
        .get_network_stats()
        .await
        .map_err(|e| e.to_string())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn send_message_to_contact(
    contact_name: String,
    content: String,
) -> Result<String, String> {
    let _engine = ENGINE.get().ok_or("Engine not initialized")?;
    // This would need mutable access to engine
    Err("Not implemented - needs mutable engine access".to_string())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn get_connected_peers() -> Result<Vec<String>, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    let peers = engine.network().get_connected_peers();
    Ok(peers.into_iter().map(|p| p.name).collect())
}

#[cfg_attr(feature = "flutter", frb)]
pub async fn is_network_running() -> Result<bool, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    Ok(engine.network().is_running())
}

// Альтернативная реализация без flutter_rust_bridge для тестирования
#[cfg(not(feature = "flutter"))]
pub mod fallback {
    use super::*;

    pub async fn start_network_server() -> Result<(), String> {
        println!("Network server start requested (fallback mode)");
        Ok(())
    }

    pub async fn stop_network_server() -> Result<(), String> {
        println!("Network server stop requested (fallback mode)");
        Ok(())
    }

    pub async fn get_network_stats() -> Result<NetworkStats, String> {
        use crate::network::NetworkStats;
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
