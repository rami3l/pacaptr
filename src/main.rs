use clap::Clap;
use pacaptr::dispatch::Opt;
use pacaptr::print::{print_err, PROMPT_ERROR};

#[tokio::main]
async fn main() {
    let opt = Opt::parse();
    if let Err(e) = opt.dispatch() {
        print_err(e, PROMPT_ERROR);
        std::process::exit(1);
    }
}
