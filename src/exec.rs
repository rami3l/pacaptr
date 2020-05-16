use crate::error::Error;
use colored::Colorize;
use std::io::{BufReader, Read, Write};
use subprocess::{Exec, Redirection};

pub static PROMPT_DRYRUN: &str = ":: Will run:";
pub static PROMPT_RUN: &str = ">>";
pub static PROMPT_INFO: &str = "::";
pub static PROMPT_ERROR: &str = ":: Error:";

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
            print_cmd(cmd, subcmd, kws, flags, PROMPT_DRYRUN);
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
        .and_then(|stream| Ok(BufReader::new(stream)))?;

    let mut out = Vec::<u8>::new();
    let mut stdout = std::io::stdout();

    for mb in stdout_reader.bytes() {
        let b = mb?;
        out.write(&[b])?;
        if !mute {
            stdout.write(&[b])?;
        }
    }

    Ok(out)
}

/// Execute a command and collect it's `stderr`.
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
        .and_then(|stream| Ok(BufReader::new(stream)))?;

    let mut out = Vec::<u8>::new();
    let mut stderr = std::io::stderr();

    for mb in stderr_reader.bytes() {
        let b = mb?;
        out.write(&[b])?;
        if !mute {
            stderr.write(&[b])?;
        }
    }

    Ok(out)
}

pub fn prompt(msg: &str, expected: &[&str]) -> String {
    loop {
        let mut answer = String::new();
        print!("{}", msg);
        let _ = std::io::stdout().flush();
        let read = std::io::stdin().read_line(&mut answer);
        if read.is_ok() {
            if let Some('\n') = answer.chars().next_back() {
                answer.pop();
            }
            if let Some('\r') = answer.chars().next_back() {
                answer.pop();
            }
            if expected.iter().find(|&&x| x == &answer).is_some() {
                break answer;
            }
        }
    }
}

fn exec_prompt(
    cmd: &str,
    subcmd: &[&str],
    kws: &[&str],
    flags: &[&str],
    mute: bool,
) -> Result<Vec<u8>, Error> {
    println!("{} `{}`", PROMPT_DRYRUN, cmd_str(cmd, subcmd, kws, flags));
    let proceed: bool = {
        let expected = vec!["", "Y", "y", "N", "n"];
        match prompt(&format!("{} Proceed? [Y/n]: ", PROMPT_INFO), &expected).as_ref() {
            "Y" | "y" | "" => true,
            "N" | "n" => false,
            _ => unreachable!(),
        }
    };
    if !proceed {
        return Ok(Vec::new());
    }
    print_cmd(cmd, subcmd, kws, flags, PROMPT_RUN);
    exec_checkerr(cmd, subcmd, kws, flags, mute)
}

/// Get the String representation of a particular command.
pub fn cmd_str(cmd: &str, subcmd: &[&str], kws: &[&str], flags: &[&str]) -> String {
    let mut res: String = cmd.into();
    for &w in subcmd.iter().chain(kws).chain(flags) {
        res.push(' ');
        res.push_str(w);
    }
    res
}

/// Print out the command after the given prompt.
pub fn print_cmd(cmd: &str, subcmd: &[&str], kws: &[&str], flags: &[&str], prompt: &str) {
    println!("{} {}", prompt, cmd_str(cmd, subcmd, kws, flags));
}

/// Print out a message after the given prompt.
pub fn print_msg(msg: &str, prompt: &str) {
    println!("{} {}", prompt, msg);
}

pub fn print_err(err: impl std::error::Error, prompt: &str) {
    eprintln!("{}", format!("{} {}", prompt, err).red());
}

/// Check if an executable exists by name (consult `$PATH`) or by path.
/// To check by one parameter only, pass `""` as another.
pub fn is_exe(name: &str, path: &str) -> bool {
    (!path.is_empty() && std::path::Path::new(path).exists())
        || (!name.is_empty() && which::which(name).is_ok())
}
