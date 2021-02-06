mod bump_tap;
mod publish;

use self::bump_tap::BumpTap;
use self::publish::Publish;
use anyhow::Result;
use clap::Clap;
use regex::Regex;
use std::env;

/// The name of the executable.
const CORE: &str = "pacaptr";

/// The project homepage.
const HOMEPAGE: &str = "https://github.com/rami3l/pacaptr";

/// The command line options to be collected.
#[derive(Debug, Clap)]
#[clap(
    about = clap::crate_description!(),
    version = clap::crate_version!(),
    author = clap::crate_authors!(),
    setting = clap::AppSettings::ColoredHelp,
    setting = clap::AppSettings::ArgRequiredElseHelp,
)]
pub enum Opts {
    Publish(Publish),
    BumpTap(BumpTap),
}

pub trait Runner {
    fn run(self) -> Result<()>;
}

/// Strip the `refs/*/` prefix from `GITHUB_REF` to get a version string.
fn get_ver<S: AsRef<str>>(gh_ref: S) -> String {
    Regex::new("refs/.*/")
        .unwrap()
        .replace(gh_ref.as_ref(), "")
        .to_string()
}

/// Strip the `refs/*/` prefix from `GITHUB_REF` to get a version string.
/// Where the value of `GITHUB_REF` is read from environment variables.
fn get_ver_from_env() -> Result<String> {
    Ok(get_ver(env::var("GITHUB_REF")?))
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
