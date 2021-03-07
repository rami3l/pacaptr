//! This module handles launching of subprocesses and their results.

use crate::error::{Error, Result};
use crate::print::*;
use bytes::Bytes;
use futures::{Stream, StreamExt, TryStreamExt};
pub use is_root::is_root;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::io::{AsyncRead, AsyncWriteExt};
use tokio::process::Command as Exec;
use tokio::task::JoinHandle;
use tokio::try_join;
use tokio_util::codec::{BytesCodec, FramedRead};
use which::which;

/// Different ways in which a command shall be dealt with.
#[derive(Copy, Clone, Debug)]
pub enum Mode {
    /// Solely print out the command that should be executed, and stop.
    PrintCmd,

    /// Silently collect all the `stdout`/`stderr` combined. Print nothing.
    Mute,

    /// Print out the command which should be executed, run it and collect its `stdout`/`stderr` combined.
    /// Potentially dangerous as it destroys the colored `stdout`. Use it only if really necessary.
    CheckAll,

    /// Print out the command which should be executed, run it and collect its `stderr`.
    /// This will work with a colored `stdout`.
    CheckErr,

    /// A CUSTOM prompt implemented by a `pacaptr` module itself.
    /// Like [`CheckErr`], but will ask for confirmation before proceeding.
    Prompt,
}

pub type StatusCode = i32;

/// Output of running a [`Cmd`].
#[derive(Debug, Clone)]
pub struct Output {
    /// The captured `stdout`,
    /// and if set to [`Mode::CheckAll`], mixed with captured `stderr`.
    pub contents: Vec<u8>,

    /// The status code returned by the [`Cmd`].
    ///
    /// Here we use [`Some(n)`] for exit code, [`None`] for signals.
    pub code: Option<StatusCode>,
}

impl Default for Output {
    fn default() -> Self {
        Output {
            contents: Default::default(),
            code: Some(0),
        }
    }
}

/// A command to be executed, provided in `command-flags-keywords` form
/// (eg. `[brew install]-[--dry-run]-[curl fish]`).
#[derive(Debug, Clone, Default)]
pub struct Cmd {
    /// Flag indicating If a **normal admin** needs to run this command with `sudo`.
    pub sudo: bool,
    pub cmd: Vec<String>,
    pub kws: Vec<String>,
    pub flags: Vec<String>,
}

impl Cmd {
    pub fn new<S: AsRef<str>>(cmd: &[S]) -> Self {
        Self {
            cmd: cmd.iter().map(|s| s.as_ref().into()).collect(),
            ..Default::default()
        }
    }

    pub fn new_sudo<S: AsRef<str>>(cmd: &[S]) -> Self {
        Self::new(cmd).sudo(true)
    }

    pub fn kws<S: AsRef<str>>(mut self, kws: &[S]) -> Self {
        self.kws = kws.iter().map(|s| s.as_ref().into()).collect();
        self
    }

    pub fn flags<S: AsRef<str>>(mut self, flags: &[S]) -> Self {
        self.flags = flags.iter().map(|s| s.as_ref().into()).collect();
        self
    }

    pub fn sudo(mut self, sudo: bool) -> Self {
        self.sudo = sudo;
        self
    }

    /// Determine if this command actually needs to run with `sudo -S`.
    ///
    /// If a **normal admin** needs to run it with `sudo`, and we are not `root`, then this is the case.
    pub fn needs_sudo(&self) -> bool {
        self.sudo && !is_root()
    }

    /// Convert a [`Cmd`] object into an [`Exec`].
    pub fn build(self) -> Exec {
        // ! Special fix for `zypper`: `zypper install -y curl` is accepted,
        // ! but not `zypper install curl -y`.
        // ! So we place the flags first, and then keywords.
        if self.needs_sudo() {
            let mut builder = Exec::new("sudo");
            builder
                .arg("-S")
                .args(&self.cmd)
                .args(&self.flags)
                .args(&self.kws);
            builder
        } else {
            let (cmd, subcmd) = self
                .cmd
                .split_first()
                .expect("Failed to build Cmd, command is empty");
            let mut builder = Exec::new(cmd);
            builder.args(subcmd).args(&self.flags).args(&self.kws);
            builder
        }
    }
}

/// Helper to implement [`Cmd::exec_checkerr`] and [`Cmd::exec_checkall`].
///
/// Take contents from an input stream and copy to an output stream (optional) and a [`Vec<u8>`],
/// then return the [`Vec<u8>`].
///
/// # Arguments
///
/// * `src` - The input stream to read from.
/// * `out` - The output stream to write to (only if `mute` is [`true`]).
/// * `mute` - Whether to mute `out`.
async fn exec_tee<S, O>(src: &mut S, out: &mut O, mute: bool) -> Result<Vec<u8>>
where
    S: Stream<Item = std::result::Result<Bytes, std::io::Error>> + Unpin,
    O: AsyncWriteExt + Unpin,
{
    let mut out1 = Vec::<u8>::new();
    while let Some(mb) = src.next().await {
        let b = mb?;
        let b = b.as_ref();
        if mute {
            out1.write_all(b).await?;
        } else {
            try_join!(out.write_all(b), out1.write_all(b))?;
        }
    }
    Ok(out1)
}

impl Cmd {
    /// Execute a [`Cmd`] and return its output.
    ///
    /// The exact behavior depends on the [`Mode`] passed in (see [`exec::Mode`] for more info).
    pub async fn exec(self, mode: Mode) -> Result<Output> {
        match mode {
            Mode::PrintCmd => {
                print_cmd(&self, PROMPT_CANCELED);
                Ok(Default::default())
            }
            Mode::Mute => self.exec_checkall(true).await,
            Mode::CheckAll => {
                print_cmd(&self, PROMPT_RUN);
                self.exec_checkall(false).await
            }
            Mode::CheckErr => {
                print_cmd(&self, PROMPT_RUN);
                self.exec_checkerr(false).await
            }
            Mode::Prompt => self.exec_prompt(false).await,
        }
    }

    /// Execute a [`Cmd`] and return its `stdout` and `stderr`.
    ///
    /// If `mute` is `false`, then its normal `stdout/stderr` will be printed in the console too.
    async fn exec_checkall(self, mute: bool) -> Result<Output> {
        use Error::*;

        let mut child = self
            .build()
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(CmdSpawnError)?;
        let stdout_reader =
            child
                .stdout
                .take()
                .map(into_bytes)
                .ok_or_else(|| CmdNoHandleError {
                    handle: "stdout".into(),
                })?;
        let stderr_reader =
            child
                .stderr
                .take()
                .map(into_bytes)
                .ok_or_else(|| CmdNoHandleError {
                    handle: "stderr".into(),
                })?;
        let mut merged_reader = tokio_stream::StreamExt::merge(stdout_reader, stderr_reader);

        let code: JoinHandle<Result<Option<i32>>> = tokio::spawn(async move {
            let status = child.wait().await.map_err(CmdWaitError)?;
            Ok(status.code())
        });

        let mut stdout = tokio::io::stdout();
        let contents = exec_tee(&mut merged_reader, &mut stdout, mute).await?;

        Ok(Output {
            contents,
            code: code.await.map_err(CmdJoinError)??,
        })
    }

    /// Execute a [`Cmd`] and collect its `stderr`.  
    /// If `mute` is `false`, then its normal `stderr` will be printed in the console too.
    async fn exec_checkerr(self, mute: bool) -> Result<Output> {
        use Error::*;

        let mut child = self
            .build()
            .stderr(Stdio::piped())
            .spawn()
            .map_err(CmdSpawnError)?;
        let mut stderr_reader =
            child
                .stderr
                .take()
                .map(into_bytes)
                .ok_or_else(|| CmdNoHandleError {
                    handle: "stderr".into(),
                })?;

        let code: JoinHandle<Result<Option<i32>>> = tokio::spawn(async move {
            let status = child.wait().await.map_err(CmdWaitError)?;
            Ok(status.code())
        });

        let mut stderr = tokio::io::stderr();
        let contents = exec_tee(&mut stderr_reader, &mut stderr, mute).await?;

        Ok(Output {
            contents,
            code: code.await.map_err(CmdJoinError)??,
        })
    }

    /// Execute a [`Cmd`] and collect its `stderr`.
    /// If `mute` is `false`, then its normal `stderr` will be printed in the console too.
    ///
    /// This function behaves just like [`exec_checkerr`], but in addition,
    /// the user will be prompted if (s)he wishes to continue with the command execution.
    async fn exec_prompt(self, mute: bool) -> Result<Output> {
        lazy_static! {
            static ref ALL_YES: AtomicBool = AtomicBool::new(false);
        }

        let proceed: bool = if ALL_YES.load(Ordering::SeqCst) {
            true
        } else {
            print_cmd(&self, PROMPT_PENDING);
            match tokio::task::block_in_place(move || {
                prompt(
                    "Proceed",
                    "[YES/All/No/^C]",
                    &["", "y", "yes", "a", "all", "n", "no"],
                    false,
                )
                .to_lowercase()
            })
            .as_ref()
            {
                // The default answer is `Yes`
                "y" | "yes" | "" => true,
                // You can also say `All` to answer `Yes` to all the other questions that follow.
                "a" | "all" => {
                    ALL_YES.store(true, Ordering::SeqCst);
                    true
                }
                // Or you can say `No`.
                "n" | "no" => false,
                // ! I didn't put a `None` option because you can just Ctrl-C it if you want.
                _ => unreachable!(),
            }
        };
        if !proceed {
            return Ok(Default::default());
        }
        print_cmd(&self, PROMPT_RUN);
        self.exec_checkerr(mute).await
    }
}

impl std::fmt::Display for Cmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sudo_prefix: &str = if self.needs_sudo() { "sudo -S " } else { "" };
        let mut res = sudo_prefix.to_owned();
        let cmd_str = self
            .cmd
            .iter()
            .chain(&self.flags)
            .chain(&self.kws)
            .map(|s| s.as_ref())
            .collect::<Vec<&str>>()
            .join(" ");
        res.push_str(&cmd_str);
        write!(f, "{}", res)
    }
}

/// Prompt and get the output string.
/// This action won't end until an expected answer is found.
///
/// If `case_sensitive` is `false`, then `expected` should be all lower case patterns.
pub fn prompt(question: &str, options: &str, expected: &[&str], case_sensitive: bool) -> String {
    use std::io::Write;
    loop {
        let mut answer = String::new();
        print_question(question, options);
        let _ = std::io::stdout().flush();
        std::io::stdin()
            .read_line(&mut answer)
            .expect("Error while reading user input");
        if !case_sensitive {
            answer = answer.to_lowercase();
        }
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

/// Find all lines in the given `text` that matches all the `patterns`.
///
/// We suppose that all patterns are legal regular expressions.
/// An error message will be printed if this is not the case.
/// If there is no legal patterns, the output [`Vec`] will be empty.
pub fn grep<'a>(text: &'a str, patterns: &[&str]) -> Result<Vec<&'a str>> {
    let rs: Vec<Regex> = patterns
        .iter()
        .map(|&pat| {
            Regex::new(pat)
                .map_err(|_e| Error::OtherError(format!("Pattern `{}` is ill-formed.", pat)))
        })
        .try_collect()?;

    let res = if !rs.is_empty() {
        text.lines()
            .filter(|line| rs.iter().all(|regex| regex.is_match(line)))
            .collect()
    } else {
        vec![]
    };
    Ok(res)
}

/// Print the result of [`grep`] line by line.
pub fn grep_print(text: &str, patterns: &[&str]) -> Result<()> {
    grep(text, patterns)?
        .iter()
        .for_each(|ln| println!("{}", ln));
    Ok(())
}

/// Check if an executable exists by name (consult `$PATH`) or by path.
///
/// To check by one parameter only, pass `""` to the other one.
pub fn is_exe(name: &str, path: &str) -> bool {
    (!path.is_empty() && which(path).is_ok()) || (!name.is_empty() && which(name).is_ok())
}

/// Helper function to turn an [`AsyncRead`] to a [`Stream`].
///
/// *Shamelessly copied from [StackOverflow](https://stackoverflow.com/a/59327560).*
pub fn into_bytes(reader: impl AsyncRead) -> impl Stream<Item = tokio::io::Result<Bytes>> {
    FramedRead::new(reader, BytesCodec::new()).map_ok(|bytes| bytes.freeze())
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[test]
    async fn simple_run() {
        println!("Starting!");
        let cmd = Cmd::new(&["bash", "-c"])
            .kws(&[r#"printf "Hello\n"; sleep 3; printf "World\n"; sleep 3; printf "!\n""#]);
        let res = cmd.exec_checkall(false).await.unwrap();
        dbg!(res);
    }
}
*/
