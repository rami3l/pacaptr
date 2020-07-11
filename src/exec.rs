use crate::error::Error;
use colored::Colorize;
use regex::Regex;
use std::io::{BufReader, Read, Write};
use std::sync::Mutex;
use subprocess::{Exec, Redirection};

pub static PROMPT_DRYRUN: &str = "Pending";
pub static PROMPT_RUN: &str = "Running";
pub static PROMPT_INFO: &str = "Info";
pub static PROMPT_ERROR: &str = "Error";

/// Different ways in which a command shall be dealt with.
pub enum Mode {
    /// Solely print out the command that should be executed, and stop.
    DryRun,

    /// Silently collect all the `stdout`/`stderr` combined. Print nothing.
    Mute,

    /// Print out the command which should be executed, run it and collect its `stdout`/`stderr` combined.
    /// Potentially dangerous as it destroys the colored `stdout`. Use it only if really necessary.
    CheckAll,

    /// Print out the command which should be executed, run it and collect its `stderr`.
    /// This will work with a colored `stdout`.
    CheckErr,

    /// Like `CheckErr`, but will ask for confirmation before proceeding.
    Prompt,
}

/// Execute a command and return a `Result<Vec<u8>, _>`.
/// The exact behavior depends on the `mode` passed in. See `exec::Mode`'s documentation for more info.
/// The command is provided in `command-subcommand-keywords` form (for example, `brew-[install]-[curl fish]`).
/// If there is no subcommand, just pass `&[]`.
pub fn exec(
    cmd: &str,
    subcmd: &[&str],
    kws: &[&str],
    flags: &[&str],
    mode: Mode,
) -> Result<Vec<u8>, Error> {
    match mode {
        Mode::DryRun => {
            print_dryrun(cmd, subcmd, kws, flags, PROMPT_DRYRUN);
            Ok(Vec::new())
        }
        Mode::Mute => exec_checkall(cmd, subcmd, kws, flags, true),
        Mode::CheckAll => {
            print_cmd(cmd, subcmd, kws, flags, PROMPT_RUN);
            exec_checkall(cmd, subcmd, kws, flags, false)
        }
        Mode::CheckErr => {
            print_cmd(cmd, subcmd, kws, flags, PROMPT_RUN);
            exec_checkerr(cmd, subcmd, kws, flags, false)
        }
        Mode::Prompt => exec_prompt(cmd, subcmd, kws, flags, false),
    }
}

/// Execute a command and return its `stdout` and `stderr`.
/// The command is provided in `command-subcommand-keywords` form (for example, `brew-[install]-[curl fish]`).
/// If there is no subcommand, just pass `&[]`.
/// If `mute` is `false`, then its normal `stdout/stderr` will be printed in the console too.
fn exec_checkall(
    cmd: &str,
    subcmd: &[&str],
    kws: &[&str],
    flags: &[&str],
    mute: bool,
) -> Result<Vec<u8>, Error> {
    let stdout_reader = Exec::cmd(cmd)
        .args(subcmd)
        .args(kws)
        .args(flags)
        .stderr(Redirection::Merge)
        .stream_stdout()
        .map_err(|_| Error::from("Could not capture stdout"))
        .map(BufReader::new)?;

    let mut out = Vec::<u8>::new();
    let mut stdout = std::io::stdout();

    for mb in stdout_reader.bytes() {
        let b = mb?;
        out.write_all(&[b])?;
        if !mute {
            stdout.write_all(&[b])?;
        }
    }

    Ok(out)
}

/// Execute a command and collect its `stderr`.
/// The command is provided in `command-subcommand-keywords` form (for example, `brew-[install]-[curl fish]`).
/// If there is no subcommand, just pass `&[]`.
/// If `mute` is `false`, then its normal `stderr` will be printed in the console too.
fn exec_checkerr(
    cmd: &str,
    subcmd: &[&str],
    kws: &[&str],
    flags: &[&str],
    mute: bool,
) -> Result<Vec<u8>, Error> {
    let stderr_reader = Exec::cmd(cmd)
        .args(subcmd)
        .args(kws)
        .args(flags)
        .stream_stderr()
        .map_err(|_| Error::from("Could not capture stderr"))
        .map(BufReader::new)?;

    let mut out = Vec::<u8>::new();
    let mut stderr = std::io::stderr();

    for mb in stderr_reader.bytes() {
        let b = mb?;
        out.write_all(&[b])?;
        if !mute {
            stderr.write_all(&[b])?;
        }
    }

    Ok(out)
}

/// Prompt and get the output string.
/// This action won't end until an expected answer is found.
/// If `case_sensitive == false`, then `expected` should be all lower case patterns.
pub fn prompt(question: &str, options: &str, expected: &[&str], case_sensitive: bool) -> String {
    loop {
        let mut answer = String::new();
        print_question(question, options);
        let _ = std::io::stdout().flush();
        let read = std::io::stdin().read_line(&mut answer);
        if !case_sensitive {
            answer = answer.to_lowercase();
        }
        if read.is_ok() {
            if let Some('\n') = answer.chars().next_back() {
                answer.pop();
            }
            if let Some('\r') = answer.chars().next_back() {
                answer.pop();
            }
            if expected.iter().any(|&x| x == answer) {
                break answer;
            }
        }
    }
}

/// Execute a command, collect its `stderr` and print its output to the console.
/// The command is provided in `command-subcommand-keywords` form (for example, `brew-[install]-[curl fish]`).
/// If there is no subcommand, just pass `&[]`.
/// The user will be prompted if (s)he wishes to continue with the command execution.
#[allow(clippy::mutex_atomic)]
fn exec_prompt(
    cmd: &str,
    subcmd: &[&str],
    kws: &[&str],
    flags: &[&str],
    mute: bool,
) -> Result<Vec<u8>, Error> {
    lazy_static! {
        static ref ALL_YES: Mutex<bool> = Mutex::new(false);
    }

    let mut all_yes = ALL_YES.lock().unwrap();
    let proceed: bool = if *all_yes {
        true
    } else {
        print_dryrun(cmd, subcmd, kws, flags, PROMPT_DRYRUN);
        match prompt(
            "Proceed",
            "[Yes/all/no]",
            &["", "y", "yes", "a", "all", "n", "no"],
            false,
        )
        .to_lowercase()
        .as_ref()
        {
            // The default answer is `Yes`
            "y" | "yes" | "" => true,

            // You can also say `All` to answer `Yes` to all the other questions that follow.
            "a" | "all" => {
                *all_yes = true;
                true
            }

            // Or you can say `No`.
            "n" | "no" => false,
            _ => unreachable!(),
        }
    };
    if !proceed {
        return Ok(Vec::new());
    }
    print_cmd(cmd, subcmd, kws, flags, PROMPT_RUN);
    exec_checkerr(cmd, subcmd, kws, flags, mute)
}

/// Find all lines in the given `text` that matches all the `patterns`.
pub fn grep(text: &str, patterns: &[&str]) -> Vec<String> {
    let rs: Vec<Regex> = patterns
        .iter()
        .map(|&pat| Regex::new(pat).unwrap())
        .collect();
    text.lines()
        .filter(|&line| rs.iter().all(|regex| regex.find(line).is_some()))
        .map(|s| s.to_owned())
        .collect()
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

/// Print out the command after the given prompt.
pub fn print_cmd(cmd: &str, subcmd: &[&str], kws: &[&str], flags: &[&str], prompt: &str) {
    println!(
        "{:>9} `{}`",
        prompt.green().bold(),
        cmd_str(cmd, subcmd, kws, flags)
    );
}

/// Print out the command after the given prompt (dry run version).
pub fn print_dryrun(cmd: &str, subcmd: &[&str], kws: &[&str], flags: &[&str], prompt: &str) {
    println!(
        "{:>9} `{}`",
        prompt.green().bold(),
        cmd_str(cmd, subcmd, kws, flags)
    );
}

/// Print out a message after the given prompt.
pub fn print_message(msg: &str, prompt: &str) {
    println!("{:>9} {}", prompt.green().bold(), msg);
}

/// Print out an error after the given prompt.
pub fn print_error(err: impl std::error::Error, prompt: &str) {
    eprintln!("{:>9} {}", prompt.bright_red().bold(), err);
}

/// Print out a question after the given prompt.
pub fn print_question(question: &str, options: &str) {
    print!("{:>9} {}? ", question.yellow(), options.underline());
}

/// Check if an executable exists by name (consult `$PATH`) or by path.
/// To check by one parameter only, pass `""` as another.
pub fn is_exe(name: &str, path: &str) -> bool {
    (!path.is_empty() && std::path::Path::new(path).exists())
        || (!name.is_empty() && which::which(name).is_ok())
}
