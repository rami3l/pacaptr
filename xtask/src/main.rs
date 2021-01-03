mod dispatch;

use crate::dispatch::{Opts, Runner, SubCmd};
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
    /*
    let name = "Julia";
    let output = cmd!("echo hello {name}!").read()?;
    assert_eq!(output, "hello Julia!");

    let err = read_file("feeling-lucky.txt").unwrap_err();
    assert_eq!(
        err.to_string(),
        "`feeling-lucky.txt`: no such file or directory (os error 2)",
    );
    */

    let opts = Opts::parse();

    use SubCmd::*;
    match opts.subcmd {
        Run(x) => x.run()?,
        Install(x) => x.run()?,
        Publish(x) => x.run()?,
    }

    Ok(())
}
