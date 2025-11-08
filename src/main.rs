mod cmd;

#[allow(clippy::pedantic)]
mod _built {
    include!(concat!(env!("OUT_DIR"), "/built.rs"));
}

use clap::Parser;
use pacaptr::error::MainError;

use crate::cmd::Pacaptr;

#[tokio::main]
async fn main() -> Result<(), MainError> {
    Pacaptr::parse().dispatch().await?;
    Ok(())
}
