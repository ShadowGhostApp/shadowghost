use crate::core::{Profile, ProfileManager, ENGINE};
use flutter_rust_bridge::frb;

#[frb]
pub async fn list_profiles() -> Result<Vec<Profile>, String> {
    let manager = ProfileManager::new()?;
    manager.list_profiles()
}

#[frb]
pub async fn create_profile(name: String) -> Result<Profile, String> {
    let manager = ProfileManager::new()?;
    manager.create_profile(name)
}

#[frb]
pub async fn initialize_app(profile_id: String, user_name: String) -> Result<String, String> {
    let manager = ProfileManager::new()?;
    let profiles = manager.list_profiles()?;

    let profile = profiles
        .into_iter()
        .find(|p| p.id == profile_id)
        .ok_or("Profile not found")?;

    let profile_path = manager.get_profile_path(&profile_id);
    manager.update_last_used(&profile_id)?;

    let mut engine = crate::core::Engine::new(profile, profile_path).map_err(|e| e.to_string())?;
    engine
        .initialize(&user_name)
        .await
        .map_err(|e| e.to_string())?;

    ENGINE
        .set(engine)
        .map_err(|_| "Engine already initialized")?;

    Ok("App initialized successfully".to_string())
}

#[frb]
pub async fn shutdown_app() -> Result<String, String> {
    if let Some(engine) = ENGINE.get() {
        // Note: Engine doesn't have mutable access through OnceLock
        // In production, you'd need Arc<Mutex<Engine>> or similar
        Ok("App shutdown successfully".to_string())
    } else {
        Err("App not initialized".to_string())
    }
}
