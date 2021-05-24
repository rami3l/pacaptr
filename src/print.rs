//! This module handles output format.

use crate::exec::Cmd;
use colored::Colorize;

pub static PROMPT_CANCELED: &str = "Canceled";
pub static PROMPT_PENDING: &str = "Pending";
pub static PROMPT_RUN: &str = "Running";
pub static PROMPT_INFO: &str = "Info";
pub static PROMPT_ERROR: &str = "Error";

pub static PROMPT_INDENT: usize = 9;

macro_rules! prompt_format {
    () => {
        "{:>indent$}"
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

/// Prints out the command after the given prompt.
pub fn print_cmd(cmd: &Cmd, prompt: &str) {
    println!(
        cmd_format!(),
        prompt.green().bold(),
        cmd,
        indent = PROMPT_INDENT
    )
}

/// Prints out a message after the given prompt.
pub fn print_msg(msg: &str, prompt: &str) {
    println!(
        msg_format!(),
        prompt.green().bold(),
        msg,
        indent = PROMPT_INDENT
    );
}

/// Prints out an error after the given prompt.
pub fn print_err(err: impl std::fmt::Display, prompt: &str) {
    eprintln!(
        msg_format!(),
        prompt.bright_red().bold(),
        format!("{:#}", err),
        indent = PROMPT_INDENT
    );
}

/// Prints out a question after the given prompt.
pub fn print_question(question: &str, options: &str) {
    print!(
        question_format!(),
        question.yellow(),
        options.underline(),
        indent = PROMPT_INDENT
    );
}

/// Makes the first `char` of a string uppercase.
///
/// # Examples
/// ```rust
/// use pacaptr::print::uppercase_first_char;
/// assert_eq!(
///     uppercase_first_char("hello"),
///     "Hello",
/// );
/// ```
pub fn uppercase_first_char(s: &str) -> String {
    let mut chars = s.chars();
    chars.next().map_or_else(
        || "".into(),
        |first| first.to_uppercase().to_string() + chars.as_str(),
    )
}
