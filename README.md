# arch-manwarn

[![AUR version](https://img.shields.io/aur/version/arch-manwarn)](https://aur.archlinux.org/packages/arch-manwarn)
[![GitHub stars](https://img.shields.io/github/stars/NLion74/arch-manwarn?style=social)](https://github.com/NLion74/arch-manwarn/stargazers)
[![License: Unlicense](https://img.shields.io/badge/license-Unlicense-blue.svg)](https://unlicense.org/)
[![GitHub release](https://img.shields.io/github/v/release/NLion74/arch-manwarn)](https://github.com/NLion74/arch-manwarn/releases)

**arch-manwarn** is a small, fast and [configurable](#configuration) Rust-Based pacman hook that detects [Arch Linux News](https://archlinux.org/news) requiring manual intervention, and blocks upgrades **only when relevant** to your system.

It’s designed to stay quiet unless action is actually needed, helping you avoid breakage without adding unnecessary noise.

## What does it do?

Whenever `pacman` performs an upgrade or install, `arch-manwarn` runs as a **pacman hook**.

It checks the Arch Linux News feed for recent posts that match your [configured](#configuration) keywords (e.g. `"manual intervention"`) and blocks the transaction if a match is found — helping you avoid breaking changes.

Once installed, test it with:

```
arch-manwarn
```

This should return a short confirmation message.

## Modes of Operation

`arch-manwarn` supports four modes:

-   `arch-manwarn` - Prints quick confirmation message (used for sanity checks).
-   `arch-manwarn check` - Used internally by the pacman hook to detect new warnings.
-   `arch-manwarn status` - Displays a list of cached matching warnings with timestamps.
-   `arch-manwarn read` - Manually mark all unread warnings as read (usually not needed unless configuration is adjusted).

    On first run, it assumes you have already read all previous warnings.

arch-manwarn flags news containing "manual intervention" (case-insensitive) as requiring action.
You can customize keywords in `/etc/arch-manwarn/config.toml`

```
keywords = ["manual intervention", "breaking change"]
```

The **pacman hook** only activates on upgrades or installs therefore if for any reason `arch-manwarn` causes issues with your system or pacman transactions you can always remove it:

```
sudo pacman -Rns arch-manwarn
```

## Why arch-manwarn?

I created this tool to avoid missing important manual interventions in the Arch news.

But unlike tools that intrusively interrupt you for every news post, **arch-manwarn** blocks transactions **only** for the ones matching your defined criteria.

It's lean, fast, and written in Rust — prioritizing **simplicity, efficiency, and precision**.

Instead of intrusively blocking every `pacman` transaction for every news, **arch-manwarn** filters _only_ those which require manual intervention.

If you want to be notified of **every** Arch news post, you can either configure **arch-manwarn** to match all entries or check out [informant](https://github.com/bradford-smith94/informant), an alternative designed for that behavior.

## Installation

### AUR (Recommended)

Install from the [AUR Package](https://aur.archlinux.org/packages/arch-manwarn):

```
paru -S arch-manwarn
```

or

```
yay -S arch-manwarn
```

### Configuration

`arch-manwarn` uses a TOML configuration file. The default path is
`/etc/arch-manwarn/config.toml`

You can override this path using the `ARCH_MANWARN_CONFIG` environment variable:

```
export ARCH_MANWARN_CONFIG=/path/to/your/config.toml
```

Example `config.toml` with default options

```
# Keywords that trigger warnings (case-insensitive)
keywords = ["manual intervention",]

# If true, match all news posts regardless of keywords
match_all_entries = false

# Ignore news entries containing any of these keywords
ignored_keywords = []

# Whether keyword query is case sensitive (Applies to both ignore and regular keywords)
case_sensitive = false

# Include the summary in keyword matching
include_summary_in_query=true

# Whether to add installed package names to keyword matching
# Recommended to disable include_summary_in_query with this option
include_installed_packages = false

# Both of these conditions must be met to prune a cached news entry:
# 1. It has not been seen in the RSS feed for `prune_missing_days`, AND
# 2. It is older than `prune_age_days`.
# This avoids removing entries that may temporarily disappear from the feed.
prune_missing_days = 30
prune_age_days = 60


# RSS feed URLs to check
# Adding feeds with high latency can massively impact performance
rss_feed_urls = [
    "https://archlinux.org/feeds/news/",
]

# Timeout (in seconds) for any HTTP requests to RSS feeds
request_timeout = 10

# Display summaries for matching news posts
show_summary = false

# Automatically mark entries as read after showing them
mark_as_read_automatically = true

# Warn only (don’t block pacman) - essentially dry-run
warn_only = false

# Where to store the cache
cache_path = "/var/cache/arch-manwarn.json"
```

## Development

A mirror of the AUR PKGBUILD is available [here](https://github.com/NLion74/arch-manwarn-aur).

Due to permission issues when running the program manually instead of via the arch-manwarn.hook, you may need to change the cache and config path:

```
ARCH_NEWS_CACHE_PATH=./arch-manwarn-cache.json \
ARCH_MANWARN_CONFIG=./arch-manwarn-config.toml \
cargo run
```

Local install for testing:

```
cargo build --release
sudo install -Dm755 target/release/arch-manwarn /usr/bin/arch-manwarn
sudo install -Dm644 hooks/arch-manwarn.hook /usr/share/libalpm/hooks/arch-manwarn.hook
```

## Contribute & Support

Contributions, bug reports, and feature requests are welcome!

Star the repo ⭐ if you find it useful!
