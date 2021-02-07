use super::{get_ver_from_env, Runner, HOMEPAGE};
use crate::replace;
use anyhow::{anyhow, Result};
use clap::Clap;
use xshell::{cmd, write_file};

const BIN_MAC: &str = "pacaptr-macos-amd64.tar.gz";
const BIN_LINUX: &str = "pacaptr-linux-amd64.tar.gz";

#[derive(Debug, Clap)]
#[clap(about = "Bump homebrew tap formula version")]
pub struct BumpTap {}

impl Runner for BumpTap {
    fn run(self) -> Result<()> {
        if cfg!(target_os = "windows") {
            panic!("This action is not meant to run under windows.")
        }

        let version = get_ver_from_env()?;
        let url_prefix = format!(
            "{homepage}/releases/download/{tag}",
            homepage = HOMEPAGE,
            tag = version
        );
        let url_mac: String = format!("{prefix}/{bin}", prefix = url_prefix, bin = BIN_MAC);
        let url_linux: String = format!("{prefix}/{bin}", prefix = url_prefix, bin = BIN_LINUX);

        println!(":: Getting checksums...");
        let sha256_mac = cmd!("curl -L {url_mac}.sha256").read()?;
        let sha256_mac = sha256_mac
            .split_whitespace()
            .next()
            .ok_or_else(|| anyhow!("Failed to get sha256_mac"))?;
        let sha256_linux = cmd!("curl -L {url_linux}.sha256").read()?;
        let sha256_linux = sha256_linux
            .split_whitespace()
            .next()
            .ok_or_else(|| anyhow!("Failed to get sha256_linux"))?;

        println!(":: Generating new brew Formula...");
        let template = cmd!("cat dist/brew/template.rb").read()?;
        let replaced = replace!(
            template,
            version,
            url_mac,
            sha256_mac,
            url_linux,
            sha256_linux
        );
        let formula = "pacaptr.rb";
        write_file(formula, replaced)?;
        cmd!("cat {formula}").run()?;

        println!(":: Uploading new Formula");
        cmd!("gh release upload {formula}").run()?;

        Ok(())
    }
}
