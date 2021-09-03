use clap::Clap;
use pacaptr::{
    dispatch::Pacaptr,
    error::Error,
    print::{print_err, PROMPT_ERROR},
};
use tap::prelude::*;

#[tokio::main]
async fn main() {
    let res = Pacaptr::parse()
        .dispatch()
        .await
        .tap_err(|e| print_err(e, PROMPT_ERROR));
    let code = match res {
        Ok(_) => 0,
        Err(Error::CmdStatusCodeError { code, .. }) => code,
        Err(_) => 1,
    };
    std::process::exit(code)
}
