use clap::Clap;
use pacaptr::dispatch::Opt;
use pacaptr::exec::{print_error, PROMPT_ERROR};

fn main() {
    let opt = Opt::parse();
    if let Err(e) = opt.dispatch() {
        print_error(e, PROMPT_ERROR);
        std::process::exit(1);
    }
}
