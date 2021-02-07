pub mod bump_choco;
pub mod bump_tap;
pub mod publish;

use anyhow::Result;
use const_format::concatcp;
use regex::Regex;
use std::env;

/// The name of the executable.
pub const CORE: &str = "pacaptr";

/// The project homepage.
pub const HOMEPAGE: &str = "https://github.com/rami3l/pacaptr";

pub const ARTIFACT_WINDOWS: &str = concatcp!(CORE, ".exe");
pub const ARTIFACT_MAC: &str = CORE;
pub const ARTIFACT_LINUX: &str = CORE;

pub const ASSET_WINDOWS: &str = concatcp!(CORE, "-windows-amd64");
pub const ASSET_MAC: &str = concatcp!(CORE, "-macos-amd64");
pub const ASSET_LINUX: &str = concatcp!(CORE, "-linux-amd64");

pub const ARCHIVE_WINDOWS: &str = concatcp!(ASSET_WINDOWS, ".tar.gz");
pub const ARCHIVE_MAC: &str = concatcp!(ASSET_MAC, ".tar.gz");
pub const ARCHIVE_LINUX: &str = concatcp!(ASSET_LINUX, ".tar.gz");

pub trait Runner {
    fn run(self) -> Result<()>;
}

/// Strip the `refs/*/` prefix from `GITHUB_REF` to get a version string.
fn get_ver(gh_ref: impl AsRef<str>) -> Result<String> {
    Ok(Regex::new(r"refs/\w+/")?
        .replace(gh_ref.as_ref(), "")
        .to_string())
}

/// Strip the `refs/*/` prefix from `GITHUB_REF` to get a version string.
/// Where the value of `GITHUB_REF` is read from environment variables.
fn get_ver_from_env() -> Result<String> {
    get_ver(env::var("GITHUB_REF")?)
}

#[macro_export]
macro_rules! replace {
    ( $s:expr, $( $x:expr ),* ) => {
        {
            let mut s = $s;
            $(s = s.replace(concat!("{", stringify!($x), "}"), &$x);)*
            s
        }
    };
}
