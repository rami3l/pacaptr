mod cmd;
pub mod config;

pub use self::cmd::Opts;
pub use self::config::Config;
use crate::exec::is_exe;
use crate::package_manager::*;

/// Detect the name of the package manager to be used in auto dispatch.
pub fn detect_pm_str<'s>() -> &'s str {
    let pairs: &[(&str, &str)] = match () {
        _ if cfg!(target_os = "windows") => &[("scoop", ""), ("choco", "")],

        _ if cfg!(target_os = "macos") => &[
            ("brew", "/usr/local/bin/brew"),
            ("port", "/opt/local/bin/port"),
        ],

        _ if cfg!(target_os = "linux") => &[
            ("apk", "/sbin/apk"),
            ("apt", "/usr/bin/apt"),
            // ("apt-get", "/usr/bin/apt-get"),
            ("dnf", "/usr/bin/dnf"),
            ("zypper", "/usr/bin/zypper"),
        ],

        _ => &[],
    };

    pairs
        .iter()
        .find_map(|(name, path)| is_exe(name, path).then(|| *name))
        .unwrap_or("unknown")
}

/// Generate the `Pm` instance according it's name, feeding it with the current `Config`.
pub fn yield_pm(cfg: Config) -> Box<dyn Pm> {
    // If the `Pm` to be used is not stated in any config,
    // we should fall back to automatic detection.
    let pm = cfg.default_pm.as_deref().unwrap_or_else(detect_pm_str);

    #[allow(clippy::match_single_binding)]
    match pm {
        // Chocolatey
        "choco" => Chocolatey { cfg }.boxed(),

        // Scoop
        "scoop" => Scoop { cfg }.boxed(),

        // Homebrew/Linuxbrew
        "brew" => Homebrew { cfg }.boxed(),

        // Macports
        "port" if cfg!(target_os = "macos") => Macports { cfg }.boxed(),

        // Apk for Alpine
        "apk" => Apk { cfg }.boxed(),

        // Apt for Debian/Ubuntu/Termux (new versions)
        "apt" => Apt { cfg }.boxed(),

        // Dnf for RedHat
        "dnf" => Dnf { cfg }.boxed(),

        // Zypper for SUSE
        "zypper" => Zypper { cfg }.boxed(),

        // * External Package Managers *

        // Conda
        "conda" => Conda { cfg }.boxed(),

        // Pip
        "pip" => Pip {
            cmd: "pip".into(),
            cfg,
        }
        .boxed(),

        "pip3" => Pip {
            cmd: "pip3".into(),
            cfg,
        }
        .boxed(),

        // Tlmgr
        "tlmgr" => Tlmgr { cfg }.boxed(),

        // Unknown package manager X
        x => Unknown::new(x).boxed(),
    }
}
