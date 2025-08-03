use crate::config::CONFIG;
use crate::rss;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_cache_path() -> PathBuf {
    #[cfg(debug_assertions)]
    {
        std::env::var("ARCH_NEWS_CACHE_PATH")
            .ok()
            .map(PathBuf::from)
            .unwrap_or_else(|| CONFIG.cache_path.clone().into())
    }
    
    #[cfg(not(debug_assertions))]
    {
        CONFIG.cache_path.clone().into()
    }

}

const CACHE_VERSION: u32 = 1;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct CachedEntry {
    pub title: String,
    pub summary: String,
    pub link: String,
    pub first_seen: u64,
    pub last_seen: u64,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct CacheFile {
    pub entries: Vec<CachedEntry>,
    pub cache_version: u32,

    #[serde(default)]
    pub last_successful_request: Option<SystemTime>,
}

impl Default for CacheFile {
    fn default() -> Self {
        CacheFile {
            entries: Vec::new(),
            cache_version: CACHE_VERSION,
            last_successful_request: None,
        }
    }
}

pub fn current_unix_time() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs()
}

fn save_cache(cache_path: &Path, cache_file: CacheFile) {
    if let Some(parent) = cache_path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("Failed to create cache directory {parent:?}: {e}");
        }
    }
    if let Err(e) = fs::write(
        cache_path,
        serde_json::to_string_pretty(&cache_file).unwrap(),
    ) {
        eprintln!("Failed to write cache file {}: {e}", cache_path.display());
        eprintln!("Try running the program as root or with sudo if you want to use /var/cache.");
    }
}

pub fn load_cache(cache_path: &Path) -> CacheFile {
    // Load previously cached entries
    let cache_file: CacheFile = if let Ok(data) = fs::read_to_string(cache_path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        CacheFile::default()
    };

    cache_file
}

pub fn check_new_entries(force_mark_as_read: bool) -> Vec<CachedEntry> {
    let result = rss::check_for_manual_intervention();
    let cache_path = get_cache_path();

    // Determining whether this is the first run
    // by checking if the cache file exists
    let first_run = !cache_path.exists();
    let mut cache_file = load_cache(&cache_path);

    // Only update cache if the result contains a successful request
    if let Some(success_timestamp) = result.last_successful_request {
        cache_file.last_successful_request = Some(success_timestamp);
    } else
    // Check whether the last successful request is older than 1 day
    if let Some(last_success) = cache_file.last_successful_request {
        if let Ok(duration) = last_success.elapsed() {
            let seconds = duration.as_secs_f64();
            if seconds > 86400.0 {
                let days = seconds / 86400.0;
                eprintln!(
                    "Warning: last successful connection to the RSS feed(s) was {days:.1} days ago."
                );
            }
        }
    } else {
        eprintln!("Warning: never successfully connected to the RSS feed(s) yet.");
    }

    // Create a mutable reference to the entries vector
    // to avoid confusion with the cache_file variable
    let cached_entries = &mut cache_file.entries;

    let mut new_entries = Vec::new();
    let mut cache_changed = false;
    let now = current_unix_time();

    for entry in result.entries {
        // Compare the title of the new entry with cached entries
        if cached_entries.iter().any(|e| e.title == entry.title) {
            // If the entry already exists in the cache,
            // update its last_seen timestamp
            if let Some(cached_entry) = cached_entries.iter_mut().find(|e| e.title == entry.title) {
                cached_entry.last_seen = now;
                cache_changed = true;
            }
        } else {
            // If the title is not found in cached entries, push it
            // to new_entries and cached_entries
            let new = CachedEntry {
                title: entry.title,
                summary: entry.summary,
                link: entry.link,
                first_seen: now,
                last_seen: now,
            };
            if CONFIG.mark_as_read_automatically || force_mark_as_read {
                cached_entries.push(new.clone());
            }
            new_entries.push(new);
            cache_changed = true;
        }
    }

    {
        // Retain only cached entries that are not over CONFIG.prune_missing_days old
        // and have not been seen in the feed entries in the last CONFIG.prune_age_days days
        let prune_threshold_missing = now.saturating_sub((CONFIG.prune_missing_days) * 24 * 3600);
        let prune_threshold_age = now.saturating_sub((CONFIG.prune_age_days) * 24 * 3600);

        let before_len = cached_entries.len();

        cached_entries.retain(|e| {
            // Keep entries unless both conditions to prune are met
            !(e.last_seen < prune_threshold_missing && e.first_seen < prune_threshold_age)
        });

        if cached_entries.len() != before_len {
            cache_changed = true;
        }
    }

    // If updated, save the cache
    if cache_changed {
        save_cache(&cache_path, cache_file);
    }

    // If this is the first run, return an empty vector
    // Otherwise, return the new entries found
    if first_run {
        Vec::new()
    } else {
        new_entries
    }
}
