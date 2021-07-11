//! APIs for command line argument parsing and environment detection.
//!
//! An overall introduction of how this module works:
//! 1. [`clap`] handles command line arguments and generate a [`Pacaptr`]
//!   instance holding all the flags and options.
//! 2. [`Config`] reads the configuration file (if it exists) and then merges it
//!   with the current command line arguments using [`Pacaptr::merge_cfg`].
//! 3. The correct package manager to be used will be indicated by the user
//!   (through command line arguments or config file), or, if this is not the
//!   case, automatically detected by [`detect_pm_str`].
//! 4. [`Pacaptr::dispatch`] will call the corresponding trait method, eg.
//!   `.suy()`, according to the combination of flags and options obtained
//!   above.

mod cmd;
pub mod config;

pub use self::{cmd::Pacaptr, config::Config};
use crate::{exec::is_exe, pm::*};

/// Detects the name of the package manager to be used in auto dispatch.
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
            ("emerge", "/usr/bin/emerge"),
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

impl From<Config> for Box<dyn Pm> {
    /// Generates the `Pm` instance according it's name, feeding it with the
    /// current `Config`.
    fn from(cfg: Config) -> Self {
        // If the `Pm` to be used is not stated in any config,
        // we should fall back to automatic detection.
        let pm = cfg.default_pm.as_deref().unwrap_or_else(detect_pm_str);

        #[allow(clippy::match_single_binding)]
        match pm {
            // Chocolatey
            "choco" => Choco { cfg }.boxed(),

            // Scoop
            "scoop" => Scoop { cfg }.boxed(),

            // Homebrew/Linuxbrew
            "brew" => Brew { cfg }.boxed(),

            // Macports
            "port" if cfg!(target_os = "macos") => Port { cfg }.boxed(),

            // Portage for Gentoo
            "emerge" => Emerge { cfg }.boxed(),

            // Apk for Alpine
            "apk" => Apk { cfg }.boxed(),

            // Apt for Debian/Ubuntu/Termux (new versions)
            "apt" => Apt { cfg }.boxed(),

            // Dnf for RedHat
            "dnf" => Dnf { cfg }.boxed(),

            // Zypper for SUSE
            "zypper" => Zypper { cfg }.boxed(),

            // -- External Package Managers --

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

            // Test-only mock package manager
            #[cfg(test)]
            "mockpm" => {
                use self::cmd::tests::MockPm;
                MockPm { cfg }.boxed()
            }

            // Unknown package manager X
            x => Unknown::new(x).boxed(),
        }
    }
}
