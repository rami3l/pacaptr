use super::PackageManager;

pub struct Unknown {
    pub name: String,
}

impl PackageManager for Unknown {
    /// Get the name of the package manager.
    fn name(&self) -> String {
        format!("unknown package manager: {}", self.name)
    }
}
