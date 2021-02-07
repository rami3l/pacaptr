use super::{get_ver_from_env, Runner, ARCHIVE_LINUX, ARCHIVE_MAC, HOMEPAGE};
use crate::replace;
use anyhow::{anyhow, Result};
use xshell::{cmd, read_file, write_file};

#[derive(Debug)]
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
        let url_mac: String = format!("{prefix}/{bin}", prefix = url_prefix, bin = ARCHIVE_MAC);
        let url_linux: String = format!("{prefix}/{bin}", prefix = url_prefix, bin = ARCHIVE_LINUX);

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
        let template = read_file("dist/brew/template.rb")?;
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
        cmd!("gh release upload {version} {formula}").run()?;

        Ok(())
    }
}
