mod dispatch;

use crate::dispatch::{Opts, Runner};
use anyhow::Result;
use clap::Clap;

/*
const BANNER: &str = r#"
                            __
   ___  ___ ________ ____  / /_____
  / _ \/ _ `/ __/ _ `/ _ \/ __/ __/
 / .__/\_,_/\__/\_,_/ .__/\__/_/
/_/                /_/
"#;
*/

fn main() -> Result<()> {
    use dispatch::Opts::*;
    match Opts::parse() {
        Publish(x) => x.run()?,
        BumpTap(x) => x.run()?,
    }
    Ok(())
}
