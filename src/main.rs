use clap::Clap;
use pacaptr::{
    dispatch::Opts,
    print::{print_err, PROMPT_ERROR},
};

#[tokio::main]
async fn main() {
    let code = Opts::parse().dispatch().await.unwrap_or_else(|e| {
        print_err(e, PROMPT_ERROR);
        1
    });
    std::process::exit(code)
}
