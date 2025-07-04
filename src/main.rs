mod rss;
mod cache;

fn main() {
    // Check for new entries which may require manual intervention and warn the user
    let new_entries = cache::check_new_entries();
    if !new_entries.is_empty() {
        eprintln!("\nManual intervention required for the following Arch news entries:\n");
        for entry in &new_entries {
            eprintln!("- {}", entry.title);
            eprintln!("  For more details see: {}", entry.link);
        }
        eprintln!("\nAll other news can be found on https://archlinux.org/news/.");
        eprintln!("Arch ManWarn: Exiting to block the upgrade process.\n");

        cache::mark_entries_as_seen(&new_entries);

        // Exit the upgrade
        std::process::exit(1);
    }
}
