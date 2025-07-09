use html2text::from_read;
use crate::config::CONFIG;
use reqwest::Client;
use feed_rs::parser;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct NewsEntry {
    pub title: String,
    pub summary: String,
    pub link: String,
}

#[derive(Debug)]
pub struct ManualInterventionResult {
    pub entries: Vec<NewsEntry>,
}

pub fn ignored_keywords(entry: &NewsEntry) -> bool {
    for keyword in &CONFIG.ignored_keywords {
        let keyword = keyword.to_ascii_lowercase();
        let title_lower = entry.title.to_ascii_lowercase();

        if title_lower.contains(&keyword) {
            return true;
        }

        if CONFIG.include_summary_in_query {
            let summary_lower = entry.summary.to_ascii_lowercase();
            if summary_lower.contains(&keyword) {
                return true;
            }
        }
    }
    false
}

pub async fn check_for_manual_intervention() -> ManualInterventionResult {
    // This gives us a vector of NewsEntry structs from the archlinux.org RSS feed
    let entries = get_entries();

    // Check for entries with keywords that indicate manual intervention
    let keywords = CONFIG.keywords.iter()
        .map(|kw| kw.to_ascii_lowercase())
        .collect::<Vec<String>>();
    let mut found_entries = Vec::new();

    // Biggest performance overhead is here:
    // This is where the actual network request to the feed is awaited
    let entries = entries.await;

    if !CONFIG.match_all_entries {
        for entry in &entries {
            let text = if CONFIG.include_summary_in_query {
                format!("{} {}", entry.title, entry.summary).to_ascii_lowercase()
            } else {
                entry.title.to_ascii_lowercase()
            };
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

pub async fn get_entries() -> Vec<NewsEntry> {
    // Build HTTP client with user agent
    let client = Client::builder()
        .user_agent("arch-manwarn")
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to build HTTP client");

    // Fetch RSS feed content asynchronously
    let content = match client.get(&CONFIG.rss_feed_url).send().await {
        Ok(response) => match response.text().await {
            Ok(text) => text,
            Err(err) => {
                eprintln!("Failed to read response text: {err}");
                return Vec::new();
            }
        },
        Err(err) => {
            eprintln!("Failed to fetch RSS feed: {err}");
            return Vec::new();
        }
    };

    // Parse feed content (feed-rs parsing is synchronous)
    let feed = match parser::parse(content.as_bytes()) {
        Ok(feed) => feed,
        Err(err) => {
            eprintln!("Failed to parse feed: {err}");
            return Vec::new();
        }
    };

    // Map feed entries into your NewsEntry struct
    feed.entries
        .iter()
        .map(|entry| {
            let title = entry.title.as_ref().map_or("[No title provided]", |t| t.content.as_str());
            let summary = entry.summary.as_ref().map_or("[No summary provided]", |s| s.content.as_str());
            let link = entry.links.get(0).map_or("https://archlinux.org/news/", |l| l.href.as_str());

            NewsEntry {
                title: title.to_string(),
                summary: from_read(summary.as_bytes(), 80).unwrap_or_else(|_| String::from("[could not parse summary]")),
                link: link.to_string(),
            }
        })
        .collect()
}