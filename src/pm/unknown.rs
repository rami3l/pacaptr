use super::Pm;
use crate::dispatch::config::Config;

pub struct Unknown {
    pub name: String,
    pub cfg: Config,
}

impl Unknown {
    pub fn new(name: &str) -> Self {
        Unknown {
            name: format!("unknown package manager: {}", name),
            cfg: Default::default(),
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
