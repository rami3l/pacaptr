use clap::Clap;
use pacaptr::dispatch::Opts;
use pacaptr::print::{print_err, PROMPT_ERROR};

#[tokio::main]
async fn main() {
    let opt = Opts::parse();
    match opt.dispatch().await {
        Ok(n) => std::process::exit(n),
        Err(e) => {
            print_err(e, PROMPT_ERROR);
            std::process::exit(1);
        }
    }
}
