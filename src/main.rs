mod rss;
mod cache;
mod config;
use crate::config::CONFIG;

fn main() {
    let mut args = std::env::args();
    let _exe = args.next();

    match args.next().as_deref() {
        None => {
            println!("arch-manwarn is installed and should block any pacman transactions before manual intervention is required!\n\
                      Usage:\n\
                      arch-manwarn check   # Check for manual intervention (default pacman hook behavior may not work with user privileges)\n");
        }

        Some("check") => {
            let new_entries = cache::check_new_entries(false);
            if !new_entries.is_empty() {
                eprintln!("\nManual intervention required for the following Arch news entries:\n");
                if !CONFIG.show_summary {
                    for entry in &new_entries {
                    eprintln!("- {}", entry.title);
                    eprintln!("  For more details see: {}", entry.link);
                    }
                } else {
                    for entry in &new_entries {
                        eprintln!("- {}", entry.title);
                        eprintln!("\nSummary: \n{}", entry.summary)
                    }
                }
                eprintln!("\nAll other news can be found on https://archlinux.org/news/.");

                if CONFIG.warn_only {
                    eprintln!("Arch ManWarn: Warning only mode is enabled — not blocking upgrade.\n");
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
                println!(
                    "Marked {} entries as manually read.",
                    new_entries.len()
                );
            }
        }

        Some("status") => {
            let cache_path = cache::get_cache_path();
            let Ok(data) = std::fs::read_to_string(&cache_path) else {
                println!("📭 No cache found. Run `arch-manwarn check` first.");
                return;
            };

            let cache_file: cache::CacheFile = cache::load_cache(&cache_path);

            if cache_file.entries.is_empty() {
                println!("No cached manual interventions found.");
                return;
            }

            fn days_ago(unix_timestamp: i64) -> i64 {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_secs() as i64;

                let diff_seconds = now - unix_timestamp;
                diff_seconds / (24 * 3600)
            }

            println!("Cached Manual Intervention Entries:\n");

            for entry in &cache_file.entries {
                let days_since_first_seen = days_ago(entry.first_seen);
                let days_since_last_seen = days_ago(entry.last_seen);

                println!(
                    "- {} (first seen {} day(s) ago, last seen {} day(s) ago)",
                    entry.title,
                    days_since_first_seen,
                    days_since_last_seen
                );
            }
        }

        Some(cmd) => {
            eprintln!("Unknown option: {cmd}");
            eprintln!("Usage:\n  arch-manwarn check");
            std::process::exit(2);
        }
    }
}
