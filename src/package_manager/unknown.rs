use super::PackageManager;
use crate::dispatch::config::Config;

pub struct Unknown {
    pub name: String,
}

impl PackageManager for Unknown {
    /// Get the name of the package manager.
    fn name(&self) -> String {
        format!("unknown package manager: {}", self.name)
    }

    fn cfg(&self) -> Config {
        Default::default()
    }
}
