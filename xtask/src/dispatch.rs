use std::todo;

use anyhow::Result;
use clap::Clap;
use xshell::cmd;

const CORE: &str = "pacaptr";
const LINUX_MUSL: &str = "x86_64-unknown-linux-musl";

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
    /*
    /// Sets a custom config file. Could have been an Option<T> with no default too
    #[clap(short, long, default_value = "default.conf")]
    config: String,
    /// Some input. Because this isn't an Option<T> it's required to be used
    input: String,
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
    */
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

#[derive(Debug, Clap)]
#[clap(about = "Build relese and upload to GitHub releases.")]
pub struct Publish {
    #[clap(long, alias = "exe", about = "Name of the executable")]
    pub artifact: String,

    #[clap(long, about = "Name of the asset")]
    pub asset: String,
}

impl Runner for Publish {
    fn run(self) -> Result<()> {
        let Self { artifact, asset } = self;

        println!("Building the binary in `release` mode...");
        if cfg!(target_os = "linux") {
            // In Linux, we need to add the `musl` target first.
            cmd!("rustup target add {LINUX_MUSL}}").run()?;
            cmd!("cargo build --release --locked --target={LINUX_MUSL}").run()?;
        } else {
            cmd!("cargo build --release --locked").run()?;
        }

        println!("Zipping the binary...");
        cmd!("tar czvf {asset}.tar.gz -C ./target/release/ {artifact}").run()?;

        println!("Generating sha256...");
        cmd!("openssl dgst -r -sha256 {asset}.tar.gz > {asset}.tar.gz.sha256").run()?;

        println!("Uploading binary...");
        todo!();
        println!("Uploading binary sha256...");
        todo!();

        #[cfg(target_os = "windows")]
        {
            println!("Publishing to `choco`...");
            todo!()
        }

        #[cfg(target_os = "macos")]
        {
            println!("Publishing to `homebrew tap`...");
            todo!()
        }

        Ok(())
    }
}
