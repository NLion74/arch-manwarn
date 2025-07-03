use std::fs;
use std::path::Path;
use crate::{cache, rss::{self, ManualInterventionResult, NewsEntry}};

fn get_cache_path() -> String {
    std::env::var("ARCH_NEWS_CACHE_PATH")
        .unwrap_or_else(|_| "/var/cache/arch-manwarn/last_seen_entries.json".to_string())
}

#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct CachedEntry {
    pub title: String,
    pub summary: String,
    pub is_new: bool,
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
    // Check the RSS feed for manual intervention entries with rss.check_for_manual_intervention();
    // Have a struct with a list of these entries and a boolean if they were already seen. Then return only the new entries.
    // Save this list to a cache file.
    // The boolean should only be updated once the user has confirmed they have read the entry. This will be handled later in the main.rs file.

    // Get current manual intervention entries
    let result: ManualInterventionResult = rss::check_for_manual_intervention();
    let cache_path = get_cache_path();

    // Load the cache
    let mut cache_initialized = false;
    let cached_entries: Vec<CachedEntry> = if let Ok(data) = fs::read_to_string(&cache_path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        cache_initialized = true;
        result.entries.iter().map(|entry| CachedEntry {
            title: entry.title.clone(),
            summary: entry.summary.clone(),
            is_new: false,
        }).collect()
    };

    // Find new entries not in the cache
    let mut new_entries = Vec::new();
    for entry in &result.entries {
        if !cached_entries.iter().any(|e| e.title == entry.title) {
            new_entries.push(CachedEntry {
                title: entry.title.clone(),
                summary: entry.summary.clone(),
                is_new: true,
            });
        }
    }

    // Combine updated cache with new entries
    let mut updated_cache = cached_entries.clone();
    updated_cache.extend(new_entries.iter().cloned());

    // Save the updated cache
    save_cache(&cache_path, &updated_cache);

    // Return only new entries
    updated_cache.into_iter().filter(|e| e.is_new).collect()
}

pub fn mark_entries_as_seen(entries: &[CachedEntry]) {
    let cache_path = get_cache_path();

    // Load the cache
    let mut cached_entries: Vec<CachedEntry> = if let Ok(data) = fs::read_to_string(&cache_path) {
        serde_json::from_str(&data).unwrap_or_default()
    } else {
        return;
    };

    // Set is_new to false for entries the user has seen
    for seen_entry in entries {
        if let Some(cached) = cached_entries.iter_mut().find(|e| e.title == seen_entry.title) {
            cached.is_new = false;
        }
    }

    // Save the updated cache
    save_cache(&cache_path, &cached_entries);
}