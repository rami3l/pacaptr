use super::{Runner, CORE};
use anyhow::Result;
use clap::Clap;
use regex::Regex;
use std::env;
use xshell::{cmd, write_file};

const LINUX_MUSL: &str = "x86_64-unknown-linux-musl";

#[derive(Debug, Clap)]
#[clap(about = "Build release and upload to GitHub releases.")]
pub struct Publish {
    #[clap(long, alias = "exe", about = "Name of the executable")]
    pub artifact: String,

    #[clap(long, about = "Name of the asset")]
    pub asset: String,
}

impl Runner for Publish {
    fn run(self) -> Result<()> {
        let Self { artifact, asset } = self;

        cmd!("gh config set prompt disabled").run()?;

        // println!(":: Logging into GitHub CLI...");
        // cmd!("gh auth login").run()?;

        println!(":: Building the binary in `release` mode...");
        if cfg!(target_os = "linux") {
            // In Linux, we need to add the `musl` target first.
            cmd!("rustup target add {LINUX_MUSL}").run()?;
            cmd!("cargo build --verbose --bin {CORE} --release --locked --target={LINUX_MUSL}")
                .run()?;
        } else {
            cmd!("cargo build --verbose --bin {CORE} --release --locked").run()?;
        }

        println!(":: Zipping the binary...");
        let bin_dir = if cfg!(target_os = "linux") {
            format!("./target/{}/release/", LINUX_MUSL)
        } else {
            "./target/release/".to_owned()
        };

        cmd!("tar czvf {asset}.tar.gz -C {bin_dir} {artifact}").run()?;

        println!(":: Generating sha256...");
        let shasum = cmd!("openssl dgst -r -sha256 {asset}.tar.gz").read()?;
        write_file(format!("{}.tar.gz.sha256", asset), shasum)?;

        println!(":: Uploading binary and sha256...");
        let gh_ref = env::var("GITHUB_REF")?;
        let tag = get_ver(gh_ref);
        cmd!("gh release create {tag} {asset}.tar.gz {asset}.tar.gz.sha256")
            .run()
            .or_else(|_| {
                cmd!("gh release upload {tag} {asset}.tar.gz {asset}.tar.gz.sha256").run()
            })?;

        /*
        #[cfg(target_os = "windows")]
        {
            println!("Publishing to `choco`...");
            todo!()
        }

        #[cfg(target_os = "macos")]
        {
            // ! Generation of tap script requires binaries for both macOS and Linux.
            println!("Publishing to `homebrew tap`...");
            todo!()
        }
        */

        Ok(())
    }
}

/// Strip the `ref/*/` prefix from `GITHUB_REF` to get a version string.
fn get_ver<S: AsRef<str>>(gh_ref: S) -> String {
    Regex::new("refs/.*/")
        .unwrap()
        .replace(gh_ref.as_ref(), "")
        .to_string()
}
