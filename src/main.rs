mod rss;
mod cache;
mod config;
use crate::config::CONFIG;

fn main() {
    let mut args = std::env::args();
    let _exe = args.next();

    println!("{:#?}", *CONFIG);

    match args.next().as_deref() {
        None => {
            println!("arch-manwarn is installed and should block any pacman transactions before manual intervention is required!\n\
                      Usage:\n\
                      arch-manwarn check   # Check for manual intervention (default pacman hook behavior may not work with user privileges)\n");
        }

        Some("check") => {
            let new_entries = cache::check_new_entries();
            if !new_entries.is_empty() {
                eprintln!("\nManual intervention required for the following Arch news entries:\n");
                for entry in &new_entries {
                    eprintln!("- {}", entry.title);
                    eprintln!("  For more details see: {}", entry.link);
                }
                eprintln!("\nAll other news can be found on https://archlinux.org/news/.");
                eprintln!("Arch ManWarn: Exiting to block the upgrade process.\n");

                std::process::exit(1);
            }
        }

        Some(cmd) => {
            eprintln!("Unknown option: {cmd}");
            eprintln!("Usage:\n  arch-manwarn check");
            std::process::exit(2);
        }
    }
}
