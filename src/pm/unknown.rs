use super::Pm;
use crate::dispatch::config::Config;

#[derive(Debug)]
pub struct Unknown {
    pub name: String,
    pub cfg: Config,
}

impl Unknown {
    #[must_use]
    /// Creates a new [`Unknown`] package manager with the given name.
    pub fn new(name: &str) -> Self {
        Unknown {
            name: format!("unknown package manager: {}", name),
            cfg: Config::default(),
        }
    }
}

impl Pm for Unknown {
    /// Gets the name of the package manager.
    fn name(&self) -> &str {
        &self.name
    }

    fn cfg(&self) -> &Config {
        &self.cfg
    }
}
