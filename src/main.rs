mod cmd;

#[allow(clippy::pedantic)]
mod _built {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use clap::Parser;
use macro_rules_attribute::apply;
use pacaptr::error::MainError;
use smol_macros::main;

use crate::cmd::Pacaptr;

#[apply(main!)]
async fn main() -> Result<(), MainError> {
    Pacaptr::parse().dispatch().await?;
    Ok(())
}
