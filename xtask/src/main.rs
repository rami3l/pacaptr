mod dispatch;

use crate::dispatch::{bump_tap::BumpTap, publish::Publish, Runner};
use anyhow::{anyhow, Result};
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
    publish
    bump-tap
"#;

const PUBLISH_HELP: &str = r#"
cargo xtask publish
Build release and upload to GitHub releases.
USAGE:
    cargo xtask publish [FLAGS]
FLAGS:
            --artifact=ARTIFACT Name of the executable.
            --asset=ASSET       Name of the asset.
"#;

const BUMP_TAP_HELP: &str = r#"
cargo xtask bump-tap
Bump homebrew tap formula version.
USAGE:
    cargo xtask bump-tap
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

            let artifact = args
                .opt_value_from_str("--artifact")?
                .ok_or_else(|| anyhow!("--artifact must be assigned."))?;
            let asset = args
                .opt_value_from_str("--asset")?
                .ok_or_else(|| anyhow!("--asset must be assigned."))?;

            Publish { artifact, asset }.run()
        }

        "bump-tap" => {
            if args.contains(["-h", "--help"]) {
                eprintln!("{}", BUMP_TAP_HELP);
                return Ok(());
            }

            BumpTap {}.run()
        }

        _ => {
            println!("{}", BANNER);
            eprintln!("{}", XTASK_HELP);
            Ok(())
        }
    }
}
