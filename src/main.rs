use clap::Parser;
use pacaptr::{dispatch::Pacaptr, error::MainError};

#[tokio::main]
async fn main() -> Result<(), MainError> {
    Pacaptr::parse().dispatch().await?;
    Ok(())
}
