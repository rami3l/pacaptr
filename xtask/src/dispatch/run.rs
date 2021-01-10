use super::{Runner, CORE};
use anyhow::Result;
use clap::Clap;
use xshell::cmd;

#[derive(Debug, Clap)]
#[clap(about = "Delegate to core's `cargo run`")]
pub struct Run {
    #[clap(name = "KEYWORDS", about = "The rest of the command")]
    pub keywords: Vec<String>,
}

impl Runner for Run {
    fn run(self) -> Result<()> {
        let keywords = self.keywords;
        cmd!("cargo run -p {CORE} {keywords...}").run()?;
        Ok(())
    }
}
