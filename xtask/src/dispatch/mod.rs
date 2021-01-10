mod install;
mod publish;
mod run;

use self::install::Install;
use self::publish::Publish;
use self::run::Run;
use anyhow::Result;
use clap::Clap;

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
    Run(Run),
    Install(Install),
    Publish(Publish),
}

pub trait Runner {
    fn run(self) -> Result<()>;
}
