mod dispatch;

use crate::dispatch::{
    bump_choco::BumpChoco, bump_tap::BumpTap, publish::Publish, Runner, ARTIFACT_LINUX,
    ARTIFACT_MAC, ARTIFACT_WINDOWS, ASSET_LINUX, ASSET_MAC, ASSET_WINDOWS,
};
use anyhow::Result;
use pico_args::Arguments;

const BANNER: &str = r#"
                            __
   ___  ___ ________ ____  / /_____
  / _ \/ _ `/ __/ _ `/ _ \/ __/ __/
 / .__/\_,_/\__/\_,_/ .__/\__/_/
/_/                /_/
"#;

const XTASK_HELP: &str = r#"
cargo xtask
Run custom build command.
USAGE:
    cargo xtask <SUBCOMMAND>
SUBCOMMANDS:
    bump-choco
    bump-tap
    publish
"#;

const PUBLISH_HELP: &str = r#"
cargo xtask publish
Build release and upload to GitHub releases.
USAGE:
    cargo xtask publish
"#;

const BUMP_TAP_HELP: &str = r#"
cargo xtask bump-tap
Bump homebrew tap formula version.
USAGE:
    cargo xtask bump-tap
"#;

const BUMP_CHOCO_HELP: &str = r#"
cargo xtask bump-choco
Bump chocolatey package version.
USAGE:
    cargo xtask bump-choco
"#;

fn main() -> Result<()> {
    let mut args = Arguments::from_env();
    let subcommand = args.subcommand()?.unwrap_or_default();

    match subcommand.as_str() {
        "publish" => {
            if args.contains(["-h", "--help"]) {
                eprintln!("{}", PUBLISH_HELP);
                return Ok(());
            }

            let (artifact, asset) = match () {
                _ if cfg!(target_os = "windows") => (ARTIFACT_WINDOWS, ASSET_WINDOWS),
                _ if cfg!(target_os = "macos") => (ARTIFACT_MAC, ASSET_MAC),
                _ if cfg!(target_os = "linux") => (ARTIFACT_LINUX, ASSET_LINUX),
                _ => panic!("Unsupported publishing platform"),
            };
            let artifact = artifact.to_owned();
            let asset = asset.to_owned();

            Publish { artifact, asset }.run()
        }

        "bump-tap" => {
            if args.contains(["-h", "--help"]) {
                eprintln!("{}", BUMP_TAP_HELP);
                return Ok(());
            }

            BumpTap {}.run()
        }

        "bump-choco" => {
            if args.contains(["-h", "--help"]) {
                eprintln!("{}", BUMP_CHOCO_HELP);
                return Ok(());
            }

            BumpChoco {}.run()
        }

        _ => {
            println!("{}", BANNER);
            eprintln!("{}", XTASK_HELP);
            Ok(())
        }
    }
}
