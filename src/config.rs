use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
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
    /// URLs for the RSS feeds(take content as summary rather description)
    pub rss_feed_urls_content: Vec<String>,

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
            rss_feed_urls_content: vec![],
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

        let config: Config =
            toml::from_str(&content).map_err(|e| format!("Failed to parse config file: {e}"))?;

        let updated = toml::to_string_pretty(&config)
            .map_err(|e| format!("Failed to serialize updated config: {e}"))?;
        fs::write(path, updated).map_err(|e| format!("Failed to write updated config: {e}"))?;

        Ok(config)
    }

    pub fn load() -> Self {
        let path = config_path();

        match Self::load_from_file(&path) {
            Ok(config) => config,
            Err(e) => {
                eprintln!("[arch-manwarn] Config error: {e}");
                eprintln!("[arch-manwarn] Using default config and regenerating...");

                let default = Config::default();
                if let Err(write_err) = default.save(&path) {
                    eprintln!("[arch-manwarn] Failed to write default config: {write_err}");
                }

                default
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
