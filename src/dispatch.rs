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
mod config;

pub use self::cmd::Pacaptr;
pub(crate) use self::config::Config;
use crate::{
    exec::is_exe,
    pm::{
        Apk, Apt, Brew, Choco, Conda, Dnf, Emerge, Pip, Pm, Port, Scoop, Tlmgr, Unknown, Xbps,
        Zypper,
    },
};

/// Detects the name of the package manager to be used in auto dispatch.
#[must_use]
fn detect_pm_str<'s>() -> &'s str {
    let pairs: &[(&str, &str)] = match () {
        _ if cfg!(target_os = "windows") => &[("scoop", ""), ("choco", "")],

        _ if cfg!(target_os = "macos") => &[
            ("brew", "/usr/local/bin/brew"),
            ("port", "/opt/local/bin/port"),
            ("apt", "/opt/procursus/bin/apt"),
        ],

        _ if cfg!(target_os = "ios") => &[("apt", "/usr/bin/apt")],

        _ if cfg!(target_os = "linux") => &[
            ("apk", "/sbin/apk"),
            ("apt", "/usr/bin/apt"),
            ("emerge", "/usr/bin/emerge"),
            ("dnf", "/usr/bin/dnf"),
            ("xbps-install", "/usr/bin/xbps-install"),
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
    fn from(mut cfg: Config) -> Self {
        // If the `Pm` to be used is not stated in any config,
        // we should fall back to automatic detection and overwrite `cfg`.
        let pm = cfg.default_pm.get_or_insert_with(|| detect_pm_str().into());

        #[allow(clippy::match_single_binding)]
        match pm as _ {
            // Chocolatey
            "choco" => Choco::new(cfg).boxed(),

            // Scoop
            "scoop" => Scoop::new(cfg).boxed(),

            // Homebrew/Linuxbrew
            "brew" => Brew::new(cfg).boxed(),

            // Macports
            "port" if cfg!(target_os = "macos") => Port::new(cfg).boxed(),

            // Apt for Debian/Ubuntu/Termux (newer versions)
            "apt" => Apt::new(cfg).boxed(),

            // Apk for Alpine
            "apk" => Apk::new(cfg).boxed(),

            // Dnf for RedHat
            "dnf" => Dnf::new(cfg).boxed(),

            // Portage for Gentoo
            "emerge" => Emerge::new(cfg).boxed(),

            // Xbps for Void Linux
            "xbps" | "xbps-install" => Xbps::new(cfg).boxed(),

            // Zypper for SUSE
            "zypper" => Zypper::new(cfg).boxed(),

            // -- External Package Managers --

            // Conda
            "conda" => Conda::new(cfg).boxed(),

            // Pip
            "pip" | "pip3" => Pip::new(cfg).boxed(),

            // Tlmgr
            "tlmgr" => Tlmgr::new(cfg).boxed(),

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
