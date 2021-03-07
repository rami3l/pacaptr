use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};

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
    pub fn load() -> Result<Self> {
        let crate_name = clap::crate_name!();
        // The configuration file is `$HOME/.config/pacaptr/pacaptr.toml`.
        let config = dirs::home_dir()
            .ok_or_else(|| Error::ConfigError {
                msg: "$HOME path not found".into(),
            })?
            .join(".config")
            .join(crate_name)
            .join(&format!("{}.toml", crate_name));
        // dbg!(&config);
        let res = confy::load_path(config).map_err(|_e| Error::ConfigError {
            msg: "Failed to read config".into(),
        })?;
        // dbg!(&res);
        Ok(res)
    }
}
