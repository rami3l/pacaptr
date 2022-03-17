use clap::Parser;
use miette::{IntoDiagnostic, Result};
use pacaptr::{
    dispatch::Pacaptr,
    error::Error,
    print::{print_err, PROMPT_ERROR},
};

#[tokio::main]
async fn main() -> Result<()> {
    let _res = Pacaptr::parse().dispatch().await.into_diagnostic()?;
    // TODO: Replace this with `Termination`. Currently blocked by https://github.com/rust-lang/rust/issues/43301.
    Ok(())
}
