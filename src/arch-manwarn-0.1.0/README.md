# arch-manwarn

Don't feel like always keeping up with arch news for required manual intervention before every single system upgrade? This package may be the solution for you.
A minimalist utility that checks Arch news RSS for manual intervention warnings and warns you before system upgrades.

## What does it do?

When pacman install or upgrade is run this program will run as a pacman hook. If it parses any news that classified as manual intervention it will exit and mark the news as read. Originally it was planned to be interactive but pacman hooks are inherently not designed for this behaviour.

On first run arch-manwarn assumes you have seen and applied all manual interventions to this point

# Why this?

I created this project so that I would not miss any manual interventions in the arch news but I also did not want to read any arch news that do not affect me directly. Therefore I created this project with a focus on minimalism. If you find yourself asking why not just interrupt for all new news and wish to have this functionality you may refer to [this project](https://github.com/bradford-smith94/informant) which I found after creating this very one. It works very similiar but interrupts for every new news.

## Installation

# Development

Due to permission issues when not using the `archNewsHook.hook`, but running the program manually instead, you may need to change the cache path from `/var/cache`

This can be done like following:

```
ARCH_NEWS_CACHE_PATH=./arch-manwarn-dev.json cargo run
```

1. Build:
   cargo build --release

2. Copy binary to `/usr/bin/arch-manwarn`

3. Copy `hooks/arch-news-check.hook` to `/etc/pacman.d/hooks/`
