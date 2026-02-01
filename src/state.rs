use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use crate::cache::CachedEntry;
use crate::config::CONFIG;


#[derive(Debug, Serialize, Deserialize)]
pub struct StateEntry {
    pub title: String,
    pub link: String,
    pub summary: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct StateFile {
    pub timestamp: u64,
    pub matching_entries_count: usize,
    pub entries: Vec<StateEntry>,
}


impl StateFile {
    pub fn new(entries: &[CachedEntry]) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs();


        let state_entries: Vec<StateEntry> = entries
            .iter()
            .map(|e| StateEntry {
                title: e.title.clone(),
                link: e.link.clone(),
                summary: e.summary.clone(),
            })
            .collect();


        Self {
            timestamp,
            matching_entries_count: entries.len(),
            entries: state_entries,
        }
    }

    pub fn get_path() -> Option<String> {
        // For development: ARCH_MANWARN_STATE_FILE=/path/to/custom/state.json
        #[cfg(debug_assertions)]
        if let Ok(env_path) = std::env::var("ARCH_MANWARN_STATE_FILE") {
            if !env_path.is_empty() {
                return Some(env_path);
            }
        }
        
        // Return None if path is None or empty string
        CONFIG.state_file_path.clone().filter(|s| !s.is_empty())
    }



    pub fn write(&self) -> std::io::Result<()> {
        let Some(state_path) = Self::get_path() else {
            return Ok(());
        };

        let path = Path::new(&state_path);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        fs::write(path, json)
    }
}
