use super::{get_ver_from_env, names::*, Runner};
use anyhow::Result;
use xshell::{cmd, write_file};

#[derive(Debug)]
pub struct Publish {
    pub artifact: String,
    pub asset: String,
}

impl Runner for Publish {
    fn run(self) -> Result<()> {
        // let Self { artifact, asset } = self;
        let github_env = &std::env::var("GITHUB_ENV")?;

        cmd!("gh config set prompt disabled").run()?;

        // println!(":: Logging into GitHub CLI...");
        // cmd!("gh auth login").run()?;

        println!(":: Building the binary in `release` mode...");
        let build_native = || cmd!("cargo build --verbose --bin {CORE} --release --locked").run();

        let build_for_target = |target: &str| {
            cmd!("rustup target add {target}").run()?;
            cmd!("cargo build --verbose --bin {CORE} --release --locked --target={target}").run()
        };

        match () {
            _ if cfg!(target_os = "linux") => {
                build_for_target(targets::LINUX_MUSL)?;
            }
            _ if cfg!(target_os = "windows") => {
                build_native()?;
            }
            _ if cfg!(target_os = "macos") => {
                // Set environment variables.
                let sdk_root = cmd!("xcrun -sdk macosx11.1 --show-sdk-path").read()?;
                let dev_target =
                    cmd!("xcrun -sdk macosx11.1 --show-sdk-platform-version").read()?;
                write_file(github_env, format!("SDKROOT={}", sdk_root))?;
                write_file(
                    github_env,
                    format!("MACOSX_DEPLOYMENT_TARGET={}", dev_target),
                )?;

                build_native()?;
                build_for_target(targets::MAC_ARM)?;
            }
            _ => panic!("Unsupported publishing platform"),
        };

        println!(":: Zipping the binary...");
        let zip_bin = |bin_dir| {
            cmd!("tar czvf {asset}.tar.gz -C {bin_dir} {artifact}").run()?;
        }
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
        let tag = get_ver_from_env()?;
        cmd!("gh release upload {tag} {asset}.tar.gz {asset}.tar.gz.sha256").run()?;

        Ok(())
    }
}
