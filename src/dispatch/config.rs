use serde_derive::{Deserialize, Serialize};

#[derive(Default, Debug, Serialize, Deserialize)]
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
}

impl Config {}
