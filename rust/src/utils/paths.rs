use std::path::PathBuf;

pub struct DataPaths;

impl DataPaths {
    pub fn get_app_data_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let data_dir = if cfg!(target_os = "windows") {
            std::env::var("APPDATA")
                .map(PathBuf::from)
                .or_else(|_| {
                    std::env::var("USERPROFILE")
                        .map(|p| PathBuf::from(p).join("AppData").join("Roaming"))
                })
                .unwrap_or_else(|_| PathBuf::from("C:\\Users\\Default\\AppData\\Roaming"))
        } else if cfg!(target_os = "macos") {
            std::env::var("HOME")
                .map(|p| PathBuf::from(p).join("Library").join("Application Support"))
                .unwrap_or_else(|_| PathBuf::from("/Users/Shared/Library/Application Support"))
        } else {
            std::env::var("XDG_DATA_HOME")
                .map(PathBuf::from)
                .or_else(|_| {
                    std::env::var("HOME").map(|p| PathBuf::from(p).join(".local").join("share"))
                })
                .unwrap_or_else(|_| PathBuf::from("/tmp"))
        };

        let app_data_dir = data_dir.join("ShadowGhost");

        if !app_data_dir.exists() {
            std::fs::create_dir_all(&app_data_dir)?;
        }

        Ok(app_data_dir)
    }

    pub fn get_config_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_dir = Self::get_app_data_dir()?.join("config");

        if !config_dir.exists() {
            std::fs::create_dir_all(&config_dir)?;
        }

        Ok(config_dir)
    }

    pub fn get_chats_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let chats_dir = Self::get_app_data_dir()?.join("chats");

        if !chats_dir.exists() {
            std::fs::create_dir_all(&chats_dir)?;
        }

        Ok(chats_dir)
    }

    pub fn get_contacts_file() -> Result<PathBuf, Box<dyn std::error::Error>> {
        Ok(Self::get_config_dir()?.join("contacts.json"))
    }

    pub fn get_peer_config_file() -> Result<PathBuf, Box<dyn std::error::Error>> {
        Ok(Self::get_config_dir()?.join("peer.json"))
    }

    pub fn get_keys_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let keys_dir = Self::get_app_data_dir()?.join("keys");

        if !keys_dir.exists() {
            std::fs::create_dir_all(&keys_dir)?;
        }

        Ok(keys_dir)
    }

    pub fn get_temp_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let temp_dir = Self::get_app_data_dir()?.join("temp");

        if !temp_dir.exists() {
            std::fs::create_dir_all(&temp_dir)?;
        }

        Ok(temp_dir)
    }

    pub fn get_config_file() -> Result<PathBuf, Box<dyn std::error::Error>> {
        Ok(Self::get_app_data_dir()?.join("config.toml"))
    }

    pub fn get_logs_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let logs_dir = Self::get_app_data_dir()?.join("logs");

        if !logs_dir.exists() {
            std::fs::create_dir_all(&logs_dir)?;
        }

        Ok(logs_dir)
    }

    pub fn get_backups_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let backups_dir = Self::get_app_data_dir()?.join("backups");

        if !backups_dir.exists() {
            std::fs::create_dir_all(&backups_dir)?;
        }

        Ok(backups_dir)
    }
}
