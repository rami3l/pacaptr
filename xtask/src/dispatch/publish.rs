use super::{names::*, Runner};
use crate::binary::*;
use anyhow::Result;
use xshell::{cmd, write_file};

#[derive(Debug)]
pub struct Publish {}

impl Runner for Publish {
    fn run(self) -> Result<()> {
        // let Self { artifact, asset } = self;
        let github_env = &std::env::var("GITHUB_ENV")?;

        cmd!("gh config set prompt disabled").run()?;

        // println!(":: Logging into GitHub CLI...");
        // cmd!("gh auth login").run()?;

        let builds = match () {
            _ if cfg!(target_os = "linux") => {
                let linux_x64 = BinaryBuilder::Cross {
                    bin: LINUX_X64,
                    rust_target: targets::LINUX_MUSL.into(),
                };
                vec![linux_x64]
            }
            _ if cfg!(target_os = "windows") => {
                let win_x64 = BinaryBuilder::Native(WIN_X64);
                vec![win_x64]
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

                let mac_x64 = BinaryBuilder::Native(MAC_X64);
                let mac_arm = BinaryBuilder::Cross {
                    bin: MAC_ARM,
                    rust_target: targets::MAC_ARM.into(),
                };
                vec![mac_x64, mac_arm]
            }
            _ => panic!("Unsupported publishing platform"),
        };

        builds.iter().try_for_each(|b| {
            b.build()?;
            b.zip()?;
            b.upload()
        })?;

        if cfg!(target_os = "macos") {
            let mac_univ = BinaryBuilder::Cross {
                bin: MAC_UNIV,
                // ! This is NOT a real rust target. It's here just to fit in.
                rust_target: "universal-apple-darwin".into(),
            };
            let out_dir = mac_univ.bin_dir();
            let out_artifact = &mac_univ.bin().artifact;
            assert_eq!(builds.len(), 2);
            let in_dir0 = builds[0].bin_dir();
            let artifact0 = &builds[0].bin().artifact;
            let in_dir1 = builds[1].bin_dir();
            let artifact1 = &builds[1].bin().artifact;
            cmd!("lipo -create -output {out_dir}{out_artifact} {in_dir0}{artifact0} {in_dir1}{artifact1}")
                .run()?;
            cmd!("chmod +x {out_dir}{out_artifact}").run()?;

            mac_univ.zip()?;
            mac_univ.upload()?;
        }

        Ok(())
    }
}
