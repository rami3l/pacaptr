use super::{Runner, CORE};
use anyhow::Result;
use clap::Clap;
use xshell::cmd;

#[derive(Debug, Clap)]
#[clap(about = "Delegate to core's `cargo install`")]
pub struct Install {
    #[clap(name = "KEYWORDS", about = "The rest of the command")]
    pub keywords: Vec<String>,
}

impl Runner for Install {
    fn run(self) -> Result<()> {
        let keywords = self.keywords;
        cmd!("cargo install {CORE} --path ./core {keywords...}").run()?;
        Ok(())
    }
}
