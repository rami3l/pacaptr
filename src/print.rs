//! Output messages and prompts.

#![allow(missing_docs, clippy::module_name_repetitions)]

pub(crate) mod style {
    use std::sync::LazyLock;

    use console::Style;

    pub static MESSAGE: LazyLock<Style> = LazyLock::new(|| Style::new().green().bold());
    pub static ERROR: LazyLock<Style> = LazyLock::new(|| Style::new().bright().red().bold());
    pub static QUESTION: LazyLock<Style> = LazyLock::new(|| Style::new().yellow().bold());
}

pub mod prompt {
    use std::sync::LazyLock;

    use crate::print::style;

    type StyledStr<'a> = console::StyledObject<&'a str>;

    pub static CANCELED: LazyLock<StyledStr> =
        LazyLock::new(|| style::MESSAGE.apply_to("Canceled"));
    pub static PENDING: LazyLock<StyledStr> = LazyLock::new(|| style::MESSAGE.apply_to("Pending"));
    pub static RUNNING: LazyLock<StyledStr> = LazyLock::new(|| style::MESSAGE.apply_to("Running"));
    pub static INFO: LazyLock<StyledStr> = LazyLock::new(|| style::MESSAGE.apply_to("Info"));
    pub static ERROR: LazyLock<StyledStr> = LazyLock::new(|| style::ERROR.apply_to("Error"));
}

use std::fmt::{self, Debug, Display};

use console::{style, Style};
use dialoguer::theme::ColorfulTheme;

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

/// Writes an error after the given prompt.
#[allow(clippy::missing_errors_doc)]
pub fn write_err(f: &mut fmt::Formatter, prompt: impl Display, err: impl Debug) -> fmt::Result {
    write!(
        f,
        concat!(prompt_format!(), " {:?}"),
        prompt,
        err,
        indent = PROMPT_INDENT,
    )
}

/// Prints out a message after the given prompt.
pub fn println(prompt: impl Display, msg: impl Display) {
    println!(
        plain_format!(),
        style::MESSAGE.apply_to(prompt),
        msg,
        indent = PROMPT_INDENT,
    );
}

/// Prints out an error message.
pub fn println_err(msg: impl Display) {
    println!(
        plain_format!(),
        &*prompt::ERROR,
        msg,
        indent = PROMPT_INDENT,
    );
}

/// Prints out a backtick-quoted message after the given prompt.
pub fn println_quoted(prompt: impl Display, msg: impl Display) {
    println!(
        quoted_format!(),
        style::MESSAGE.apply_to(prompt),
        msg,
        indent = PROMPT_INDENT,
    );
}

/// Returns a [`dialoguer`] theme with the given prompt.
pub(crate) fn question_theme(prompt: impl Display) -> impl dialoguer::theme::Theme {
    let prompt_prefix = style::QUESTION.apply_to(format!(
        prompt_format!(),
        style::QUESTION.apply_to(prompt),
        indent = PROMPT_INDENT,
    ));
    ColorfulTheme {
        success_prefix: prompt_prefix.clone(),
        error_prefix: prompt_prefix.clone().red(),
        prompt_prefix,
        prompt_style: Style::new(),
        prompt_suffix: style(String::new()),
        active_item_prefix: style("  *".into()).bold().for_stderr(),
        active_item_style: Style::new().bold(),
        inactive_item_prefix: style("   ".into()).for_stderr(),
        ..ColorfulTheme::default()
    }
}
