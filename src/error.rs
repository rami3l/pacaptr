//! Basic error definitions specific to this crate.

use miette::Diagnostic;
use thiserror::Error;
use tokio::{io, task::JoinError};

use crate::exec::{Output, StatusCode};

/// A specialized [`Result`](std::result::Result) type used by
/// [`pacaptr`](crate).
pub(crate) type Result<T, E = Error> = std::result::Result<T, E>;

/// Error type for the [`pacaptr`](crate) library.
#[derive(Debug, Error, Diagnostic)]
#[non_exhaustive]
pub enum Error {
    /// Error while parsing CLI arguments.
    #[error("Failed to parse arguments: {msg}")]
    #[allow(missing_docs)]
    ArgParseError { msg: String },

    /// Error when handling a [`Config`](crate::dispatch::Config).
    #[error("Failed to handle config: {msg}")]
    #[allow(missing_docs)]
    ConfigError { msg: String },

    /// An [`Cmd`](crate::exec::Cmd) fails to finish.
    #[error("Failed to get exit code of subprocess: {0}")]
    CmdJoinError(JoinError),

    /// An [`Cmd`](crate::exec::Cmd) fails to spawn.
    #[error("Failed to spawn subprocess: {0}")]
    CmdSpawnError(io::Error),

    /// Error when trying to get the `stdout`/`stderr`/... handler out of a
    /// running an [`Cmd`](crate::exec::Cmd).
    #[error("Subprocess didn't have a handle to {handle}")]
    #[allow(missing_docs)]
    CmdNoHandleError { handle: String },

    /// An [`Cmd`](crate::exec::Cmd) fails while waiting for it to finish.
    #[error("Subprocess failed while running: {0}")]
    CmdWaitError(io::Error),

    /// An [`Cmd`](crate::exec::Cmd) exits with an error.
    #[error("Subprocess exited with code {code}")]
    #[allow(missing_docs)]
    CmdStatusCodeError { code: StatusCode, output: Output },

    /// An [`Cmd`](crate::exec::Cmd) gets interrupted by a signal.
    #[error("Subprocess interrupted by signal")]
    CmdInterruptedError,

    /// Error while converting a [`Vec<u8>`] to a [`String`].
    #[error(transparent)]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    /// An unmentioned case of [`io::Error`].
    #[error(transparent)]
    IoError(#[from] io::Error),

    /// A [`Pm`](crate::pm::Pm) operation is not implemented.
    #[error("Operation `{op}` is unimplemented for `{pm}`")]
    #[allow(missing_docs)]
    OperationUnimplementedError { op: String, pm: String },

    /// Miscellaneous other error.
    #[error("{0}")]
    OtherError(String),
}
