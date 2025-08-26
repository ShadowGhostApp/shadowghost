use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_used: chrono::DateTime<chrono::Utc>,
}

pub struct ProfileManager {
    profiles_dir: PathBuf,
}

impl ProfileManager {
    pub fn new() -> Result<Self, String> {
        let profiles_dir = Self::get_profiles_dir()?;
        if !profiles_dir.exists() {
            std::fs::create_dir_all(&profiles_dir)
                .map_err(|e| format!("Failed to create profiles directory: {}", e))?;
        }
        Ok(Self { profiles_dir })
    }

    pub fn list_profiles(&self) -> Result<Vec<Profile>, String> {
        let mut profiles = Vec::new();

        let entries = std::fs::read_dir(&self.profiles_dir)
            .map_err(|e| format!("Failed to read profiles: {}", e))?;

        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    let profile_file = path.join("profile.json");
                    if profile_file.exists() {
                        let content = std::fs::read_to_string(&profile_file)
                            .map_err(|e| format!("Failed to read profile: {}", e))?;
                        let profile: Profile = serde_json::from_str(&content)
                            .map_err(|e| format!("Failed to parse profile: {}", e))?;
                        profiles.push(profile);
                    }
                }
            }
        }

        profiles.sort_by(|a, b| b.last_used.cmp(&a.last_used));
        Ok(profiles)
    }

    pub fn create_profile(&self, name: String) -> Result<Profile, String> {
        let profile = Profile {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            created_at: chrono::Utc::now(),
            last_used: chrono::Utc::now(),
        };

        let profile_path = self.profiles_dir.join(&profile.id);
        std::fs::create_dir_all(&profile_path)
            .map_err(|e| format!("Failed to create profile directory: {}", e))?;

        let profile_file = profile_path.join("profile.json");
        let content = serde_json::to_string_pretty(&profile)
            .map_err(|e| format!("Failed to serialize profile: {}", e))?;
        std::fs::write(&profile_file, content)
            .map_err(|e| format!("Failed to write profile: {}", e))?;

        Ok(profile)
    }

    pub fn get_profile_path(&self, profile_id: &str) -> PathBuf {
        self.profiles_dir.join(profile_id)
    }

    pub fn update_last_used(&self, profile_id: &str) -> Result<(), String> {
        let profile_path = self.get_profile_path(profile_id);
        let profile_file = profile_path.join("profile.json");

        if profile_file.exists() {
            let content = std::fs::read_to_string(&profile_file)
                .map_err(|e| format!("Failed to read profile: {}", e))?;
            let mut profile: Profile = serde_json::from_str(&content)
                .map_err(|e| format!("Failed to parse profile: {}", e))?;

            profile.last_used = chrono::Utc::now();

            let content = serde_json::to_string_pretty(&profile)
                .map_err(|e| format!("Failed to serialize profile: {}", e))?;
            std::fs::write(&profile_file, content)
                .map_err(|e| format!("Failed to write profile: {}", e))?;
        }

        Ok(())
    }

    fn get_profiles_dir() -> Result<PathBuf, String> {
        let base_dir = if cfg!(target_os = "windows") {
            std::env::var("APPDATA")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("C:\\ProgramData"))
        } else if cfg!(target_os = "macos") {
            std::env::var("HOME")
                .map(|h| PathBuf::from(h).join("Library/Application Support"))
                .unwrap_or_else(|_| PathBuf::from("/tmp"))
        } else {
            std::env::var("HOME")
                .map(|h| PathBuf::from(h).join(".local/share"))
                .unwrap_or_else(|_| PathBuf::from("/tmp"))
        };

        Ok(base_dir.join("shadowghost").join("profiles"))
    }
}
