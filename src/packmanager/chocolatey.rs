use super::PackManager;

pub struct Chocolatey {
    pub dry_run: bool,
    pub no_confirm: bool,
}

impl PackManager for Chocolatey {}
