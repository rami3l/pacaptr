use super::Runner;
use anyhow::Result;
use clap::Clap;
use xshell::cmd;

const LINUX_MUSL: &str = "x86_64-unknown-linux-musl";

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
            cmd!("rustup target add {LINUX_MUSL}").run()?;
            cmd!("cargo build --release --locked --target={LINUX_MUSL}").run()?;
        } else {
            cmd!("cargo build --release --locked").run()?;
        }

        println!("Zipping the binary...");
        cmd!("tar czvf {asset}.tar.gz -C ./target/release/ {artifact}").run()?;

        println!("Generating sha256...");
        cmd!("openssl dgst -r -sha256 {asset}.tar.gz > {asset}.tar.gz.sha256").run()?;

        println!("Uploading binary and sha256...");
        cmd!("gh release upload $GITHUB_REF {asset}.tar.gz {asset}.tar.gz.sha256").run()?;

        /*
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
        */

        Ok(())
    }
}
