//! APIs for spawning subprocesses and handling their results.

use std::{
    process::Stdio,
    sync::atomic::{AtomicBool, Ordering},
};

use bytes::Bytes;
use futures::prelude::*;
pub use is_root::is_root;
use itertools::Itertools;
use once_cell::sync::Lazy;
use regex::Regex;
use tap::prelude::*;
use tokio::{
    io::{self, AsyncRead, AsyncWrite},
    process::Command as Exec,
    task::JoinHandle,
};
use tokio_util::{
    codec::{BytesCodec, FramedRead},
    compat::*,
    either::Either,
};
use which::which;

use crate::{
    error::{Error, Result},
    print::*,
};

/// Different ways in which a command shall be dealt with.
#[derive(Copy, Clone, Debug)]
pub enum Mode {
    /// Solely prints out the command that should be executed and stops.
    PrintCmd,

    /// Silently collects all the `stdout`/`stderr` combined. Prints nothing.
    Mute,

    /// Prints out the command which should be executed, runs it and collects
    /// its `stdout`/`stderr` combined.
    ///
    /// This is potentially dangerous as it destroys the colored `stdout`. Use
    /// it only if really necessary.
    CheckAll,

    /// Prints out the command which should be executed, runs it and collects
    /// its `stderr`.
    ///
    /// This will work with a colored `stdout`.
    CheckErr,

    /// A CUSTOM prompt implemented by a `pacaptr` module itself.
    ///
    /// Prints out the command which should be executed, runs it and collects
    /// its `stderr`. Also, this will ask for confirmation before
    /// proceeding.
    Prompt,
}

pub type StatusCode = i32;

/// Output of running a [`Cmd`].
#[derive(Debug, Clone)]
pub struct Output {
    /// The captured `stdout`, and if set to [`Mode::CheckAll`], mixed with
    /// captured `stderr`.
    pub contents: Vec<u8>,

    /// The status code returned by the [`Cmd`].
    ///
    /// Here we use [`Some`] for exit code, [`None`] for signals.
    pub code: Option<StatusCode>,
}

impl Default for Output {
    fn default() -> Self {
        Output {
            code: Some(0),
            contents: vec![],
        }
    }
}

/// A command to be executed, provided in `command-flags-keywords` form
/// (eg. `[brew install]-[--dry-run]-[curl fish]`).
#[derive(Debug, Clone, Default)]
pub struct Cmd {
    /// Flag indicating If a **normal admin** needs to run this command with
    /// `sudo`.
    pub sudo: bool,
    pub cmd: Vec<String>,
    pub kws: Vec<String>,
    pub flags: Vec<String>,
}

impl Cmd {
    pub fn new(cmd: &[impl AsRef<str>]) -> Self {
        Self {
            cmd: cmd.iter().map(AsRef::as_ref).map_into().collect(),
            ..Default::default()
        }
    }

    pub fn with_sudo(cmd: &[impl AsRef<str>]) -> Self {
        Self::new(cmd).sudo(true)
    }

    pub fn kws(self, kws: &[impl AsRef<str>]) -> Self {
        self.tap_mut(|s| s.kws = kws.iter().map(AsRef::as_ref).map_into().collect())
    }

    pub fn flags(self, flags: &[impl AsRef<str>]) -> Self {
        self.tap_mut(|s| s.flags = flags.iter().map(AsRef::as_ref).map_into().collect())
    }

    pub fn sudo(self, sudo: bool) -> Self {
        self.tap_mut(|s| s.sudo = sudo)
    }

    /// Determines if this command actually needs to run with `sudo -S`.
    ///
    /// If a **normal admin** needs to run it with `sudo`, and we are not
    /// `root`, then this is the case.
    pub fn should_sudo(&self) -> bool {
        self.sudo && !is_root()
    }

    /// Converts a [`Cmd`] object into an [`Exec`].
    pub fn build(self) -> Exec {
        // ! Special fix for `zypper`: `zypper install -y curl` is accepted,
        // ! but not `zypper install curl -y`.
        // ! So we place the flags first, and then keywords.
        if self.should_sudo() {
            Exec::new("sudo").tap_mut(|builder| {
                builder
                    .arg("-S")
                    .args(&self.cmd)
                    .args(&self.flags)
                    .args(&self.kws);
            })
        } else {
            let (cmd, subcmd) = self
                .cmd
                .split_first()
                .expect("Failed to build Cmd, command is empty");
            Exec::new(cmd).tap_mut(|builder| {
                builder.args(subcmd).args(&self.flags).args(&self.kws);
            })
        }
    }
}

/// Takes contents from an input stream and copy to an output stream (optional)
/// and a [`Vec<u8>`], then returns the [`Vec<u8>`].
///
/// Helper to implement [`Cmd::exec_checkerr`] and [`Cmd::exec_checkall`].
///
/// # Arguments
///
/// * `src` - The input stream to read from.
/// * `out` - The optional output stream to write to.
async fn exec_tee<S, O>(src: &mut S, out: Option<O>) -> Result<Vec<u8>>
where
    S: Stream<Item = io::Result<Bytes>> + Unpin,
    O: AsyncWrite + Unpin,
{
    let mut buf = Vec::<u8>::new();
    let buf_sink = (&mut buf).into_sink();

    let sink = if let Some(out) = out {
        let out_sink = out.compat_write().into_sink();
        buf_sink.fanout(out_sink).left_sink()
    } else {
        buf_sink.right_sink()
    };

    src.forward(sink).await?;
    Ok(buf)
}

impl Cmd {
    /// Executes a [`Cmd`] and returns its output.
    ///
    /// The exact behavior depends on the [`Mode`] passed in (see the definition
    /// of [`Mode`] for more info).
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

    /// Inner implementation of [`Cmd::exec_checkerr`] and
    /// [`Cmd::exec_checkall`].
    ///
    /// `merge == false` goes to [`Cmd::exec_checkerr`], and
    /// [`Cmd::exec_checkall`] otherwise.
    async fn exec_check_output(self, mute: bool, merge: bool) -> Result<Output> {
        use tokio_stream::StreamExt;
        use Error::*;

        let mut child = self
            .build()
            .stderr(Stdio::piped())
            .tap_deref_mut(|cmd| {
                if merge {
                    cmd.stdout(Stdio::piped());
                }
            })
            .spawn()
            .map_err(CmdSpawnError)?;

        fn make_reader(
            st: Option<impl AsyncRead>,
            name: &str,
        ) -> Result<impl Stream<Item = io::Result<Bytes>>> {
            st.map(into_bytes).ok_or_else(|| CmdNoHandleError {
                handle: name.into(),
            })
        }

        let stderr_reader = make_reader(child.stderr.take(), "stderr")?;
        let mut reader = if merge {
            let stdout_reader = make_reader(child.stdout.take(), "stdout")?;
            StreamExt::merge(stdout_reader, stderr_reader).left_stream()
        } else {
            stderr_reader.right_stream()
        };

        let mut out = if merge {
            Either::Left(io::stdout())
        } else {
            Either::Right(io::stderr())
        };

        let code: JoinHandle<Result<Option<i32>>> = tokio::spawn(async move {
            let status = child.wait().await.map_err(CmdWaitError)?;
            Ok(status.code())
        });

        let contents = exec_tee(&mut reader, (!mute).then(|| &mut out)).await?;

        Ok(Output {
            contents,
            code: code.await.map_err(CmdJoinError)??,
        })
    }

    /// Executes a [`Cmd`] and returns its `stdout` and `stderr`.
    ///
    /// If `mute` is `false`, then normal `stdout/stderr` output will be printed
    /// to `stdout` too.
    pub async fn exec_checkall(self, mute: bool) -> Result<Output> {
        self.exec_check_output(mute, true).await
    }

    /// Executes a [`Cmd`] and collects its `stderr`.
    ///
    /// If `mute` is `false`, then its `stderr` output will be printed to
    /// `stderr` too.
    pub async fn exec_checkerr(self, mute: bool) -> Result<Output> {
        self.exec_check_output(mute, false).await
    }

    /// Executes a [`Cmd`] and collects its `stderr`.
    ///
    /// If `mute` is `false`, then its `stderr` output will be printed to
    /// `stderr` too.
    ///
    /// This function behaves just like [`exec_checkerr`](Cmd::exec_checkerr),
    /// but in addition, the user will be prompted if (s)he wishes to
    /// continue with the command execution.
    pub async fn exec_prompt(self, mute: bool) -> Result<Output> {
        static ALL_YES: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));

        let proceed = if ALL_YES.load(Ordering::SeqCst) {
            true
        } else {
            print_cmd(&self, PROMPT_PENDING);
            let answer = tokio::task::block_in_place(move || {
                prompt(
                    "Proceed",
                    "[YES/All/No/^C]",
                    &["", "y", "yes", "a", "all", "n", "no"],
                    false,
                )
                .to_lowercase()
            });
            match answer.as_ref() {
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
        let sudo: &str = self.should_sudo().then(|| "sudo -S ").unwrap_or_default();
        let cmd = self
            .cmd
            .iter()
            .chain(&self.flags)
            .chain(&self.kws)
            .join(" ");
        write!(f, "{}{}", sudo, cmd)
    }
}

/// Gives a prompt and gets the output string.
/// This action won't end until an expected answer is found.
///
/// If `case_sensitive` is `false`, then `expected` should be all lower case
/// patterns.
pub fn prompt(question: &str, options: &str, expected: &[&str], case_sensitive: bool) -> String {
    use std::io::{self, Write};

    std::iter::repeat_with(|| {
        print_question(question, options);
        io::stdout().flush().expect("Error while flushing stdout");
        let mut answer = String::new();
        io::stdin()
            .read_line(&mut answer)
            .expect("Error while reading user input");
        if case_sensitive {
            answer
        } else {
            answer.to_lowercase()
        }
    })
    .find_map(|answer| {
        let answer = answer.trim();
        expected
            .iter()
            .any(|&x| x == answer)
            .then(|| answer.to_owned())
    })
    .unwrap() // It's impossible to find nothing out of an infinite loop.
}

/// Finds all lines in the given `text` that matches all the `patterns`.
///
/// We suppose that all patterns are legal regular expressions.
/// An error message will be returned if this is not the case.
pub fn grep<'t>(text: &'t str, patterns: &[&str]) -> Result<Vec<&'t str>> {
    patterns
        .iter()
        .map(|&pat| {
            Regex::new(pat)
                .map_err(|_e| Error::OtherError(format!("Pattern `{}` is ill-formed", pat)))
        })
        .try_collect()
        .map(|rs: Vec<Regex>| {
            text.lines()
                .filter(|line| rs.iter().all(|regex| regex.is_match(line)))
                .collect_vec()
        })
}

/// Prints the result of [`grep`] line by line.
pub fn grep_print(text: &str, patterns: &[&str]) -> Result<()> {
    grep(text, patterns).map(|lns| lns.iter().for_each(|ln| println!("{}", ln)))
}

/// Checks if an executable exists by name (consult `$PATH`) or by path.
///
/// To check by one parameter only, pass `""` to the other one.
pub fn is_exe(name: &str, path: &str) -> bool {
    (!path.is_empty() && which(path).is_ok()) || (!name.is_empty() && which(name).is_ok())
}

/// Turns an [`AsyncRead`] into a [`Stream`].
///
/// _Shamelessly copied from [StackOverflow](https://stackoverflow.com/a/59327560)._
pub fn into_bytes(reader: impl AsyncRead) -> impl Stream<Item = io::Result<Bytes>> {
    FramedRead::new(reader, BytesCodec::new()).map_ok(|bytes| bytes.freeze())
}
