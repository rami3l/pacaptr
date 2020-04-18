mod dispatch;
mod error;
mod exec;
mod packmanager;

use colored::*;
use dispatch::Opt;
use exec::PROMPT_INFO;
use structopt::StructOpt;

#[macro_use]
extern crate lazy_static;

fn main() {
    let opt = Opt::from_args();
    if let Err(e) = opt.dispatch() {
        eprintln!("{}", format!("{} Error: {}", PROMPT_INFO, e).red());
    }
}
