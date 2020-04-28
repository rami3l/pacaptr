use super::PackManager;

pub struct Dpkg {
    pub dry_run: bool,
    pub no_confirm: bool,
}

impl PackManager for Dpkg {}
