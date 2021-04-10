use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::{
    env,
    path::{Path, PathBuf},
};

/// The environment variable name for custom config file path.
const CONFIG_ENV_VAR: &str = "PACAPTR_CONFIG";

/// Configurations that may vary when running the package manager.
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
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
    /// This method will almost always success, and will only fail if `$HOME` is not found.
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

    /// Gets the custom config file path specified by the `PACAPTR_CONFIG` environment variable.
    pub fn custom_path() -> Result<PathBuf> {
        env::var(CONFIG_ENV_VAR)
            .map_err(|e| Error::ConfigError {
                msg: format!("Config path environment variable not found: {}", e),
            })
            .map(|p| Path::new(&p).to_owned())
    }

    /// Loads up the config file from the user-specified path.
    ///
    /// I decided not to trash user's `$HOME` without their permission, so:
    /// - If the user hasn't yet specified any path to look at, we will look for the config file in the default path.
    /// - If the config file is not present anyway, a default one will be loaded with [`Default::default`], and no files will be written.
    pub fn load() -> Result<Self> {
        let path = Config::custom_path().or_else(|_| Config::default_path())?;
        path.exists()
            .then(|| {
                confy::load_path(&path).map_err(|_e| Error::ConfigError {
                    msg: format!("Failed to read config at `{:?}`", &path),
                })
            })
            .transpose()
            .map(|cfg| cfg.unwrap_or_default())
    }
}
