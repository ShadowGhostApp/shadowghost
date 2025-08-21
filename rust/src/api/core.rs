use crate::core::ShadowGhostCore;
use flutter_rust_bridge::frb;
use std::sync::{Arc, LazyLock};
use tokio::sync::Mutex;

pub static CORE: LazyLock<Mutex<Option<Arc<Mutex<ShadowGhostCore>>>>> =
    LazyLock::new(|| Mutex::const_new(None));

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

#[frb(sync)]
pub fn shutdown_core() -> Result<String, String> {
    let rt = tokio::runtime::Handle::current();
    let mut core_guard = rt.block_on(CORE.lock());
    if core_guard.is_some() {
        *core_guard = None;
        Ok("Core shutdown successfully".to_string())
    } else {
        Err("Core not initialized".to_string())
    }
}

#[frb(sync)]
pub fn is_core_initialized() -> bool {
    let rt = tokio::runtime::Handle::current();
    rt.block_on(CORE.lock()).is_some()
}

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
