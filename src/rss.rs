use feed_rs::parser;
use reqwest::blocking::get;
use std::io::Read;

#[derive(Debug)]
pub struct NewsEntry {
    pub title: String,
    pub summary: String,
    pub link: String,
}

#[derive(Debug)]
pub struct ManualInterventionResult {
    pub entries: Vec<NewsEntry>,
}

impl Clone for NewsEntry {
    fn clone(&self) -> Self {
        NewsEntry {
            title: self.title.clone(),
            summary: self.summary.clone(),
            link: self.link.clone(),
        }
    }
}

pub fn check_for_manual_intervention() -> ManualInterventionResult {
    // This gives us a vector of NewsEntry structs from the archlinux.org RSS feed
    let entries: Vec<NewsEntry> = get_entries();

    // Check for entries with keywords that indicate manual intervention
    let keywords = ["manual intervention", "action required", "attention", "intervention"];
    let mut found_entries = Vec::new();

    for entry in &entries {
        let text = format!("{}", entry.title.to_lowercase());
        if keywords.iter().any(|kw| text.contains(kw)) {
            found_entries.push(entry.clone());
        }
    }

    ManualInterventionResult {
        entries: found_entries,
    }
}

pub fn get_entries() -> Vec<NewsEntry> {
    let mut content = String::new();
    if let Ok(mut response) = get("https://archlinux.org/feeds/news/") {
        if response.read_to_string(&mut content).is_ok() {
            if let Ok(feed) = parser::parse(content.as_bytes()) {
                let mut entries = Vec::new();
                for entry in feed.entries {
                    let title = entry.title.as_ref().map_or("[No title provided]", |t| &t.content);
                    let summary = entry.summary.as_ref().map_or("[No summary provided]", |s| &s.content);
                    let link = entry.links.get(0).map_or("https://archlinux.org/news/", |l| &l.href);

                    entries.push(NewsEntry {
                        title: title.to_string(),
                        summary: summary.to_string(),
                        link: link.to_string(),
                    });
                }
                return entries;
            }
        }
    }

    Vec::new() // return empty if any step fails
}