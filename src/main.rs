mod cache;
mod config;
mod rss;
use crate::config::CONFIG;

fn main() {
    let mut args = std::env::args();
    let _exe = args.next();

    match args.next().as_deref() {
        None => {
            println!(
                "arch-manwarn is installed as a pacman hook to check for relevant entries in the Arch Linux news feed.\n\
                 There are 4 modes of operation:\n\n\
                 arch-manwarn            - Shows this short message to confirm installation.\n\
                 arch-manwarn check      - Used internally by the pacman hook to check for new matching entries.\n\
                 arch-manwarn status     - Shows a summary of cached matching entries, including how long ago they were first and last seen.\n\
                 arch-manwarn read       - Manually marks all unread items as read (usually not needed unless configuration is adjusted).\n"
            );
        }

        Some("check") => {
            let new_entries = cache::check_new_entries(false);
            if !new_entries.is_empty() {
                eprintln!("\nMatched the following Arch news entries:\n");
                if !CONFIG.show_summary {
                    for entry in &new_entries {
                        eprintln!("- {}", entry.title);
                        eprintln!("  For more details see: {}", entry.link);
                        eprintln!("---")
                    }
                } else {
                    for entry in &new_entries {
                        eprintln!("- {}", entry.title);
                        eprintln!("\nSummary: \n{}", entry.summary);
                        eprintln!("---")
                    }
                }
                eprintln!("\nAll other news can be found on https://archlinux.org/news/.");

                if CONFIG.warn_only {
                    eprintln!(
                        "Arch ManWarn: Warning only mode is enabled — not blocking upgrade.\n"
                    );
                } else {
                    eprintln!("Arch ManWarn: Exiting to block the upgrade process.\n");
                    std::process::exit(1);
                }
            }
        }

        Some("read") => {
            let new_entries = cache::check_new_entries(true);
            if new_entries.is_empty() {
                println!("No unseen entries — nothing to mark as read.");
            } else {
                println!("Marked {} entries as manually read.", new_entries.len());
            }
        }

        Some("status") => {
            let cache_path = cache::get_cache_path();
            let Ok(_data) = std::fs::read_to_string(&cache_path) else {
                println!("No cache found. Run `arch-manwarn check` first.");
                return;
            };

            let cache_file: cache::CacheFile = cache::load_cache(&cache_path);

            if cache_file.entries.is_empty() {
                println!("No cached matching entries found.");
                return;
            }

            fn days_ago_float(unix_timestamp: i64) -> f64 {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs() as i64;

                let diff_seconds = now - unix_timestamp;
                diff_seconds as f64 / 86400.0
            }

            println!("Cached Matching Entries:\n");

            // Sort entries by last seen date, most recent at the bottom
            let mut sorted_entries = cache_file.entries.clone();
            sorted_entries.sort_by(|a, b| a.last_seen.cmp(&b.last_seen));

            for entry in &sorted_entries {
                let days_since_first_seen = days_ago_float(entry.first_seen);
                let days_since_last_seen = days_ago_float(entry.last_seen);

                println!(
                    "- {} (first seen {:.1} day(s) ago, last seen {:.1} day(s) ago)",
                    entry.title, days_since_first_seen, days_since_last_seen
                );
            }

            if let Some(ts) = cache_file.last_successful_request {
                let days = days_ago_float(
                    ts.duration_since(std::time::UNIX_EPOCH).unwrap().as_secs() as i64,
                );
                println!(
                    "\nLast successful feed request: {:.1} day{} ago.",
                    days,
                    if days == 1.0 { "" } else { "s" }
                );
            } else {
                println!("\nLast successful feed request: never.");
            }
        }

        Some(cmd) => {
            eprintln!("Error: Unknown option '{cmd}'");
            eprintln!(
                "Usage:
                arch-manwarn            - Shows a short confirmation message.
                arch-manwarn check      - Checks for new matching Arch news entries.
                arch-manwarn status     - Displays summary of cached entries.
                arch-manwarn read       - Marks all entries as read."
            );
            std::process::exit(2);
        }
    }
}
