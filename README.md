# arch-manwarn

Tired of having to check Arch news for manual interventions before every upgrade? This tool is for you.
**arch-manwarn** is a minimalist utility that checks the Arch news RSS feed for manual intervention warnings and blocks your `pacman` upgrade or install if you need to take action.

## What does it do?

When `pacman` installs or upgrades packages, this tool runs as a **pacman hook**. If it detects any recent news classified as requiring manual intervention, it blocks the pacman transaction and mark the news as read.

On the first run, arch-manwarn assumes you have seen and applied all manual interventions up to that point.

It classifies Arch news as requiring manual intervention with the keywords:

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

I created this project so that I would not miss any manual interventions in the Arch news, but I also did not want to read any Arch news that does not affect me directly. Therefore, this project focuses on minimalism rather than notifying about all news.

If you want a tool that interrupts for every new Arch news, you may refer to [this project](https://github.com/bradford-smith94/informant), which I found shortly after creating this one. It works similarly but interrupts for every new Arch news.

# Installation

## AUR (Recommended)

Since this package is exclusive to Arch and the pacman package manager, this package is only available to be installed from my [AUR Package](https://aur.archlinux.org/packages/arch-manwarn).

## Development

Due to permission issues when running the program manually instead of via the arch-manwarn.hook, you may need to change the cache path from /var/cache. You can do this like so:

```
ARCH_NEWS_CACHE_PATH=./arch-manwarn-dev.json cargo run
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
