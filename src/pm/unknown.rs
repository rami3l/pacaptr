//! An empty mapping for unidentified package managers.

use async_trait::async_trait;

use super::Pm;
use crate::config::Config;

/// An empty mapping for unidentified package managers.
#[derive(Debug)]
pub struct Unknown {
    name: String,
    cfg: Config,
}

impl Unknown {
    #[must_use]
    /// Creates a new [`Unknown`] package manager with the given name.
    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: format!("unknown package manager: {name}"),
            cfg: Config::default(),
        }
    }
}

#[async_trait]
impl Pm for Unknown {
    /// Gets the name of the package manager.
    fn name(&self) -> &str {
        &self.name
    }

    fn cfg(&self) -> &Config {
        &self.cfg
    }
}
