use super::*;
use crate::config::Config;
use crate::rss::NewsEntry;
use crate::rss::match_entries::matches;

#[test]
fn case_insensitive() {
    let _permit = init_config(Config {
        keywords: vec!["manual intervention".to_string()],
        match_all_entries: false,
        ignored_keywords: vec![],
        case_sensitive: false,
        include_summary_in_query: false,
        ..Default::default()
    });

    assert_eq!(select_entries([true; 4]), matches(entries()));
}

#[test]
fn case_sensitive() {
    let _permit = init_config(Config {
        keywords: vec!["manual intervention".to_string()],
        match_all_entries: false,
        ignored_keywords: vec![],
        case_sensitive: true,
        include_summary_in_query: false,
        ..Default::default()
    });

    assert_eq!(
        select_entries([true, true, false, true]),
        matches(entries())
    );
}

fn select_entries(bools: [bool; 4]) -> Vec<NewsEntry> {
    entries()
        .into_iter()
        .zip(bools.into_iter())
        .filter_map(|(a, b)| b.then_some(a))
        .collect::<Vec<NewsEntry>>()
}

fn entries() -> Vec<NewsEntry> {
    let raw = serde_json::json!( [
      {
        "title": "linux-firmware >= 20250613.12fe085f-5 upgrade requires manual intervention",
        "summary": "With `20250613.12fe085f-5`, we split our firmware into several vendor-focused\npackages. `linux-firmware` is now an empty package depending on our default set\nof firmware.\n\nUnfortunately, this coincided with upstream reorganizing the symlink layout of\nthe NVIDIA firmware, resulting in a situation that Pacman cannot handle. When\nattempting to upgrade from `20250508.788aadc8-2` or earlier, you will see the\nfollowing errors:\n\n`linux-firmware-nvidia: /usr/lib/firmware/nvidia/ad103 exists in filesystem\nlinux-firmware-nvidia: /usr/lib/firmware/nvidia/ad104 exists in filesystem\nlinux-firmware-nvidia: /usr/lib/firmware/nvidia/ad106 exists in filesystem\nlinux-firmware-nvidia: /usr/lib/firmware/nvidia/ad107 exists in filesystem\n`\n\nTo progress with the system upgrade, first remove `linux-firmware`, then\nreinstall it as part of the upgrade:\n\n`# pacman -Rdd linux-firmware\n# pacman -Syu linux-firmware\n`\n",
        "link": ""
      },
      {
        "title": "Plasma 6.4.0 will need manual intervention if you are on X11",
        "summary": "On Plasma 6.4 the wayland session will be the only one installed when the users\ndoes not manually specify kwin-x11.\n\nWith the recent split of kwin into kwin-wayland and kwin-x11, users running the\nold X11 session needs to manually install plasma-x11-session, or they will not\nbe able to login. Currently pacman is not able to figure out your personal\nsetup, and it wouldn't be ok to install plasma-x11-session and kwin-x11 for\nevery one using Plasma.\n\n### tldr: Install plasma-x11-session if you are still using x11\n",
        "link": ""
      },
      {
        "title": "Manual intervention for pacman 7.0.0 and local repositories required",
        "summary": "With the release of [version 7.0.0][1] pacman has added support for downloading\npackages as a separate user with dropped privileges.\n\nFor users with local repos however this might imply that the download user does\nnot have access to the files in question, which can be fixed by assigning the\nfiles and folder to the `alpm` group and ensuring the executable bit (`+x`) is\nset on the folders in question.\n\n`$ chown :alpm -R /path/to/local/repo\n`\n\nRemember to [merge the .pacnew][2] files to apply the new default.\n\nPacman also introduced [a change][3] to improve checksum stability for git repos\nthat utilize `.gitattributes` files. This might require a one-time checksum\nchange for `PKGBUILD`s that use git sources.\n\n[1]: https://gitlab.archlinux.org/pacman/pacman/-/blob/master/NEWS?ref_type=head\ns\n[2]: https://wiki.archlinux.org/title/Pacman/Pacnew_and_Pacsave\n[3]: https://gitlab.archlinux.org/pacman/pacman/-/commit/9548d6cc765b1a8dcf933e8\nb1b89d0bcc3e50209\n",
        "link": ""
      },
      {
        "title": "zabbix >= 7.4.1-2 may require manual intervention",
        "summary": "Starting with `7.4.1-2`, the following Zabbix system user accounts (previously\nshipped by their related packages) will no longer be used. Instead, all Zabbix\ncomponents will now rely on a shared `zabbix` user account (as originally\n[intended by upstream][1] and done by other distributions):\n* zabbix-server\n* zabbix-proxy\n* zabbix-agent *(also used by the `zabbix-agent2` package)*\n* zabbix-web-service\n\nThis shared `zabbix` user account is provided by the newly introduced\n`zabbix-common` *split* package, which is now a dependency for all relevant\n`zabbix-*` packages.\n\nThe switch to the new user account is handled automatically for the\ncorresponding main configuration files and `systemd` service units.\n\nHowever, **manual intervention may be required** if you created custom files or\nconfigurations referencing to and / or being owned by the above deprecated users\naccounts, for example:\n* `PSK` files used for encrypted communication\n* Custom scripts for metrics collections or report generations\n* `sudoers` rules for metrics requiring elevated privileges to be collected\n* ...\n\nThose should therefore be updated to refer to and / or be owned by the new\n`zabbix` user account, otherwise some services or user parameters may fail to\nwork properly, or not at all.\n\nOnce migrated, you may [remove the obsolete user accounts from your system][2].\n\n[1]: https://www.zabbix.com/documentation/current/en/manual/installation/install\n#create-user-account\n[2]: https://wiki.archlinux.org/title/Users_and_groups#Other_examples_of_user_ma\nnagement\n",
        "link": ""
      }
    ]);

    serde_json::from_value(raw).unwrap()
}
