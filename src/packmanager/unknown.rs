use super::PackManager;

pub struct Unknown {
    pub name: String,
}

impl PackManager for Unknown {
    /// Get the name of the package manager.
    fn name(&self) -> String {
        format!("unknown package manager: {}", self.name)
    }
}
