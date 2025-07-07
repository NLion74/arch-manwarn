use std::fs;
use std::path::Path;
use crate::{rss::{self, ManualInterventionResult}};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::config::CONFIG;

fn get_cache_path() -> String {
    std::env::var("ARCH_NEWS_CACHE_PATH")
        .ok()
        .or_else(|| Some(CONFIG.cache_path.clone()))
        .unwrap_or_else(|| "/var/cache/arch-manwarn/last_seen_entries.json".to_string())
}

const CACHE_VERSION: u32 = 1;

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct CachedEntry {
    pub title: String,
    pub summary: String,
    pub link: String,
    pub first_seen: i64,
    pub last_seen: i64,
}

#[derive(Default, Debug, serde::Serialize, serde::Deserialize, Clone)]

pub struct CacheFile {
    pub entries: Vec<CachedEntry>,
    pub cache_version: u32,
}


fn current_unix_time() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs() as i64
}

fn save_cache(cache_path: String, cache_file: CacheFile) {
    if let Some(parent) = Path::new(&cache_path).parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("Failed to create cache directory {:?}: {}", parent, e);
        }
    }
    if let Err(e) = fs::write(&cache_path, serde_json::to_string_pretty(&cache_file).unwrap()) {
        eprintln!("Failed to write cache file {}: {}", cache_path, e);
        eprintln!("Try running the program as root or with sudo if you want to use /var/cache.");
    }
}

pub fn check_new_entries() -> Vec<CachedEntry> {
    let result: ManualInterventionResult = rss::check_for_manual_intervention();
    let cache_path = get_cache_path();

    // Determining whether this is the first run
    // by checking if the cache file exists
    let mut first_run = !Path::new(&cache_path).exists();

    // Load previously cached entries
    let mut cache_file: CacheFile = if let Ok(data) = fs::read_to_string(&cache_path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        CacheFile {
            entries: Vec::new(),
            cache_version: CACHE_VERSION,
        }
    };

    // Ensure the cache version is correct
    if cache_file.cache_version != CACHE_VERSION {
        eprintln!("Cache version mismatch. Resetting cache.");
        cache_file = CacheFile {
            entries: Vec::new(),
            cache_version: CACHE_VERSION,
        };
        first_run = true; // Treat as first run if cache version is reset
    }

    // Create a mutable reference to the entries vector
    // to avoid confusion with the cache_file variable
    let cached_entries = &mut cache_file.entries;

    let mut new_entries = Vec::new();
    let mut cache_changed = false;
    let now = current_unix_time();

    for entry in &result.entries {
        // Compare the title of the new entry with cached entries
        // If the title is not found in cached entries, push it
        // to new_entries and cached_entries
        if !cached_entries.iter().any(|e| e.title == entry.title) {
            let new = CachedEntry {
                title: entry.title.clone(),
                summary: entry.summary.clone(),
                link: entry.link.clone(),
                first_seen: now,
                last_seen: now,
            };
            new_entries.push(new.clone());
            cached_entries.push(new);
            cache_changed = true;
        } else {
            // If the entry already exists in the cache,
            // update its last_seen timestamp
            if let Some(cached_entry) = cached_entries.iter_mut().find(|e| e.title == entry.title) {
                cached_entry.last_seen = now;
                cache_changed = true;
            }
        }
    }

    // Retain only cached entries that are not over CONFIG.prune_missing_days old
    // and have not been seen in the feed entries in the last CONFIG.prune_age_days days
    let prune_threshold_missing = now - (CONFIG.prune_missing_days as i64) * 24 * 3600;
    let prune_threshold_age = now - (CONFIG.prune_age_days as i64) * 24 * 3600;

    let before_len = cached_entries.len();

    cached_entries.retain(|e| {
        // Keep entries unless both conditions to prune are met
        !(e.last_seen < prune_threshold_missing && e.first_seen < prune_threshold_age)
    });

    if cached_entries.len() != before_len {
        cache_changed = true;
    }

    // If updated, save the cache
    if cache_changed {
        save_cache(cache_path, cache_file);
    }

    // If this is the first run, return an empty vector
    // Otherwise, return the new entries found
    if first_run { Vec::new() } else { new_entries }
}