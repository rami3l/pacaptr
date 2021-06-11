pub mod bump_choco;
pub mod bump_tap;
pub mod publish;

use anyhow::Result;
use regex::Regex;
use std::env;

pub mod names {
    /// The name of the executable.
    pub const CORE: &str = "pacaptr";

    /// The project homepage.
    pub const HOMEPAGE: &str = "https://github.com/rami3l/pacaptr";

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
