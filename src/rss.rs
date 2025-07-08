use feed_rs::parser;
use reqwest::blocking::get;
use std::io::Read;
use html2text::from_read;
use crate::config::CONFIG;

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

pub fn ignored_keywords(entry: &NewsEntry) -> bool {
    for keyword in &CONFIG.ignored_keywords {
        let keyword = keyword.to_lowercase();
        let title_lower = entry.title.to_lowercase();

        if title_lower.contains(&keyword) {
            return true;
        }

        if CONFIG.include_summary_in_query {
            let summary_lower = entry.summary.to_lowercase();
            if summary_lower.contains(&keyword) {
                return true;
            }
        }
    }
    false
}

pub fn check_for_manual_intervention() -> ManualInterventionResult {
    // This gives us a vector of NewsEntry structs from the archlinux.org RSS feed
    let entries: Vec<NewsEntry> = get_entries();

    // Check for entries with keywords that indicate manual intervention
    let keywords = CONFIG.keywords.iter()
        .map(|kw| kw.to_lowercase())
        .collect::<Vec<String>>();
    let mut found_entries = Vec::new();

    if !CONFIG.match_all_entries {
        for entry in &entries {
            let mut text = format!("{}", entry.title.to_lowercase());
            if CONFIG.include_summary_in_query {
                text.push_str(&format!(" {}", entry.summary.to_lowercase()));
            }
            if keywords.iter().any(|kw| text.contains(kw)) {
                if !ignored_keywords(entry) {
                    found_entries.push(entry.clone());
                }
            }
        }
    } else {
        for entry in &entries {
            if !ignored_keywords(entry) {
                    found_entries.push(entry.clone());
            }
        }
    }

    ManualInterventionResult {
        entries: found_entries,
    }
}

pub fn get_entries() -> Vec<NewsEntry> {
    let mut content = String::new();
    if let Ok(mut response) = get(&CONFIG.rss_feed_url) {
        if response.read_to_string(&mut content).is_ok() {
            if let Ok(feed) = parser::parse(content.as_bytes()) {
                let mut entries = Vec::new();
                for entry in feed.entries {
                    let title = entry.title.as_ref().map_or("[No title provided]", |t| &t.content);
                    let summary = entry.summary.as_ref().map_or("[No summary provided]", |s| &s.content);
                    let link = entry.links.get(0).map_or("https://archlinux.org/news/", |l| &l.href);

                    entries.push(NewsEntry {
                        title: title.to_string(),
                        summary: match from_read(summary.as_bytes(), 80) {
                            Ok(text) => text,
                            Err(_) => String::from("[could not parse summary]"),
                        },
                        link: link.to_string(),
                    });
                }
                return entries;
            }
        }
    }

    Vec::new() // return empty if any step fails
}