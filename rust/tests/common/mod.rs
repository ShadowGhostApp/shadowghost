pub mod events;

use shadowghost::prelude::*;
use std::sync::Once;

static INIT: Once = Once::new();

pub fn init_test_logging() {
    INIT.call_once(|| {
        env_logger::Builder::from_env("RUST_LOG")
            .filter_level(log::LevelFilter::Debug)
            .is_test(true)
            .init();
    });
}

pub struct TestSetup {
    pub core: ShadowGhostCore,
    test_id: String,
}

impl TestSetup {
    pub async fn new(test_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let test_id = format!("{}-{}", test_name, uuid::Uuid::new_v4());

        let mut core = ShadowGhostCore::new_for_test(&test_id)?;

        core.initialize(Some(test_name.to_string())).await?;
        core.start_server().await?;

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        Ok(Self { core, test_id })
    }

    pub fn get_event_receiver(&self) -> tokio::sync::broadcast::Receiver<AppEvent> {
        self.core.get_event_bus().subscribe()
    }

    pub async fn shutdown(mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.core.shutdown().await?;

        let temp_dir = std::env::temp_dir()
            .join("shadowghost_test")
            .join(&self.test_id);
        if temp_dir.exists() {
            let _ = std::fs::remove_dir_all(&temp_dir);
        }

        Ok(())
    }
}

impl Drop for TestSetup {
    fn drop(&mut self) {
        let temp_dir = std::env::temp_dir()
            .join("shadowghost_test")
            .join(&self.test_id);
        if temp_dir.exists() {
            let _ = std::fs::remove_dir_all(&temp_dir);
        }
    }
}
