#![doc = docs_self!()]

use async_trait::async_trait;
use indoc::indoc;

use super::Pm;
use crate::dispatch::Config;

macro_rules! docs_self {
    () => {
        indoc! {"
            An empty mapping for unidentified package managers.
        "}
    };
}

#[doc = docs_self!()]
#[derive(Debug)]
pub(crate) struct Unknown {
    name: String,
    cfg: Config,
}

impl Unknown {
    #[must_use]
    /// Creates a new [`Unknown`] package manager with the given name.
    pub(crate) fn new(name: &str) -> Self {
        Unknown {
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
