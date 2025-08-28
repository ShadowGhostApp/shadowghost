use crate::core::ENGINE;
use crate::crypto::{PublicKey, TrustStats};
use flutter_rust_bridge::frb;

#[frb]
pub async fn get_public_key() -> Result<PublicKey, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    Ok(engine.crypto().get_public_key().await)
}

#[frb]
pub async fn encrypt_message(message: String, recipient_key: PublicKey) -> Result<String, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    let encrypted = engine
        .crypto()
        .encrypt_message(&message, &recipient_key)
        .await
        .map_err(|e| e.to_string())?;

    // Convert encrypted message to string representation
    serde_json::to_string(&encrypted).map_err(|e| e.to_string())
}

#[frb]
pub async fn decrypt_message(encrypted_data: String) -> Result<String, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;

    // Parse encrypted message from string
    let encrypted_msg = serde_json::from_str(&encrypted_data)
        .map_err(|e| format!("Failed to parse encrypted message: {}", e))?;

    engine
        .crypto()
        .decrypt_message(&encrypted_msg)
        .await
        .map_err(|e| e.to_string())
}

#[frb]
pub async fn get_trust_stats() -> Result<TrustStats, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    Ok(engine.crypto().get_trust_stats())
}

#[frb]
pub async fn add_trusted_key(peer_id: String, public_key: PublicKey) -> Result<(), String> {
    let _engine = ENGINE.get().ok_or("Engine not initialized")?;
    // This would need mutable access to engine
    Err("Not implemented - needs mutable engine access".to_string())
}

#[frb]
pub async fn remove_trusted_key(peer_id: String) -> Result<(), String> {
    let _engine = ENGINE.get().ok_or("Engine not initialized")?;
    // This would need mutable access to engine
    Err("Not implemented - needs mutable engine access".to_string())
}

#[frb]
pub async fn is_peer_trusted(peer_id: String) -> Result<bool, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    Ok(engine.crypto().is_peer_trusted(&peer_id))
}

#[frb]
pub async fn block_peer(peer_id: String) -> Result<(), String> {
    let _engine = ENGINE.get().ok_or("Engine not initialized")?;
    // This would need mutable access to engine
    Err("Not implemented - needs mutable engine access".to_string())
}

#[frb]
pub async fn is_peer_blocked(peer_id: String) -> Result<bool, String> {
    let engine = ENGINE.get().ok_or("Engine not initialized")?;
    Ok(engine.crypto().is_peer_blocked(&peer_id))
}
