pub mod config;
mod opt;

use crate::exec::is_exe;

pub use self::opt::Opts;

/// Detect the name of the package manager to be used in auto dispatch.
pub fn detect_pm<'s>() -> &'s str {
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
