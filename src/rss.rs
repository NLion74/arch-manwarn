use html2text::from_read;
use crate::config::CONFIG;
use futures::future::join_all;
use reqwest::Client;
use feed_rs::parser;
use std::time::Duration;
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct NewsEntry {
    pub title: String,
    pub summary: String,
    pub link: String,
}

#[derive(Debug)]
pub struct ManualInterventionResult {
    pub entries: Vec<NewsEntry>,
    pub last_successful_request: Option<std::time::SystemTime>,
}

pub fn ignored_keywords(entry: &NewsEntry) -> bool {
    for keyword in &CONFIG.ignored_keywords {
        if CONFIG.case_sensitive {
            // Case-sensitive matching
            if entry.title.contains(keyword) {
                return true;
            }
            if CONFIG.include_summary_in_query && entry.summary.contains(keyword) {
                return true;
            }
        } else {
            // Case-insensitive matching
            let keyword_lower = keyword.to_ascii_lowercase();
            let title_lower = entry.title.to_ascii_lowercase();

            if title_lower.contains(&keyword_lower) {
                return true;
            }
            if CONFIG.include_summary_in_query {
                let summary_lower = entry.summary.to_ascii_lowercase();
                if summary_lower.contains(&keyword_lower) {
                    return true;
                }
            }
        }
    }
    false
}


pub async fn check_for_manual_intervention() -> ManualInterventionResult {
    // This gives us a vector of NewsEntry structs from the archlinux.org RSS feed
    let start_time = SystemTime::now();
    let entries = get_entries_from_feeds();

    // Check for entries with keywords that indicate manual intervention
    let keywords: Vec<String> = if CONFIG.case_sensitive {
        CONFIG.keywords.iter().cloned().collect()
    } else {
        CONFIG.keywords.iter()
            .map(|kw| kw.to_ascii_lowercase())
            .collect()
    };
    let mut found_entries = Vec::new();

    // Biggest performance overhead is here:
    // This is where the actual network request to the feed is awaited
    let entries = entries.await;

    if !CONFIG.match_all_entries {
        for entry in &entries {
            let text = if CONFIG.include_summary_in_query {
                format!("{} {}", entry.title, entry.summary)
            } else {
                entry.title.clone()
            };

            let text_to_check = if CONFIG.case_sensitive {
                text
            } else {
                text.to_ascii_lowercase()
            };

            if keywords.iter().any(|kw| text_to_check.contains(kw)) {
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

    let last_successful_request = if !entries.is_empty() {
        Some(start_time)
    } else {
        None
    };

    ManualInterventionResult {
        entries: found_entries,
        last_successful_request: last_successful_request,
    }
}

pub async fn get_entries_from_feeds() -> Vec<NewsEntry> {
    let client = Client::builder()
        .user_agent("arch-manwarn")
        .timeout(Duration::from_secs(10))
        .build()
        .expect("Failed to build HTTP client");

    // Create a vector of futures, one for each feed URL
    let fetches = CONFIG.rss_feed_urls.iter().map(|url| {
        fetch_and_parse_single_feed(&client, url)
    });

    // Await all fetches concurrently
    let results: Vec<Vec<NewsEntry>> = join_all(fetches).await;

    // Flatten all entries into one Vec
    results.into_iter().flatten().collect()
}

async fn fetch_and_parse_single_feed(client: &Client, url: &str) -> Vec<NewsEntry> {
    let content = match client.get(url).send().await {
        Ok(response) => match response.text().await {
            Ok(text) => text,
            Err(err) => {
                eprintln!("Failed to read response text from {}: {err}", url);
                return Vec::new();
            }
        },
        Err(err) => {
            eprintln!("Failed to fetch RSS feed {}: {err}", url);
            return Vec::new();
        }
    };

    let feed = match parser::parse(content.as_bytes()) {
        Ok(feed) => feed,
        Err(err) => {
            eprintln!("Failed to parse feed {}: {err}", url);
            return Vec::new();
        }
    };

    feed.entries
        .iter()
        .map(|entry| {
            let title = entry.title.as_ref().map_or("[No title provided]", |t| t.content.as_str());
            let summary = entry.summary.as_ref().map_or("[No summary provided]", |s| s.content.as_str());
            let link = entry.links.get(0).map_or("[No link provided]", |l| l.href.as_str());

            NewsEntry {
                title: title.to_string(),
                summary: from_read(summary.as_bytes(), 80).unwrap_or_else(|_| String::from("[could not parse summary]")),
                link: link.to_string(),
            }
        })
        .collect()
}
