use std::fs;
use std::path::Path;
use crate::{rss::{self, ManualInterventionResult}};

fn get_cache_path() -> String {
    std::env::var("ARCH_NEWS_CACHE_PATH")
        .unwrap_or_else(|_| "/var/cache/arch-manwarn/last_seen_entries.json".to_string())
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct CachedEntry {
    pub title: String,
    pub summary: String,
    pub link: String,
}

fn save_cache(cache_path: &str, entries: &[CachedEntry]) {
    if let Some(parent) = Path::new(cache_path).parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            eprintln!("Failed to create cache directory {:?}: {}", parent, e);
        }
    }
    if let Err(e) = fs::write(cache_path, serde_json::to_string_pretty(entries).unwrap()) {
        eprintln!("Failed to write cache file {}: {}", cache_path, e);
        eprintln!("Try running the program as root or with sudo if you want to use /var/cache.");
    }
}

pub fn check_new_entries() -> Vec<CachedEntry> {
    let result: ManualInterventionResult = rss::check_for_manual_intervention();
    let cache_path = get_cache_path();

    // Determining whether this is the first run
    // by checking if the cache file exists
    let first_run = !Path::new(&cache_path).exists();

    // Load previously cached entries
    let mut cached_entries: Vec<CachedEntry> = if let Ok(data) = fs::read_to_string(&cache_path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        Vec::new()
    };

    let mut new_entries = Vec::new();

    let mut cache_changed = false;

    for entry in &result.entries {
        // Compare the title of the new entry with cached entries
        // If the title is not found in cached entries, push it
        // to new_entries and cached_entries
        if !cached_entries.iter().any(|e| e.title == entry.title) {
            let new = CachedEntry {
                title: entry.title.clone(),
                summary: entry.summary.clone(),
                link: entry.link.clone(),
            };
            new_entries.push(new.clone());
            cached_entries.push(new);
            cache_changed = true;
        }
    }

    // If updated, save the cache
    if cache_changed {
        save_cache(&cache_path, &cached_entries);
    }

    // If this is the first run, return an empty vector
    // Otherwise, return the new entries found
    if first_run { Vec::new() } else { new_entries }
}