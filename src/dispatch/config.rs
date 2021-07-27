//! APIs for reading [`pacaptr`](crate) configurations from the filesystem.

use std::{env, path::PathBuf};

use serde::{Deserialize, Serialize};

use crate::error::{Error, Result};

/// The environment variable name for custom config file path.
const CONFIG_ENV_VAR: &str = "PACAPTR_CONFIG";

/// Configurations that may vary when running the package manager.
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct Config {
    #[serde(default)]
    pub dry_run: bool,

    #[serde(default)]
    pub needed: bool,

    #[serde(default)]
    pub no_confirm: bool,

    #[serde(default)]
    pub no_cache: bool,

    #[serde(default)]
    pub default_pm: Option<String>,
}

impl Config {
    /// The default config file path is `$HOME/.config/pacaptr/pacaptr.toml`.
    ///
    /// # Errors
    /// This function returns an [`Error::ConfigError`] when `$HOME` is not
    /// found.
    #[allow(trivial_numeric_casts)]
    pub fn default_path() -> Result<PathBuf> {
        let crate_name = clap::crate_name!();
        dirs_next::home_dir()
            .ok_or_else(|| Error::ConfigError {
                msg: "$HOME path not found".into(),
            })
            .map(|p| {
                p.join(".config")
                    .join(crate_name)
                    .join(&format!("{}.toml", crate_name))
            })
    }

    /// Gets the custom config file path specified by the `PACAPTR_CONFIG`
    /// environment variable.
    ///
    /// # Errors
    /// This function returns an [`Error::ConfigError`] when [`CONFIG_ENV_VAR`]
    /// is not found.
    pub fn custom_path() -> Result<PathBuf> {
        env::var(CONFIG_ENV_VAR)
            .map_err(|e| Error::ConfigError {
                msg: format!("Config path environment variable not found: {}", e),
            })
            .map(PathBuf::from)
    }

    /// Loads up the config file from the user-specified path.
    ///
    /// I decided not to trash user's `$HOME` without their permission, so:
    /// - If the user hasn't yet specified any path to look at, we will look for
    ///   the config file in the default path.
    /// - If the config file is not present anyway, a default one will be loaded
    ///   with [`Default::default`], and no files will be written.
    ///
    /// # Errors
    /// This function returns an [`Error::ConfigError`] when the config file
    /// loading fails.
    pub fn load() -> Result<Self> {
        let path = Config::custom_path().or_else(|_| Config::default_path())?;
        path.exists()
            .then(|| confy::load_path(&path))
            .transpose()
            .map_err(|_e| Error::ConfigError {
                msg: format!("Failed to read config at `{:?}`", &path),
            })
            .map(Option::unwrap_or_default)
    }
}
