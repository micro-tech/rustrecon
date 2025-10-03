use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const DEFAULT_CONFIG_FILE_NAME: &str = "rustrecon_config.toml";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub llm: Option<LlmConfig>,
    pub rate_limiting: Option<RateLimitConfig>,
    pub cache: Option<CacheConfig>,
    // Add other configuration sections as needed, e.g., [scanner], [report]
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LlmConfig {
    pub gemini_api_key: String,
    pub gemini_api_endpoint: String,
    pub gemini_model: Option<String>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub min_request_interval_seconds: Option<f32>,
    pub max_requests_per_minute: Option<u32>,
    pub enable_rate_limiting: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct CacheConfig {
    pub enabled: Option<bool>,
    pub database_path: Option<String>,
    pub max_age_days: Option<u32>,
    pub auto_cleanup: Option<bool>,
}

impl Config {
    /// Loads the configuration from a specified path or default locations.
    pub fn load_from_path(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)?;
        Ok(config)
    }

    /// Tries to load the configuration from common default paths.
    /// Order of precedence: user config directory, local data directory, home directory, current directory.
    pub fn load_from_default_paths() -> Result<Self> {
        // 1. User config directory (e.g., %APPDATA%\RustRecon on Windows)
        if let Some(mut config_dir) = dirs::config_dir() {
            config_dir.push("RustRecon");
            config_dir.push(DEFAULT_CONFIG_FILE_NAME);
            if config_dir.exists() {
                println!("Loading config from: {}", config_dir.display());
                return Config::load_from_path(&config_dir);
            }
        }

        // 2. Local app data directory (e.g., %LOCALAPPDATA%\RustRecon on Windows) - Fallback
        if let Some(mut local_data_dir) = dirs::data_local_dir() {
            local_data_dir.push("RustRecon");
            local_data_dir.push(DEFAULT_CONFIG_FILE_NAME);
            if local_data_dir.exists() {
                println!("Loading config from: {}", local_data_dir.display());
                return Config::load_from_path(&local_data_dir);
            }
        }

        // 3. Home directory fallback (e.g., ~/.rustrecon on Linux/macOS)
        if let Some(mut home_dir) = dirs::home_dir() {
            home_dir.push(".rustrecon");
            home_dir.push(DEFAULT_CONFIG_FILE_NAME);
            if home_dir.exists() {
                println!("Loading config from: {}", home_dir.display());
                return Config::load_from_path(&home_dir);
            }
        }

        // 4. Current directory (last resort fallback)
        let current_dir_path = PathBuf::from(DEFAULT_CONFIG_FILE_NAME);
        if current_dir_path.exists() {
            println!("Loading config from: {}", current_dir_path.display());
            return Config::load_from_path(&current_dir_path);
        }

        anyhow::bail!(
            "No configuration file found. Please run `rustrecon init` or create `{}` manually.",
            DEFAULT_CONFIG_FILE_NAME
        )
    }

    /// Generates a default configuration file at the specified path.
    pub fn generate_default_config(path: PathBuf) -> Result<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            println!("📁 Creating directory: {}", parent.display());
            fs::create_dir_all(parent)?;
            println!("✓ Directory created successfully");
        }

        let default_config = Self {
            llm: Some(LlmConfig {
                gemini_api_key: "PASTE_YOUR_GEMINI_API_KEY_HERE".to_string(),
                gemini_api_endpoint: "https://generativelanguage.googleapis.com".to_string(),
                gemini_model: Some("gemini-2.5-pro-latest".to_string()),
                temperature: Some(0.7),
                max_tokens: Some(1024),
            }),
            rate_limiting: Some(RateLimitConfig {
                min_request_interval_seconds: Some(2.0),
                max_requests_per_minute: Some(20),
                enable_rate_limiting: Some(true),
            }),
            cache: Some(CacheConfig {
                enabled: Some(true),
                database_path: None,    // Will use default location
                max_age_days: Some(90), // Keep cache for 3 months
                auto_cleanup: Some(true),
            }),
        };

        let toml_string = toml::to_string_pretty(&default_config)?;
        println!("📄 Writing configuration file: {}", path.display());
        fs::write(&path, toml_string)?;
        println!("✓ Configuration file created successfully");
        println!();
        println!("📍 Configuration stored at:");
        println!("   {}", path.display());
        println!();
        println!("🔑 IMPORTANT: You need to add your Gemini API key!");
        println!("   1. Get your API key from: https://aistudio.google.com/app/apikey");
        println!("   2. Edit the config file and replace: PASTE_YOUR_GEMINI_API_KEY_HERE");
        println!("   3. Test your setup with: rustrecon test");
        Ok(())
    }

    /// Gets the default configuration path for the current user
    pub fn get_default_config_path() -> Result<PathBuf> {
        // Try multiple locations in order of preference

        // 1. User config directory (e.g., %APPDATA%\RustRecon on Windows)
        if let Some(mut config_dir) = dirs::config_dir() {
            config_dir.push("RustRecon");
            config_dir.push(DEFAULT_CONFIG_FILE_NAME);
            return Ok(config_dir);
        }

        // 2. Local app data directory (e.g., %LOCALAPPDATA%\RustRecon on Windows) - Fallback
        if let Some(mut local_data_dir) = dirs::data_local_dir() {
            local_data_dir.push("RustRecon");
            local_data_dir.push(DEFAULT_CONFIG_FILE_NAME);
            return Ok(local_data_dir);
        }

        // 3. Home directory fallback (e.g., ~/.rustrecon on Linux/macOS)
        if let Some(mut home_dir) = dirs::home_dir() {
            home_dir.push(".rustrecon");
            home_dir.push(DEFAULT_CONFIG_FILE_NAME);
            return Ok(home_dir);
        }

        // 4. Last resort: current directory
        Ok(PathBuf::from(DEFAULT_CONFIG_FILE_NAME))
    }
}
