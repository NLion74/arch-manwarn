use crate::config::CONFIG;
use nanohtml2text::html2text;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::io::BufReader;
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

pub fn check_for_manual_intervention() -> ManualInterventionResult {
    let start_time = SystemTime::now();

    // Check for entries with keywords that indicate manual intervention
    let keywords: Vec<String> = if CONFIG.case_sensitive {
        CONFIG.keywords.to_vec()
    } else {
        CONFIG
            .keywords
            .iter()
            .map(|kw| kw.to_ascii_lowercase())
            .collect()
    };
    let mut found_entries = Vec::new();

    // Biggest performance overhead is here:
    // This is where the actual network request to the feed is awaited
    // Include tokio runtime initializing
    let entries = get_entries_from_feeds();

    let last_successful_request = if !entries.is_empty() {
        Some(start_time)
    } else {
        None
    };

    if !CONFIG.match_all_entries {
        for entry in entries {
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

            if keywords.iter().any(|kw| text_to_check.contains(kw)) && !ignored_keywords(&entry) {
                found_entries.push(entry);
            }
        }
    } else {
        for entry in entries {
            if !ignored_keywords(&entry) {
                found_entries.push(entry);
            }
        }
    }

    ManualInterventionResult {
        entries: found_entries,
        last_successful_request,
    }
}

pub fn get_entries_from_feeds() -> Vec<NewsEntry> {
    // Create a vector of futures, one for each feed URL
    let fetches = CONFIG
        .rss_feed_urls
        .par_iter() // multithreading here
        .map(|url| fetch_and_parse_single_feed(url));

    // Await all fetches concurrently
    // Flatten all entries into one Vec
    fetches.flatten().collect()
}

fn fetch_and_parse_single_feed(url: &str) -> Vec<NewsEntry> {
    let mut current_url = url.to_string();

    let content = {
        let mut redirects = 0;
        loop {
            let response = match minreq::get(&current_url)
                .with_follow_redirects(false)
                .with_timeout(10)
                .with_header("User-Agent", "arch-manwarn")
                .send_lazy()
            {
                Ok(resp) => resp,
                Err(err) => {
                    eprintln!("Failed to fetch RSS feed {current_url}: {err}");
                    return Vec::new();
                }
            };

            if http_status_code::is_redirection(response.status_code) {
                if redirects >= CONFIG.max_redirects {
                    eprintln!("Too many redirects for {current_url}");
                    return Vec::new();
                }
                if let Some(location) = response.headers.get("Location") {
                    current_url = location.to_string();
                    redirects += 1;
                    continue;
                } else {
                    eprintln!("Redirect without Location header for {current_url}");
                    return Vec::new();
                }
            } else if http_status_code::is_success(response.status_code) {
                break response;
            } else {
                eprintln!(
                    "Failed to fetch RSS feed {current_url}: HTTP status {}",
                    response.status_code
                );
                return Vec::new();
            }
        }
    };

    let channel = match rss::Channel::read_from(BufReader::new(content)) {
        Ok(ch) => ch,
        Err(err) => {
            eprintln!("Failed to read/parse feed {url}: {err}");
            return Vec::new();
        }
    };

    channel
        .items
        .into_iter()
        .map(|entry| {
            let title = entry
                .title
                .unwrap_or_else(|| "[No title provided]".to_string());
            let summary = match (entry.content, entry.description) {
                (None, None) => "[No summary provided]".to_string(),
                (Some(c), Some(d)) if c.len() > d.len() => c,
                (_, Some(s)) | (Some(s), None) => s,
            };
            let link = entry
                .link
                .unwrap_or_else(|| "[No link provided]".to_string());

            NewsEntry {
                title,
                summary: html2text(&summary),
                link,
            }
        })
        .collect()
}

#[allow(dead_code)]
mod http_status_code {
    // codes here are borrowed from https://github.com/hyperium/http/blob/master/src/status.rs

    type StatusCode = i32;

    /// Check if status is within 100-199.
    #[inline]
    pub fn is_informational(code: StatusCode) -> bool {
        (100..200).contains(&code)
    }

    /// Check if status is within 200-299.
    #[inline]
    pub fn is_success(code: StatusCode) -> bool {
        (200..300).contains(&code)
    }

    /// Check if status is within 300-399.
    #[inline]
    pub fn is_redirection(code: StatusCode) -> bool {
        (300..400).contains(&code)
    }

    /// Check if status is within 400-499.
    #[inline]
    pub fn is_client_error(code: StatusCode) -> bool {
        (400..500).contains(&code)
    }

    /// Check if status is within 500-599.
    #[inline]
    pub fn is_server_error(code: StatusCode) -> bool {
        (500..600).contains(&code)
    }
}
