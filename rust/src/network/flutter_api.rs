use crate::core::ENGINE;
use crate::network::{ChatMessage, Contact, NetworkStats};
use flutter_rust_bridge::frb;

#[frb]
pub async fn start_network_server() -> Result<(), String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    // Note: This is a read-only reference, actual implementation would need Arc<Mutex<Engine>>
    Ok(())
}

#[frb]
pub async fn stop_network_server() -> Result<(), String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    // Note: This is a read-only reference, actual implementation would need Arc<Mutex<Engine>>
    Ok(())
}

#[frb]
pub async fn get_network_stats() -> Result<NetworkStats, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    engine
        .network()
        .get_network_stats()
        .await
        .map_err(|e| e.to_string())
}

#[frb]
pub async fn send_message_to_contact(
    contact_name: String,
    content: String,
) -> Result<String, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    // This would need mutable access to engine
    Err("Not implemented - needs mutable engine access".to_string())
}

#[frb]
pub async fn get_connected_peers() -> Result<Vec<String>, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    let peers = engine.network().get_connected_peers();
    Ok(peers.into_iter().map(|p| p.name).collect())
}

#[frb]
pub async fn is_network_running() -> Result<bool, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    Ok(engine.network().is_running())
}
