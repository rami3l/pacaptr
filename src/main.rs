use clap::Clap;
use pacaptr::{
    dispatch::Pacaptr,
    error::Error,
    print::{print_err, PROMPT_ERROR},
};

#[tokio::main]
async fn main() {
    let res = Pacaptr::parse().dispatch().await;
    // TODO: Replace this with `Termination`. Currently blocked by https://github.com/rust-lang/rust/issues/43301.
    if let Err(e) = &res {
        print_err(e, PROMPT_ERROR);
        std::process::exit(match e {
            Error::CmdStatusCodeError { code, .. } => *code,
            _ => 1,
        })
    }
}
