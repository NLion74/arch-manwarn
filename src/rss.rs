use crate::config::CONFIG;
use nanohtml2text::html2text;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::io::BufReader;
use std::time::SystemTime;

#[derive(Debug)]
#[cfg_attr(test, derive(serde::Deserialize, PartialEq))]
pub struct NewsEntry {
    pub title: String,
    pub summary: String,
    pub link: String,
}

#[derive(Debug)]
pub struct ManualInterventionResult {
    pub entries: Vec<NewsEntry>,
    pub last_successful_request: Option<SystemTime>,
}

pub fn check_for_manual_intervention() -> ManualInterventionResult {
    let start_time = SystemTime::now();

    // Biggest performance overhead is here:
    // This is where the actual network request to the feed is awaited
    let entries: Vec<NewsEntry> = CONFIG
        .rss_feed_urls
        .par_iter() // multithreading here
        .map(|url| fetch_and_parse_single_feed(url))
        .flatten()
        .collect();

    let last_successful_request = (!entries.is_empty()).then_some(start_time);

    let found_entries = match_entries::matches(entries);

    ManualInterventionResult {
        entries: found_entries,
        last_successful_request,
    }
}

fn fetch_and_parse_single_feed(url: &str) -> Vec<NewsEntry> {
    let content = match minreq::get(url)
        .with_timeout(CONFIG.request_timeout)
        .with_header("User-Agent", "arch-manwarn")
        .send_lazy()
    {
        Ok(resp) => resp,
        Err(err) => {
            eprintln!("Failed to fetch RSS feed {url}: {err}");
            return Vec::new();
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

pub mod match_entries {
    #[cfg(not(test))]
    use crate::config::CONFIG;
    use crate::rss::NewsEntry;
    #[cfg(test)]
    use crate::test::CONFIG;

    fn match_kw(kws: &[String], strs: &str) -> bool {
        let strs = if CONFIG.case_sensitive {
            strs.to_string()
        } else {
            strs.to_ascii_lowercase()
        };

        kws.iter().any(|kw| strs.contains(kw))
    }

    fn match_kw_all(kws: &[String], entry: &NewsEntry) -> bool {
        let kws = if CONFIG.case_sensitive {
            kws.to_vec()
        } else {
            kws.iter().map(|kw| kw.to_ascii_lowercase()).collect()
        };

        match_kw(&kws, &entry.title)
            || (CONFIG.include_summary_in_query && match_kw(&kws, &entry.summary))
    }

    pub fn matches(entries: Vec<NewsEntry>) -> Vec<NewsEntry> {
        entries
            .into_iter()
            // remove exclude first
            .filter(|entry| !match_kw_all(&CONFIG.ignored_keywords, entry))
            // keep all or match include
            .filter(|entry| CONFIG.match_all_entries || match_kw_all(&CONFIG.keywords, entry))
            .collect()
    }
}
