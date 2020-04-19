use crate::error::Error;
use std::io::BufReader;
use std::io::{Read, Write};
use subprocess::{Exec, Redirection};

pub static PROMPT_DRYRUN: &str = "#>";
pub static PROMPT_RUN: &str = ">>";
pub static PROMPT_INFO: &str = "::";

/// Different ways in which a command shall be dealt with.
pub enum Mode {
    /// Solely print out the command that should be executed, and stop.
    DryRun,

    /// Silently collect all the output. Print nothing.
    Mute,

    // Print out the command that should be executed, run it and collect the output.
    Verbose,
}

pub fn exec(cmd: &str, subcmd: &[&str], kws: &[&str], mode: Mode) -> Result<Vec<u8>, Error> {
    match mode {
        Mode::DryRun => {
            print(cmd, subcmd, kws, PROMPT_DRYRUN);
            Ok(Vec::new())
        }
        Mode::Mute => exec_impl(cmd, subcmd, kws, true),
        Mode::Verbose => {
            print(cmd, subcmd, kws, PROMPT_RUN);
            exec_impl(cmd, subcmd, kws, false)
        }
    }
}

/// Execute a command and return its stdout and stderr collected in a `Vec<u8>`.
/// The command is provided in `command-subcommand-keywords` form.
/// For example, `brew-[install]-[curl fish]`. If there is no subcommand, just pass &[].
/// If `mute` is `false`, then its normal `stdout/stderr` will be printed in the console too.
fn exec_impl(cmd: &str, subcmd: &[&str], kws: &[&str], mute: bool) -> Result<Vec<u8>, Error> {
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
        let b = mb.unwrap();
        out.write(&[b])?;
        if !mute {
            stdout.write(&[b])?;
        }
    }

    Ok(out)
}

/// Print the command after a given prompt.
pub fn print(cmd: &str, subcmd: &[&str], kws: &[&str], prompt: &str) {
    let mut cmd_str: String = cmd.into();
    for &w in subcmd.iter().chain(kws) {
        cmd_str.push(' ');
        cmd_str.push_str(w);
    }
    println!("{} {}", prompt, cmd_str);
}
