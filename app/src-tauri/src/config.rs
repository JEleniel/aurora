use config::{Config, File, FileFormat};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// User preferences and application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Theme preference: "light", "dark", or "auto"
    pub theme: String,
    /// Last opened file path
    pub last_opened_file: Option<String>,
    /// Window width in pixels
    pub window_width: u32,
    /// Window height in pixels
    pub window_height: u32,
    /// Whether to restore last window position
    pub restore_window_position: bool,
    /// Last window X position
    pub window_x: Option<i32>,
    /// Last window Y position
    pub window_y: Option<i32>,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            theme: "auto".to_string(),
            last_opened_file: None,
            window_width: 1200,
            window_height: 800,
            restore_window_position: true,
            window_x: None,
            window_y: None,
        }
    }
}

pub struct ConfigManager {
    config_path: PathBuf,
    config: Config,
}

impl ConfigManager {
    /// Initialize the config manager and ensure config directory exists
    pub fn new() -> Result<Self, String> {
        let config_dir = dirs::config_dir()
            .ok_or("Could not determine config directory".to_string())?
            .join("aurora");

        std::fs::create_dir_all(&config_dir)
            .map_err(|e| format!("Failed to create config directory: {}", e))?;

        let config_path = config_dir.join("config.json");

        let config = Config::builder()
            .add_source(
                File::new(
                    config_path.to_str().unwrap_or("config.json"),
                    FileFormat::Json,
                )
                .required(false),
            )
            .build()
            .map_err(|e| format!("Failed to build config: {}", e))?;

        log::info!("Config path: {:?}", config_path);

        Ok(ConfigManager {
            config_path,
            config,
        })
    }

    /// Load configuration as AppConfig struct, or return default if not found
    pub fn load(&self) -> Result<AppConfig, String> {
        match self.config.clone().try_deserialize::<AppConfig>() {
            Ok(config) => Ok(config),
            Err(_) => {
                log::info!(
                    "No config found or parse error, using defaults at: {:?}",
                    self.config_path
                );
                Ok(AppConfig::default())
            }
        }
    }

    /// Save configuration to disk
    pub fn save(&self, config: &AppConfig) -> Result<(), String> {
        let json = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        std::fs::write(&self.config_path, json)
            .map_err(|e| format!("Failed to write config file: {}", e))?;
        log::debug!("Config saved to: {:?}", self.config_path);
        Ok(())
    }

    /// Get the config directory path
    pub fn get_config_dir(&self) -> PathBuf {
        self.config_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_default()
    }

    /// Get the config file path
    pub fn get_config_path(&self) -> &PathBuf {
        &self.config_path
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        match Self::new() {
            Ok(manager) => manager,
            Err(e) => {
                log::error!("Failed to initialize default config manager: {}", e);
                // Create minimal config manager with default config
                let config_dir = dirs::config_dir()
                    .map(|d| d.join("aurora"))
                    .unwrap_or_else(|| std::path::PathBuf::from("."));
                ConfigManager {
                    config_path: config_dir.join("config.json"),
                    config: Config::builder().build().unwrap_or_default(),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.theme, "auto");
        assert_eq!(config.window_width, 1200);
        assert_eq!(config.window_height, 800);
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.theme, deserialized.theme);
    }
}
