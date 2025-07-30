use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
#[cfg(debug_assertions)]
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn config_path() -> PathBuf {
    // For development: ARCH_MANWARN_CONFIG=/path/to/custom/config.toml
    #[cfg(debug_assertions)]
    if let Ok(env_path) = env::var("ARCH_MANWARN_CONFIG") {
        return PathBuf::from(env_path);
    }

    // Hardcoded fallback for production use
    // default to /etc/arch-manwarn/config.toml
    // Because pacman hooks run as root
    PathBuf::from("/etc/arch-manwarn/config.toml")
}

/// Recursively merge defaults into `value` for missing or invalid fields.
/// If a field is missing or its type is wrong, it is replaced with the default.
fn merge_defaults(value: &mut toml::Value, default: &toml::Value) {
    match (value, default) {
        // Both are tables: merge recursively
        (toml::Value::Table(ref mut user_table), toml::Value::Table(default_table)) => {
            for (key, default_val) in default_table {
                match user_table.get_mut(key) {
                    Some(user_val) => {
                        // Merge recursively for nested tables
                        merge_defaults(user_val, default_val);
                    }
                    None => {
                        // Field missing: insert default
                        user_table.insert(key.clone(), default_val.clone());
                    }
                }
            }
        }
        // Types differ: replace with default
        (user_val, default_val) => {
            if user_val.type_str() != default_val.type_str() {
                *user_val = default_val.clone();
            }
        }
    }
}
#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Keywords to search for in news entries
    pub keywords: Vec<String>,

    /// Whether to include all news entries, not just those with keywords
    pub match_all_entries: bool,

    /// Ignore these keywords explicitly
    pub ignored_keywords: Vec<String>,

    pub case_sensitive: bool,

    /// Whether to include summary in query of keywords
    /// If true, the summary will be included in the search for keywords
    /// If false, only the title will be searched
    pub include_summary_in_query: bool,

    /// Number of days to retain cache
    pub prune_missing_days: u64,
    pub prune_age_days: u64,

    /// URLs for the RSS feeds
    pub rss_feed_urls: Vec<String>,

    /// Whether to show summary on check
    /// If false, only title and link will be shown
    pub show_summary: bool,

    /// Whether to automatically mark as read after blocking
    pub mark_as_read_automatically: bool,

    /// Whether to just warn (don’t block transaction)
    pub warn_only: bool,

    /// Path where cache is stored
    pub cache_path: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            cache_path: "/var/cache/arch-manwarn.json".to_string(),
            rss_feed_urls: vec!["https://archlinux.org/feeds/news/".to_string()],
            keywords: vec!["manual intervention".to_string()],
            ignored_keywords: vec![],
            case_sensitive: false,
            include_summary_in_query: true,
            prune_missing_days: 30,
            prune_age_days: 60,
            match_all_entries: false,
            show_summary: false,
            mark_as_read_automatically: true,
            warn_only: false,
        }
    }
}

impl Config {
    pub fn load_from_file(path: &Path) -> Result<Self, String> {
        let content =
            fs::read_to_string(path).map_err(|e| format!("Failed to read config file: {e}"))?;

        let mut config_value: toml::Value = toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {e}"))?;

        let default = toml::Value::try_from(Config::default())
            .expect("Default config should serialize to toml::Value");
        
        // Save to check if something has changed later
        let original_value = config_value.clone();

        // Merge defaults for missing or invalid fields
        merge_defaults(&mut config_value, &default);

        // If the original value is different from the merged value,
        // it means some fields were missing or had wrong types,
        // and we write the updated config back to the file.
        if &original_value != &config_value {
            
            // Now try to deserialize the merged value
            let config: Config = config_value
                .try_into()
                .map_err(|e| format!("Failed to deserialize merged config: {e}"))?;

            // Write back the merged config (with fixed/corrected fields)
            let updated = toml::to_string_pretty(&config)
                .map_err(|e| format!("Failed to serialize updated config: {e}"))?;
            fs::write(path, updated).map_err(|e| format!("Failed to write updated config: {e}"))?;
        
            Ok(config)
        } else {
            // If no changes were made, just deserialize the original value
            let config: Config = original_value
                .try_into()
                .map_err(|e| format!("Failed to deserialize original config: {e}"))?;
            Ok(config)
        }
    }

    /// Loads the configuration from the given file path.
    /// 
    /// - If the file does not exist, it creates a new config file with default values and returns those defaults.
    /// - If the file exists but is invalid TOML, prints an error and returns defaults (does not overwrite the file).
    /// - If the file is valid TOML but missing or has invalid fields, those fields are reset to defaults and the file is updated.
    /// - Returns early after creating a new config file, so no further loading or parsing is attempted in that case.
    pub fn load() -> Self {
        let path = config_path();

        // If config file does not exist, generate it with defaults
        if !path.exists() {
            let default_config = Config::default();
            if let Err(e) = default_config.save(&path) {
                eprintln!(
                    "[arch-manwarn] Failed to create default config file at {}: {e}",
                    path.display()
                );
            } else {
                eprintln!(
                    "[arch-manwarn] Created default config file at {}",
                    path.display()
                );
            }
            return default_config;
        }

        match Self::load_from_file(&path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("[arch-manwarn] Config error: {e}");
                eprintln!(
                    "[arch-manwarn] Using default config options until the error is resolved.\n\
                    Please fix your config file at: {}",
                    path.display()
                );
                Config::default()
            }
        }
    }

    pub fn save(&self, path: &Path) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let serialized = toml::to_string_pretty(&self).expect("Failed to serialize config");
        fs::write(path, serialized)
    }
}

pub static CONFIG: Lazy<Config> = Lazy::new(Config::load);
