mod publish;

use self::publish::Publish;
use anyhow::Result;
use clap::Clap;

/// The name of the executable.
const CORE: &str = "pacaptr";

/// The command line options to be collected.
#[derive(Debug, Clap)]
#[clap(
    about = clap::crate_description!(),
    version = clap::crate_version!(),
    author = clap::crate_authors!(),
    setting = clap::AppSettings::ColoredHelp,
    setting = clap::AppSettings::ArgRequiredElseHelp,
)]
pub struct Opts {
    #[clap(subcommand)]
    pub subcmd: SubCmd,
}

#[derive(Debug, Clap)]
pub enum SubCmd {
    Publish(Publish),
}

pub trait Runner {
    fn run(self) -> Result<()>;
}
