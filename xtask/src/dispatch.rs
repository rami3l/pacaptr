use clap::Clap;

/// The command line options to be collected.
#[derive(Debug, Clap)]
#[clap(
    about = clap::crate_description!(),
    version = clap::crate_version!(),
    author = clap::crate_authors!(),
    setting = clap::AppSettings::ColoredHelp,
    setting = clap::AppSettings::ArgRequiredElseHelp,
)]
pub struct Opts {
    /*
    /// Sets a custom config file. Could have been an Option<T> with no default too
    #[clap(short, long, default_value = "default.conf")]
    config: String,
    /// Some input. Because this isn't an Option<T> it's required to be used
    input: String,
    /// A level of verbosity, and can be used multiple times
    #[clap(short, long, parse(from_occurrences))]
    verbose: i32,
    */
    #[clap(subcommand)]
    pub subcmd: SubCmd,
}

#[derive(Debug, Clap)]
pub enum SubCmd {
    Run(Run),
    Install(Install),
}

#[derive(Debug, Clap)]
#[clap(about = "Delegate to core's `cargo run`")]
pub struct Run {
    #[clap(name = "KEYWORDS", about = "The rest of the command")]
    pub keywords: Vec<String>,
}

#[derive(Debug, Clap)]
#[clap(about = "Delegate to core's `cargo install`")]
pub struct Install {
    #[clap(name = "KEYWORDS", about = "The rest of the command")]
    pub keywords: Vec<String>,
}
