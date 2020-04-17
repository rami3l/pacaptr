use super::PackManager;
use regex::Regex;
use subprocess::{Exec, Redirection};

pub struct Homebrew {
    dry_run: bool,
    cask: bool,
}

enum CaskState {
    NotFound,
    Unneeded,
    Needed,
}

impl Homebrew {
    fn search(&self, pack: &str) -> Result<CaskState, String> {
        let out = Exec::cmd("brew")
            .arg("info")
            .arg(pack)
            .stdout(Redirection::Pipe)
            .stderr(Redirection::Merge)
            .capture()
            .map_err(|_| "Could not capture stdout".to_string())?
            .stdout_str();

        let code = {
            lazy_static! {
                static ref RE_BOTTLE: Regex =
                    Regex::new(r"No available formula with the name").unwrap();
                static ref RE_CASK: Regex = Regex::new(r"Found a cask named").unwrap();
            }

            if RE_BOTTLE.find(&out).is_some() {
                if RE_CASK.find(&out).is_some() {
                    CaskState::Needed
                } else {
                    CaskState::NotFound
                }
            } else {
                CaskState::Unneeded
            }
        };

        Ok(code)
    }
}

impl PackManager for Homebrew {}
