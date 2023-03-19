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
    pub dry_run: bool,

    /// Prevent reinstalling previously installed packages.
    pub needed: bool,

    /// Answer yes to every question.
    pub no_confirm: bool,

    /// Remove cache after installation.
    pub no_cache: bool,

    /// The default package manager to be invoked.
    pub default_pm: Option<String>,
}

impl Config {
    /// The default config file path is `$HOME/.config/pacaptr/pacaptr.toml`.
    fn default_path() -> Option<PathBuf> {
        dirs_next::home_dir().map(|home| {
            home.join(".config")
                .join(CRATE_NAME)
                .join(format!("{CRATE_NAME}.toml"))
        })
    }

    /// Gets the custom config file path specified by the `PACAPTR_CONFIG`
    /// environment variable.
    fn custom_path() -> Option<PathBuf> {
        env::var(CONFIG_FILE_ENV).ok().map(PathBuf::from)
    }

    /// Returns the config [`Provider`] from the user-specified file path.
    #[must_use]
    pub fn file_provider() -> impl Provider {
        Self::custom_path()
            .or_else(Self::default_path)
            .map(Toml::file)
            .map_or_else(Figment::new, Figment::from)
    }

    /// Returns the environment config [`Provider`].
    #[must_use]
    pub fn env_provider() -> impl Provider {
        Env::prefixed(CONFIG_ITEM_ENV_PREFIX)
    }
}
