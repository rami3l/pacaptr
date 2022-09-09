//! Output messages and prompts.

#![allow(missing_docs, clippy::module_name_repetitions)]

pub mod style {
    use console::Style;
    use once_cell::sync::Lazy;

    pub static MESSAGE: Lazy<Style> = Lazy::new(|| Style::new().green().bold());
    pub static ERROR: Lazy<Style> = Lazy::new(|| Style::new().bright().red().bold());
    pub static QUESTION: Lazy<Style> = Lazy::new(|| Style::new().yellow());
    pub static OPTIONS: Lazy<Style> = Lazy::new(|| Style::new().underlined());
}

pub mod prompt {
    use once_cell::sync::Lazy;

    use crate::print::style;

    type StyledStr<'a> = console::StyledObject<&'a str>;

    pub static CANCELED: Lazy<StyledStr> = Lazy::new(|| style::MESSAGE.apply_to("Canceled"));
    pub static PENDING: Lazy<StyledStr> = Lazy::new(|| style::MESSAGE.apply_to("Pending"));
    pub static RUNNING: Lazy<StyledStr> = Lazy::new(|| style::MESSAGE.apply_to("Running"));
    pub static INFO: Lazy<StyledStr> = Lazy::new(|| style::MESSAGE.apply_to("Info"));
    pub static ERROR: Lazy<StyledStr> = Lazy::new(|| style::ERROR.apply_to("Error"));
}

use std::fmt::{self, Display};

/// The right indentation to be applied on prompt prefixes.
static PROMPT_INDENT: usize = 9;

macro_rules! prompt_format {
    () => {
        "{:>indent$}"
    };
}

macro_rules! plain_format {
    () => {
        concat!(prompt_format!(), " {}")
    };
}

macro_rules! quoted_format {
    () => {
        concat!(prompt_format!(), " `{}`")
    };
}

macro_rules! question_format {
    () => {
        concat!(prompt_format!(), " {}? ")
    };
}

/// Writes a message after the given prompt.
pub(crate) fn write(
    f: &mut fmt::Formatter,
    prompt: impl Display,
    msg: impl Display,
) -> fmt::Result {
    write!(f, plain_format!(), prompt, msg, indent = PROMPT_INDENT)
}

/// Prints out a message after the given prompt.
pub(crate) fn println(prompt: impl Display, msg: impl Display) {
    println!(
        plain_format!(),
        style::MESSAGE.apply_to(prompt),
        msg,
        indent = PROMPT_INDENT
    );
}

/// Prints out an error message.
pub(crate) fn println_err(msg: impl Display) {
    println!(
        plain_format!(),
        &*prompt::ERROR,
        msg,
        indent = PROMPT_INDENT
    );
}

/// Prints out a backtick-quoted message after the given prompt.
pub(crate) fn println_quoted(prompt: impl Display, msg: impl Display) {
    println!(
        quoted_format!(),
        style::MESSAGE.apply_to(prompt),
        msg,
        indent = PROMPT_INDENT
    );
}

/// Prints out a question after the given prompt.
pub(crate) fn print_question(question: impl Display, options: impl Display) {
    print!(
        question_format!(),
        style::QUESTION.apply_to(question),
        style::OPTIONS.apply_to(options),
        indent = PROMPT_INDENT
    );
}
