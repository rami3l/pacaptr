use super::{get_ver_from_env, names::*, Runner};
use crate::binary::WIN_X64;
use crate::replace;
use anyhow::{anyhow, Result};
use regex::Regex;
use std::env;
use xml::escape::escape_str_attribute;
use xshell::{cmd, read_file, write_file};

#[derive(Debug)]
pub struct BumpChoco {}

impl Runner for BumpChoco {
    fn run(self) -> Result<()> {
        if !cfg!(target_os = "windows") {
            panic!("This action is meant to run under windows.")
        }

        let tag = get_ver_from_env()?;

        // Remove leading `v` and suffix `-take.X` from the tag.
        let ver = {
            let tag = tag.strip_prefix('v').unwrap_or(&tag);
            Regex::new(r"-?take\.\d+")?.replace(tag, "")
        };

        let release_uri = format!(
            "{homepage}/releases/download/{tag}/{bin}",
            homepage = HOMEPAGE,
            tag = tag,
            bin = WIN_X64.archive()
        );

        println!(":: Downloading release binary...");
        cmd!("powershell iwr {release_uri} -OutFile ./release.tar.gz").run()?;

        println!(":: Extracting release binary...");
        let bin_out_dir = "./dist/choco/tools/";
        cmd!("tar xvf ./release.tar.gz -C {bin_out_dir}").run()?;

        println!(":: Adding license");
        cmd!("cp LICENSE dist/choco/tools/LICENSE.txt").run()?;

        println!(":: Generating Nuspec from template...");
        let nuspec_temp = read_file("dist/choco/pacaptr.template.nuspec")?;
        let nuspec = {
            let version = escape_str_attribute(&ver);
            replace!(nuspec_temp, version)
        };
        let nuspec_path = "dist/choco/pacaptr.nuspec";
        write_file(nuspec_path, nuspec)?;
        cmd!("cat {nuspec_path}").run()?;

        println!(":: Generating `VERIFICATION.txt`...");
        let verif_path = "dist/choco/tools/VERIFICATION.txt";
        let verif_temp_path = "dist/choco/tools/VERIFICATION.template.txt";
        let verif_temp = read_file(verif_temp_path)?;
        let bin_path = format!("{}{}", bin_out_dir, WIN_X64.artifact);
        let algos = &["sha1", "sha256"];
        let checksums = {
            let mut checksums = "".to_owned();
            for algo in algos {
                let line = format!("{}: {}\n", algo, checksum(&bin_path, algo)?);
                checksums.push_str(&line);
            }
            checksums
        };
        let verification = {
            let repository = HOMEPAGE;
            replace!(verif_temp, repository, release_uri, checksums)
        };
        write_file(verif_path, verification)?;
        cmd!("cat {verif_path}").run()?;
        cmd!("rm {verif_temp_path}").run()?;

        println!(":: Setting choco API key...");
        let choco_api_key = env::var("CHOCO_API_KEY")?;
        cmd!("choco apikey --key {choco_api_key} --source https://push.chocolatey.org --verbose")
            .run()?;

        println!(":: Packing up NuGet package...");
        cmd!("choco pack {nuspec_path} --verbose").run()?;

        println!(":: Pushing to choco repository...");
        let ver = ver.as_ref();
        cmd!("choco push pacaptr.{ver}.nupkg --source https://push.chocolatey.org --verbose")
            .run()?;

        Ok(())
    }
}

fn checksum(path: &str, algo: &str) -> Result<String> {
    cmd!("openssl dgst -r -{algo} {path}")
        .read()?
        .split_whitespace()
        .next()
        .map(|s| s.to_owned())
        .ok_or_else(|| anyhow!("Failed to fetch checksum from `openssl dgst`"))
}
