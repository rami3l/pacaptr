use super::PackManager;
use crate::error::Error;
use crate::exec::{self, Mode};
use regex::Regex;

pub struct Homebrew {
    pub dry_run: bool,
    pub force_cask: bool,
}

enum CaskState {
    NotFound,
    Unneeded,
    Needed,
}

impl Homebrew {
    fn search(&self, pack: &str) -> Result<CaskState, Error> {
        let out_bytes = exec::exec("brew", &["info", pack], Mode::Mute)?;
        let out = String::from_utf8(out_bytes).unwrap();

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
