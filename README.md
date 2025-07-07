# arch-manwarn

Tired of having to check Arch news for manual interventions before every upgrade? This tool is for you.

**arch-manwarn** is a minimalist utility written in Rust that checks the Arch news RSS feed for manual intervention warnings and blocks your `pacman` upgrade or install if action is needed.

Its small, efficient codebase emphasizes performance and simplicity, staying out of the way unless your attention is truly required.

## What does it do?

When `pacman` installs or upgrades packages, this tool runs as a **pacman hook**. If it detects any recent news classified as requiring manual intervention, it blocks the pacman transaction and mark the news as read.

Once installed, you can verify that the tool is working by running:

```
arch-manwarn
```

This should output a short confirmation message.

There are 4 modes of operation:

-   `arch-manwarn` - shows a short message to confirm installation.
-   `arch-manwarn check` - used internally by the pacman hook to check for new manual interventions.
-   `arch-manwarn status` - Shows a summary of cached manual interventions, including how long ago they were first and last seen.
-   `arch-manwarn read` - Manually marks all unread items as read (usually not needed unless configuration is adjusted).

    On the first run, arch-manwarn assumes you have already read and handled all previous manual interventions.

By default it classifies Arch news as requiring manual intervention with the keywords:

-   `manual intervention`
-   `action required`
-   `attention`
-   `intervention`

Originally, this was planned to be interactive, but pacman hooks are inherently not designed for this behavior.

The **pacman hook** only activates on upgrades or installs therefore if for any reason `arch-manwarn` causes issues with your system or pacman transactions you can always remove it:

```
sudo pacman -Rns arch-manwarn
```

# Why this?

I created this project to avoid missing important manual interventions in the Arch news - without having to read through every news that does not affect me directly.
Instead of intrusively blocking every `pacman` transaction for every news, **arch-manwarn** filters _only_ those which require manual intervention.

It’s written in Rust with a small, efficient codebase that prioritizes **minimalism, performance, and staying out of the way.**

If you’re thinking, _"Why not just alert me for every news post?"_ - you may prefer [this project](https://github.com/bradford-smith94/informant), which I found shortly after creating this one. It works similarly but interrupts for every new Arch news.

# Installation

## AUR (Recommended)

Since this package is exclusive to Arch and the pacman package manager, this package is only available to be installed from my [AUR Package](https://aur.archlinux.org/packages/arch-manwarn).

## Configuration

`arch-manwarn` is configured using a TOML file that is initiated with default options. It allows users to customize behavior such as cache location, keyword matching, pruning policy, and warning behavior.

By default, the config file is located at: `/etc/arch-manwarn/config.toml`

You can override this path using the `ARCH_MANWARN_CONFIG` environment variable:

```
export ARCH_MANWARN_CONFIG=/path/to/your/config.toml
```

Example `config.toml` with default options

```
# List of keywords to match in news entry titles
keywords = ["manual intervention", "action required", "attention", "intervention"]

# If true, show *all* news entries regardless of keywords
match_all_entries = false

# Ignore news entries containing any of these keywords in either title or summary
ignored_keywords = []

# Both of these conditions must be met to prune a cached news entry:
# 1. It has not been seen in the RSS feed for `prune_missing_days`, AND
# 2. It is older than `prune_age_days`.
# This avoids removing entries that may temporarily disappear from the feed.
prune_missing_days = 30
prune_age_days = 60


# URL of the RSS feed (defaults to Arch Linux's official news)
rss_feed_url = "https://archlinux.org/feeds/news/"

# If true, also display the summary for each news entry
show_summary = false

# If true, automatically mark entries as read after displaying them
mark_as_read_automatically = true

# If true, **warn only** without blocking upgrades (exit code 0)
warn_only = false

# Where to store the entries cache file
cache_path = "/var/cache/arch-manwarn.json"
```

# Development

A mirror of the AUR PKGBUILD can be found [here](https://github.com/NLion74/arch-manwarn-aur)

Due to permission issues when running the program manually instead of via the arch-manwarn.hook, you may need to change the cache path from /var/cache. You can do this like so:

```

ARCH_NEWS_CACHE_PATH=./arch-manwarn-cache.json ARCH_MANWARN_CONFIG=./arch-manwarn-config.toml cargo run

```

1. Build the release binary:
   `cargo build --release`

2. Copy binary to `/usr/bin/arch-manwarn`

    ```
    sudo install -Dm755 target/release/arch-manwarn /usr/bin/arch-manwarn
    ```

3. Copy `hooks/arch-news-check.hook` to `/etc/pacman.d/hooks/`
    ```
    sudo install -Dm644 hooks/arch-manwarn.hook /usr/share/libalpm/hooks/arch-manwarn.hook
    ```

```

```
