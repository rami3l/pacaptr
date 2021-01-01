mod dispatch;

use crate::dispatch::{Opts, Run, SubCmd};
use clap::Clap;

use anyhow::Result;
use xshell::{cmd, read_file};

const CORE: &str = "pacaptr";
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
    match opts.subcmd {
        SubCmd::Run(run) => {
            let keywords = run.keywords;
            let _o = cmd!("cargo run -p {CORE} {keywords...}").run()?;
        }
        SubCmd::Install(install) => {
            let keywords = install.keywords;
            let _o = cmd!("cargo install {CORE} --path ./core {keywords...}").run()?;
        }
    }

    Ok(())
}
