mod dispatch;
mod error;
mod exec;
mod packmanager;

use dispatch::Opt;
use structopt::StructOpt;

#[macro_use]
extern crate lazy_static;

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);
}
