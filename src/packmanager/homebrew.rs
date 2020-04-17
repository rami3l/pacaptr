use super::PackManager;

pub struct Homebrew {
    dry_run: bool,
    cask: bool,
}

impl PackManager for Homebrew {}
