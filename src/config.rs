//! APIs for reading [`pacaptr`](crate) configurations from the filesystem.
//!
//! I decided not to trash user's `$HOME` without their permission, so:
//! - If the user hasn't yet specified any path to look at, we will look for the
//!   config file in the default path.
//! - If the config file is not present anyway, a default one will be loaded
//!   with [`Default::default`], and no files will be written.
//! - Any config item can be overridden by the corresponding `PACAPTR_*`
//!   environment variable. For example, `PACAPTR_NEEDED=false` is prioritized
//!   over `needed = true` in `pacaptr.toml`.

use std::{env, path::PathBuf};

use figment::{
    providers::{Env, Format, Toml},
    Figment, Provider,
};
use serde::{Deserialize, Serialize};
use tap::prelude::*;

/// The crate name.
const CRATE_NAME: &str = clap::crate_name!();

/// The environment variable prefix for config item literals.
const CONFIG_ITEM_ENV_PREFIX: &str = "PACAPTR_";

/// The environment variable name for custom config file path.
const CONFIG_FILE_ENV: &str = "PACAPTR_CONFIG";

/// Configurations that may vary when running the package manager.
#[must_use]
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct Config {
    /// Perform a dry run.
    #[serde(default)]
    pub dry_run: bool,

    /// Prevent reinstalling previously installed packages.
    #[serde(default)]
    pub needed: bool,

    /// Answer yes to every question.
    #[serde(default)]
    pub no_confirm: bool,

    /// Remove cache after installation.
    #[serde(default)]
    pub no_cache: bool,

    /// Suppress log output.
    pub quiet: Option<bool>,

    /// The default package manager to be invoked.
    pub default_pm: Option<String>,
}

impl Config {
    /// Returns the value of the `quiet` flag if it is present,
    /// otherwise returns whether the current `stdout` is **not** a TTY.
    #[must_use]
    pub fn quiet(&self) -> bool {
        self.quiet
            .unwrap_or_else(|| !console::Term::stdout().is_term())
    }

    /// Performs a left-biased join of two `Config`s.
    pub fn join(&self, other: Self) -> Self {
        Self {
            dry_run: self.dry_run || other.dry_run,
            needed: self.needed || other.dry_run,
            no_confirm: self.no_confirm || other.no_confirm,
            no_cache: self.no_cache || other.no_cache,
            quiet: self.quiet.or(other.quiet),
            default_pm: self.default_pm.clone().or(other.default_pm),
        }
    }

    /// The default config file path is defined with the following precedence:
    ///
    /// - `$XDG_CONFIG_HOME/pacaptr/pacaptr.toml`, if `$XDG_CONFIG_HOME` is set;
    /// - `$HOME/.config/pacaptr/pacaptr.toml`.
    ///
    /// This aligns with `fish`'s behavior.
    /// See: <https://github.com/fish-shell/fish-shell/issues/3170#issuecomment-228311857>
    fn default_path() -> Option<PathBuf> {
        env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| dirs_next::home_dir().map(|p| p.join(".config")))
            .tap_some_mut(|p| {
                p.extend([CRATE_NAME, &format!("{CRATE_NAME}.toml")]);
            })
    }

    /// Gets the custom config file path specified by the `PACAPTR_CONFIG`
    /// environment variable.
    fn custom_path() -> Option<PathBuf> {
        env::var_os(CONFIG_FILE_ENV).map(PathBuf::from)
    }

    /// Returns the config [`Provider`] from the custom or default config file
    /// path.
    #[must_use]
    pub fn file_provider() -> impl Provider {
        Self::custom_path()
            .or_else(Self::default_path)
            .map_or_else(Figment::new, |f| Figment::from(Toml::file(f)))
    }

    /// Returns the environment config [`Provider`].
    #[must_use]
    pub fn env_provider() -> impl Provider {
        Env::prefixed(CONFIG_ITEM_ENV_PREFIX)
    }
}
