use crate::error::Error;
use colored::Colorize;
use std::io::BufReader;
use std::io::{Read, Write};
use subprocess::{Exec, Redirection};

pub static PROMPT_DRYRUN: &str = "#>";
pub static PROMPT_RUN: &str = ">>";
pub static PROMPT_INFO: &str = "::";
pub static PROMPT_ERROR: &str = "xx";

/// Different ways in which a command shall be dealt with.
pub enum Mode {
    /// Solely print out the command that should be executed, and stop.
    DryRun,

    /// Silently collect all the output as a String. Print nothing.
    Mute,

    /// Print out the command which should be executed, run it and collect its stdout/stderr combined as a Vec<u8>.
    /// Potentially dangerous as it destroys the colored stdout. Use it only if really necessary.
    CheckAll,

    /// Print out the command which should be executed, run it and collect its stderr as a Vec<u8>.
    /// This will not break the colored stdout.
    CheckErr,
}

pub fn exec(cmd: &str, subcmd: &[&str], kws: &[&str], mode: Mode) -> Result<Vec<u8>, Error> {
    match mode {
        Mode::DryRun => {
            print_cmd(cmd, subcmd, kws, PROMPT_DRYRUN);
            Ok(Vec::new())
        }
        Mode::Mute => exec_checkall(cmd, subcmd, kws, true),
        Mode::CheckAll => {
            print_cmd(cmd, subcmd, kws, PROMPT_RUN);
            exec_checkall(cmd, subcmd, kws, false)
        }
        Mode::CheckErr => {
            print_cmd(cmd, subcmd, kws, PROMPT_RUN);
            exec_checkerr(cmd, subcmd, kws, false)
        }
    }
}

/// Execute a command and return its stdout and stderr collected in a `Vec<u8>`.
/// The command is provided in `command-subcommand-keywords` form (for example, `brew-[install]-[curl fish]`).
/// If there is no subcommand, just pass `&[]`.
/// If `mute` is `false`, then its normal `stdout/stderr` will be printed in the console too.
fn exec_checkall(cmd: &str, subcmd: &[&str], kws: &[&str], mute: bool) -> Result<Vec<u8>, Error> {
    let stdout_reader = Exec::cmd(cmd)
        .args(subcmd)
        .args(kws)
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

/// Execute a command and collect it's stderr in a `Vec<u8>`.
/// The command is provided in `command-subcommand-keywords` form (for example, `brew-[install]-[curl fish]`).
/// If there is no subcommand, just pass `&[]`.
/// If `mute` is `false`, then its normal `stderr` will be printed in the console too.
fn exec_checkerr(cmd: &str, subcmd: &[&str], kws: &[&str], mute: bool) -> Result<Vec<u8>, Error> {
    let stdout_reader = Exec::cmd(cmd)
        .args(subcmd)
        .args(kws)
        .stream_stderr()
        .map_err(|_| Error::from("Could not capture stderr"))
        .and_then(|stream| Ok(BufReader::new(stream)))?;

    let mut out = Vec::<u8>::new();
    let mut stderr = std::io::stderr();

    for mb in stdout_reader.bytes() {
        let b = mb?;
        out.write(&[b])?;
        if !mute {
            stderr.write(&[b])?;
        }
    }

    Ok(out)
}

/// Print out the command after the given prompt.
pub fn print_cmd(cmd: &str, subcmd: &[&str], kws: &[&str], prompt: &str) {
    let mut cmd_str: String = cmd.into();
    for &w in subcmd.iter().chain(kws) {
        cmd_str.push(' ');
        cmd_str.push_str(w);
    }
    println!("{} {}", prompt, cmd_str);
}

/// Print out a message after the given prompt.
pub fn print_msg(msg: &str, prompt: &str) {
    println!("{} {}", prompt, msg);
}

pub fn print_err(err: impl std::error::Error, prompt: &str) {
    eprintln!("{}", format!("{} {}", prompt, err).red());
}
