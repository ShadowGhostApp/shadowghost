use crate::prelude::*;
use flutter_rust_bridge::frb;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex};

pub static CORE: LazyLock<Mutex<Option<Arc<ShadowGhostCore>>>> = LazyLock::new(|| Mutex::new(None));

pub async fn initialize_core() -> Result<String, String> {
    let app_data_dir = std::env::temp_dir().join("shadowghost_flutter");
    let config_path = app_data_dir.join("config.toml");

    match ShadowGhostCore::new(&config_path) {
        Ok(mut core) => match core.initialize(Some("Flutter User".to_string())).await {
            Ok(_) => {
                *CORE.lock().unwrap() = Some(Arc::new(core));
                Ok("Core initialized successfully".to_string())
            }
            Err(e) => Err(format!("Failed to initialize core: {}", e)),
        },
        Err(e) => Err(format!("Failed to create core: {}", e)),
    }
}

#[frb(sync)]
pub fn shutdown_core() -> Result<String, String> {
    let mut core_guard = CORE.lock().unwrap();
    if core_guard.is_some() {
        *core_guard = None;
        Ok("Core shutdown successfully".to_string())
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb(sync)]
pub fn is_core_initialized() -> bool {
    CORE.lock().unwrap().is_some()
}

#[frb(sync)]
pub fn shutdown_core() -> Result<String, String> {
    let mut core_guard = CORE.lock().unwrap();
    if core_guard.is_some() {
        *core_guard = None;
        Ok("Core shutdown successfully".to_string())
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb(sync)]
pub fn is_core_initialized() -> bool {
    CORE.lock().unwrap().is_some()
}
