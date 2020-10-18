use anyhow::{Error, Result};
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
    pub force_cask: bool,

    #[serde(default)]
    pub no_cache: bool,

    #[serde(default)]
    pub default_pm: Option<String>,
}

impl Config {
    pub fn load() -> Result<Self> {
        let crate_name = clap::crate_name!();
        let config = dirs::home_dir()
            .ok_or_else(|| anyhow!("$HOME path not found"))?
            .join(".config")
            .join(crate_name)
            .join(&format!("{}.toml", crate_name));
        // dbg!(&config);
        let res = confy::load_path(config)?;
        // dbg!(&res);
        Ok(res)
    }
}
