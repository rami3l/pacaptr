use clap::Clap;
use pacaptr::{
    dispatch::Opts,
    print::{print_err, PROMPT_ERROR},
};
use tap::prelude::*;

#[tokio::main]
async fn main() {
    let code = Opts::parse()
        .dispatch()
        .await
        .tap_err(|e| print_err(e, PROMPT_ERROR))
        .unwrap_or(1);
    std::process::exit(code)
}
