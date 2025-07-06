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
    get("https://archlinux.org/feeds/news/")
        .expect("Failed to fetch RSS feed")
        .read_to_string(&mut content)
        .expect("Failed to read feed content");

    let feed = parser::parse(content.as_bytes()).expect("Failed to parse feed");

    let mut entries = Vec::new();
    for entry in feed.entries {
        let title = match &entry.title {
            Some(text) => &text.content,
            None => "[No title provided]",
        };
        let summary = match &entry.summary {
            Some(text) => &text.content,
            None => "[No summary provided]",
        };
        let link = match entry.links.get(0) {
            Some(l) => &l.href,
            None => "https://archlinux.org/news/",
        };

        entries.push(NewsEntry {
            title: title.to_string(),
            summary: summary.to_string(),
            link: link.to_string(),
        });
    }

    entries
}