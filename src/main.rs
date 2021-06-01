use clap::Clap;
use pacaptr::{
    dispatch::Pacaptr,
    print::{print_err, PROMPT_ERROR},
};
use tap::prelude::*;

#[tokio::main]
async fn main() {
    let code = Pacaptr::parse()
        .dispatch()
        .await
        .tap_err(|e| print_err(e, PROMPT_ERROR))
        .unwrap_or(1);
    std::process::exit(code)
}
