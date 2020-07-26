use colored::Colorize;

pub static PROMPT_DRYRUN: &str = "Pending";
pub static PROMPT_RUN: &str = "Running";
pub static PROMPT_INFO: &str = "Info";
pub static PROMPT_ERROR: &str = "Error";

macro_rules! prompt_format {
    () => {
        "{:>9}"
    };
}

macro_rules! cmd_format {
    () => {
        concat!(prompt_format!(), " `{}`")
    };
}

macro_rules! msg_format {
    () => {
        concat!(prompt_format!(), " {}")
    };
}

macro_rules! question_format {
    () => {
        concat!(prompt_format!(), " {}? ")
    };
}

/// Print out the command after the given prompt.
pub fn print_cmd(cmd: &str, subcmd: &[&str], kws: &[&str], flags: &[&str], prompt: &str) {
    println!(
        cmd_format!(),
        prompt.green().bold(),
        cmd_str(cmd, subcmd, kws, flags)
    );
}

/// Print out the command after the given prompt (dry run version).
pub fn print_dryrun(cmd: &str, subcmd: &[&str], kws: &[&str], flags: &[&str], prompt: &str) {
    println!(
        cmd_format!(),
        prompt.green().bold(),
        cmd_str(cmd, subcmd, kws, flags)
    );
}

/// Print out a message after the given prompt.
pub fn print_msg(msg: &str, prompt: &str) {
    println!(msg_format!(), prompt.green().bold(), msg);
}

/// Print out an error after the given prompt.
pub fn print_err(err: impl std::error::Error, prompt: &str) {
    eprintln!(msg_format!(), prompt.bright_red().bold(), err);
}

/// Print out a question after the given prompt.
pub fn print_question(question: &str, options: &str) {
    print!(question_format!(), question.yellow(), options.underline());
}

/// Get the String representation of a particular command.
pub fn cmd_str(cmd: &str, subcmd: &[&str], kws: &[&str], flags: &[&str]) -> String {
    [cmd]
        .iter()
        .chain(subcmd)
        .chain(kws)
        .chain(flags)
        .cloned()
        .collect::<Vec<&str>>()
        .join(" ")
}
