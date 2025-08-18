use shadowghost::prelude::*;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

pub struct TestSetup {
    _temp_dir: tempfile::TempDir,
    _config_path: PathBuf,
    pub core: ShadowGhostCore,
}

impl TestSetup {
    pub async fn new(user_name: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let temp_dir = tempfile::tempdir()?;
        let config_path = temp_dir.path().join("config.toml");

        let data_dir = temp_dir.path().join("data");
        fs::create_dir_all(&data_dir)?;
        fs::create_dir_all(data_dir.join("contacts"))?;
        fs::create_dir_all(data_dir.join("chats"))?;
        fs::create_dir_all(data_dir.join("keys"))?;
        fs::create_dir_all(data_dir.join("backups"))?;

        let config_content = format!(
            r#"
[user]
name = "{}"
language = "en"
auto_start_server = true

[network]
default_port = 8000
max_connections = 100
connection_timeout_ms = 30000
heartbeat_interval_ms = 60000

[security]
auto_accept_contacts = false
require_encryption = false
allow_anonymous_contacts = false
max_message_size = 1048576

[storage]
data_dir = "{}"
max_chat_history = 1000
auto_cleanup_days = 90
compress_old_messages = false
"#,
            user_name,
            data_dir.to_string_lossy().replace('\\', "/")
        );

        fs::write(&config_path, config_content)?;

        let mut core = ShadowGhostCore::new(&config_path)?;
        core.initialize(Some(user_name.to_string())).await?;
        core.start_server().await?;

        tokio::time::sleep(Duration::from_millis(500)).await;

        Ok(Self {
            _temp_dir: temp_dir,
            _config_path: config_path,
            core,
        })
    }

    pub async fn shutdown(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.core.shutdown().await?;
        Ok(())
    }
}

pub fn init_test_logging() {
    env_logger::Builder::from_default_env()
        .filter_level(log::LevelFilter::Debug)
        .try_init()
        .ok();
}
