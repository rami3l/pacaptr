use pacaptr::dispatch::Opt;
use pacaptr::exec::{print_err, PROMPT_ERROR};
use structopt::StructOpt;

fn main() {
    let opt = Opt::from_args();
    if let Err(e) = opt.dispatch() {
        print_err(e, PROMPT_ERROR)
    }
}
