use anyhow::{anyhow, Result};
use xshell::{cmd, Shell};

use super::{get_ver_from_env, names::*, Runner};
use crate::{
    binary::{LINUX_X64, MAC_UNIV},
    replace,
};

#[derive(Debug)]
pub struct BumpTap {}

impl Runner for BumpTap {
    fn run(self) -> Result<()> {
        if cfg!(target_os = "windows") {
            panic!("this action is not meant to run under windows")
        }

        let s = Shell::new()?;
        let version = get_ver_from_env()?;
        let url_prefix = format!(
            "{homepage}/releases/download/{tag}",
            homepage = HOMEPAGE,
            tag = version
        );
        let url_mac = format!(
            "{prefix}/{bin}",
            prefix = url_prefix,
            bin = MAC_UNIV.archive()
        );
        let url_linux = format!(
            "{prefix}/{bin}",
            prefix = url_prefix,
            bin = LINUX_X64.archive()
        );

        println!(":: Getting checksums...");
        let sha256_mac = cmd!(s, "curl -L {url_mac}.sha256").read()?;
        let sha256_mac = sha256_mac
            .split_whitespace()
            .next()
            .ok_or_else(|| anyhow!("Failed to get sha256_mac"))?;
        let sha256_linux = cmd!(s, "curl -L {url_linux}.sha256").read()?;
        let sha256_linux = sha256_linux
            .split_whitespace()
            .next()
            .ok_or_else(|| anyhow!("Failed to get sha256_linux"))?;

        println!(":: Generating new brew Formula...");
        let template = s.read_file("dist/brew/template.rb")?;
        let replaced = replace!(
            template,
            version,
            url_mac,
            sha256_mac,
            url_linux,
            sha256_linux
        );
        let formula = "pacaptr.rb";
        s.write_file(formula, replaced)?;
        cmd!(s, "cat {formula}").run()?;

        println!(":: Uploading new Formula");
        cmd!(s, "gh release upload {version} {formula}").run()?;

        Ok(())
    }
}
