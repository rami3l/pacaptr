pub mod bump_choco;
pub mod bump_tap;
pub mod publish;

use anyhow::Result;
use regex::Regex;
use std::env;

pub mod names {
    use const_format::concatcp;

    /// The name of the executable.
    pub const CORE: &str = "pacaptr";

    /// The project homepage.
    pub const HOMEPAGE: &str = "https://github.com/rami3l/pacaptr";

    pub const ARTIFACT_WINDOWS: &str = concatcp!(CORE, ".exe");
    pub const ARTIFACT_MAC: &str = CORE;
    pub const ARTIFACT_LINUX: &str = CORE;

    pub const ASSET_WIN_X64: &str = concatcp!(CORE, "-windows-amd64");
    pub const ASSET_MAC_X64: &str = concatcp!(CORE, "-macos-amd64");
    pub const ASSET_MAC_ARM: &str = concatcp!(CORE, "-macos-aarch64");
    pub const ASSET_MAC_UNIV: &str = concatcp!(CORE, "-macos-universal");
    pub const ASSET_LINUX_X64: &str = concatcp!(CORE, "-linux-amd64");

    pub const ARCHIVE_WIN_X64: &str = concatcp!(ASSET_WIN_X64, ".tar.gz");
    pub const ARCHIVE_MAC_X64: &str = concatcp!(ASSET_MAC_X64, ".tar.gz");
    pub const ARCHIVE_MAC_ARM: &str = concatcp!(ASSET_MAC_ARM, ".tar.gz");
    pub const ARCHIVE_MAC_UNIV: &str = concatcp!(ASSET_MAC_UNIV, ".tar.gz");
    pub const ARCHIVE_LINUX_X64: &str = concatcp!(ASSET_LINUX_X64, ".tar.gz");

    pub mod targets {
        pub const MAC_ARM: &str = "aarch64-apple-darwin";
        pub const LINUX_MUSL: &str = "x86_64-unknown-linux-musl";
    }
}

pub trait Runner {
    fn run(self) -> Result<()>;
}

/// Strip the `refs/*/` prefix from `GITHUB_REF` to get a version string.
pub fn get_ver(gh_ref: impl AsRef<str>) -> Result<String> {
    Ok(Regex::new(r"refs/\w+/")?
        .replace(gh_ref.as_ref(), "")
        .to_string())
}

/// Strip the `refs/*/` prefix from `GITHUB_REF` to get a version string.
/// Where the value of `GITHUB_REF` is read from environment variables.
pub fn get_ver_from_env() -> Result<String> {
    get_ver(env::var("GITHUB_REF")?)
}

#[macro_export]
macro_rules! replace {
    ( $s:expr, $( $x:expr ),* ) => {{
        let mut s = $s;
        $(s = s.replace(concat!("{", stringify!($x), "}"), &$x);)*
        s
    }};
}
