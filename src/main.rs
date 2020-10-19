use clap::Clap;
use pacaptr::dispatch::Opt;
use pacaptr::print::{print_err, PROMPT_ERROR};

#[tokio::main]
async fn main() {
    let opt = Opt::parse();
    match opt.dispatch().await {
        Ok(0) => (),
        Ok(n) => std::process::exit(n),
        Err(e) => {
            print_err(e, PROMPT_ERROR);
            std::process::exit(1);
        }
    }
}
