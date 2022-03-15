use std::fs;

use anyhow::Result;
use xshell::{cmd, Shell};

use super::{names::*, Runner};
use crate::binary::*;

#[derive(Debug)]
pub struct Publish {}

impl Runner for Publish {
    fn run(self) -> Result<()> {
        // let Self { artifact, asset } = self;
        let s = Shell::new()?;

        cmd!(s, "gh config set prompt disabled").run()?;

        // println!(":: Logging into GitHub CLI...");
        // cmd!(s, "gh auth login").run()?;

        let publish = |b: &BinaryBuilder| {
            b.build()?;
            b.zip()?;
            b.upload()
        };

        match () {
            _ if cfg!(target_os = "linux") => {
                let linux_x64 = BinaryBuilder::Cross {
                    bin: LINUX_X64,
                    rust_target: targets::LINUX_MUSL,
                };
                publish(&linux_x64)?;
            }
            _ if cfg!(target_os = "windows") => {
                let win_x64 = BinaryBuilder::Native(WIN_X64);
                publish(&win_x64)?;
            }
            _ if cfg!(target_os = "macos") => {
                let mac_x64 = BinaryBuilder::Native(MAC_X64);
                let mac_arm = BinaryBuilder::Cross {
                    bin: MAC_ARM,
                    rust_target: targets::MAC_ARM,
                };
                publish(&mac_x64)?;
                publish(&mac_arm)?;

                let mac_univ = BinaryBuilder::Cross {
                    bin: MAC_UNIV,
                    // ! This is NOT a real rust target. It's here just to fit in.
                    rust_target: "universal-apple-darwin",
                };

                let out_dir = &mac_univ.bin_dir();
                let out_artifact = mac_univ.bin().artifact;
                let in_dir0 = &mac_x64.bin_dir();
                let artifact0 = mac_x64.bin().artifact;
                let in_dir1 = &mac_arm.bin_dir();
                let artifact1 = mac_arm.bin().artifact;
                fs::create_dir_all(out_dir)?;
                cmd!(s, "lipo -create -output {out_dir}{out_artifact} {in_dir0}{artifact0} {in_dir1}{artifact1}")
                    .run()?;
                cmd!(s, "chmod +x {out_dir}{out_artifact}").run()?;

                mac_univ.zip()?;
                mac_univ.upload()?;
            }
            _ => panic!("unsupported publishing platform"),
        };

        Ok(())
    }
}
