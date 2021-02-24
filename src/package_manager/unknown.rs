use super::PackageManager;
use crate::dispatch::config::Config;

pub struct Unknown {
    pub name: String,
    pub cfg: Config,
}

impl Unknown {
    pub fn new(name: &str) -> Self {
        Unknown {
            name: name.to_owned(),
            cfg: Default::default(),
        }
    }
}

impl PackageManager for Unknown {
    /// Get the name of the package manager.
    fn name(&self) -> String {
        format!("unknown package manager: {}", self.name)
    }

    fn cfg(&self) -> &Config {
        &self.cfg
    }
}
