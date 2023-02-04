//! APIs for spawning subprocesses and handling their results.

use std::{
    process::Stdio,
    sync::atomic::{AtomicBool, Ordering},
};

use bytes::{Bytes, BytesMut};
use dialoguer::FuzzySelect;
use futures::prelude::*;
use indoc::indoc;
use is_root::is_root;
use itertools::{chain, Itertools};
use regex::Regex;
use tap::prelude::*;
use tokio::{
    io::{self, AsyncRead, AsyncWrite},
    process::Command as Exec,
    task::JoinHandle,
};
#[allow(clippy::wildcard_imports)]
use tokio_util::{
    codec::{BytesCodec, FramedRead},
    compat::*,
    either::Either,
};
use which::which;

use crate::{
    error::{Error, Result},
    print::{println_quoted, prompt, question_theme},
};

/// Different ways in which a [`Cmd`] shall be dealt with.
#[derive(Copy, Clone, Debug)]
pub(crate) enum Mode {
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

/// The status code type returned by a [`Cmd`],
pub(crate) type StatusCode = i32;

/// Returns a [`Result`] for a [`Cmd`] according to if its exit status code
/// indicates an error.
///
/// # Errors
/// This function might return one of the following errors:
///
/// - [`Error::CmdStatusCodeError`], when `status` is `Some(n)` where `n != 0`.
/// - [`Error::CmdInterruptedError`], when `status` is `None`.
fn exit_result(code: Option<StatusCode>, output: Output) -> Result<Output> {
    match code {
        Some(0) => Ok(output),
        Some(code) => Err(Error::CmdStatusCodeError { code, output }),
        None => Err(Error::CmdInterruptedError),
    }
}

/// The type for captured `stdout`, and if set to [`Mode::CheckAll`], mixed with
/// captured `stderr`.
pub(crate) type Output = Vec<u8>;

/// A command to be executed, provided in `command-flags-keywords` form.
#[must_use]
#[derive(Debug, Clone, Default)]
pub(crate) struct Cmd {
    /// Flag indicating If a **normal admin** needs to run this command with
    /// `sudo`.
    pub sudo: bool,

    /// The "command" part of the command string, eg. `brew install`.
    pub cmd: Vec<String>,

    /// The "flags" part of the command string, eg. `--dry-run`.
    pub flags: Vec<String>,

    /// The "keywords" part of the command string, eg. `curl fish`.
    pub kws: Vec<String>,
}

impl Cmd {
    /// Makes a new [`Cmd`] instance with the given [`cmd`](Cmd::cmd) part.
    pub(crate) fn new(cmd: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        Cmd {
            cmd: cmd.into_iter().map(|s| s.as_ref().into()).collect(),
            ..Cmd::default()
        }
    }

    /// Makes a new [`Cmd`] instance with the given [`cmd`](Cmd::cmd) part,
    /// setting [`sudo`](field@Cmd::sudo) to `true`.
    pub(crate) fn with_sudo(cmd: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        Cmd::new(cmd).sudo(true)
    }

    /// Overrides the value of [`flags`](field@Cmd::flags).
    pub(crate) fn flags(self, flags: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        Cmd {
            flags: flags.into_iter().map(|s| s.as_ref().into()).collect(),
            ..self
        }
    }

    /// Overrides the value of [`kws`](field@Cmd::kws).
    pub(crate) fn kws(self, kws: impl IntoIterator<Item = impl AsRef<str>>) -> Self {
        Cmd {
            kws: kws.into_iter().map(|s| s.as_ref().into()).collect(),
            ..self
        }
    }

    /// Overrides the value of [`sudo`](field@Cmd::sudo).
    pub(crate) fn sudo(self, sudo: bool) -> Self {
        Cmd { sudo, ..self }
    }

    /// Determines if this command actually needs to run with `sudo -S`.
    ///
    /// If a **normal admin** needs to run it with `sudo`, and we are not
    /// `root`, then this is the case.
    #[must_use]
    fn should_sudo(&self) -> bool {
        self.sudo && !is_root()
    }

    /// Converts a [`Cmd`] object into an [`Exec`].
    #[must_use]
    fn build(self) -> Exec {
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
async fn exec_tee(
    src: impl Stream<Item = io::Result<Bytes>>,
    out: Option<impl AsyncWrite>,
) -> Result<Vec<u8>> {
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

macro_rules! docs_errors_exec {
    () => {
        indoc! {"
            # Errors
            This function might return one of the following errors:

            - [`Error::CmdJoinError`]
            - [`Error::CmdNoHandleError`]
            - [`Error::CmdSpawnError`]
            - [`Error::CmdWaitError`]
            - [`Error::CmdStatusCodeError`]
            - [`Error::CmdInterruptedError`]
        "}
    };
}

impl Cmd {
    /// Executes a [`Cmd`] and returns its output.
    ///
    /// The exact behavior depends on the [`Mode`] passed in (see the definition
    /// of [`Mode`] for more info).
    #[doc = docs_errors_exec!()]
    pub(crate) async fn exec(self, mode: Mode) -> Result<Output> {
        match mode {
            Mode::PrintCmd => {
                println_quoted(&*prompt::CANCELED, &self);
                Ok(Output::default())
            }
            Mode::Mute => self.exec_checkall(true).await,
            Mode::CheckAll => {
                println_quoted(&*prompt::RUNNING, &self);
                self.exec_checkall(false).await
            }
            Mode::CheckErr => {
                println_quoted(&*prompt::RUNNING, &self);
                self.exec_checkerr(false).await
            }
            Mode::Prompt => self.exec_prompt(false).await,
        }
    }

    /// Inner implementation of [`Cmd::exec_checkerr`] (if `merge` is `false`)
    /// and [`Cmd::exec_checkall`] (otherwise).
    #[doc = docs_errors_exec!()]
    async fn exec_check_output(self, mute: bool, merge: bool) -> Result<Output> {
        use tokio_stream::StreamExt;
        use Error::{CmdJoinError, CmdNoHandleError, CmdSpawnError, CmdWaitError};

        fn make_reader(
            src: Option<impl AsyncRead>,
            name: &str,
        ) -> Result<impl Stream<Item = io::Result<Bytes>>> {
            src.map(into_bytes).ok_or_else(|| CmdNoHandleError {
                handle: name.into(),
            })
        }

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

        let output = exec_tee(&mut reader, (!mute).then_some(&mut out)).await?;
        let code = code.await.map_err(CmdJoinError)??;
        exit_result(code, output)
    }

    /// Executes a [`Cmd`] and returns its `stdout` and `stderr`.
    ///
    /// If `mute` is `false`, then normal `stdout/stderr` output will be printed
    /// to `stdout` too.
    #[doc = docs_errors_exec!()]
    async fn exec_checkall(self, mute: bool) -> Result<Output> {
        self.exec_check_output(mute, true).await
    }

    /// Executes a [`Cmd`] and collects its `stderr`.
    ///
    /// If `mute` is `false`, then its `stderr` output will be printed to
    /// `stderr` too.
    #[doc = docs_errors_exec!()]
    async fn exec_checkerr(self, mute: bool) -> Result<Output> {
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
    #[doc = docs_errors_exec!()]
    async fn exec_prompt(self, mute: bool) -> Result<Output> {
        /// If the user has skipped all the prompts with `yes`.
        static ALL: AtomicBool = AtomicBool::new(false);

        // The answer obtained from the prompt.
        // The only Atomic* we're dealing with is `ALL`, so `Ordering::Relaxed` is fine.
        // See: <https://marabos.nl/atomics/memory-ordering.html#relaxed>
        let proceed = ALL.load(Ordering::Relaxed) || {
            println_quoted(&*prompt::PENDING, &self);
            let answer = tokio::task::block_in_place(move || {
                prompt(
                    "Proceed",
                    "with the previous command?",
                    &["Yes", "All", "No"],
                )
            })?;
            match answer {
                // The default answer is `Yes`.
                0 => true,
                // You can also say `All` to answer `Yes` to all the other questions that follow.
                1 => {
                    ALL.store(true, Ordering::Relaxed);
                    true
                }
                // Or you can say `No`.
                2 => false,
                // ! I didn't put a `None` option because you can just Ctrl-C it if you want.
                _ => unreachable!(),
            }
        };
        if !proceed {
            return Ok(Output::default());
        }
        println_quoted(&*prompt::RUNNING, &self);
        self.exec_checkerr(mute).await
    }
}

impl std::fmt::Display for Cmd {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let sudo: &str = if self.should_sudo() { "sudo -S " } else { "" };
        let cmd = chain!(&self.cmd, &self.flags, &self.kws).join(" ");
        write!(f, "{sudo}{cmd}")
    }
}

/// Gives a prompt and returns the index of the user choice.
fn prompt(prompt: &str, question: &str, expected: &[&str]) -> io::Result<usize> {
    FuzzySelect::with_theme(&question_theme(prompt))
        .with_prompt(question)
        .items(expected)
        .default(0)
        .interact()
}

macro_rules! docs_errors_grep {
    () => {
        indoc! {"
            # Errors
            Returns an [`Error::OtherError`] when any of the
            regex patterns is ill-formed.
        "}
    };
}

/// Finds all lines in the given `text` that matches all the `patterns`.
///
/// We suppose that all patterns are legal regular expressions.
/// An error message will be returned if this is not the case.
#[doc = docs_errors_grep!()]
fn grep<'t>(text: &'t str, patterns: &[&str]) -> Result<Vec<&'t str>> {
    let patterns: Vec<Regex> = patterns
        .iter()
        .map(|pat| {
            Regex::new(pat)
                .map_err(|_e| Error::OtherError(format!("Pattern `{pat}` is ill-formed")))
        })
        .try_collect()?;
    Ok(text
        .lines()
        .filter(|line| patterns.iter().all(|pat| pat.is_match(line)))
        .collect())
}

/// Prints the result of [`grep`] line by line.
#[doc = docs_errors_grep!()]
pub(crate) fn grep_print(text: &str, patterns: &[&str]) -> Result<()> {
    grep(text, patterns).map(|lns| lns.iter().for_each(|ln| println!("{ln}")))
}

/// Checks if an executable exists by name (consult `$PATH`) or by path.
///
/// To check by one parameter only, pass `""` to the other one.
#[must_use]
pub(crate) fn is_exe(name: &str, path: &str) -> bool {
    (!path.is_empty() && which(path).is_ok()) || (!name.is_empty() && which(name).is_ok())
}

/// Turns an [`AsyncRead`] into a [`Stream`].
///
/// _Shamelessly copied from [`StackOverflow`](https://stackoverflow.com/a/59327560)._
fn into_bytes(reader: impl AsyncRead) -> impl Stream<Item = io::Result<Bytes>> {
    FramedRead::new(reader, BytesCodec::new()).map_ok(BytesMut::freeze)
}
