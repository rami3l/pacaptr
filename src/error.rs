//! Basic error definitions specific to this crate.

use std::{
    fmt::{self, Debug},
    process::{ExitCode, Termination},
    string::FromUtf8Error,
};

use thiserror::Error;
use tokio::{io, task::JoinError};

use crate::exec::{Output, StatusCode};
use crate::print;

/// A specialized [`Result`](std::result::Result) type used by
/// [`pacaptr`](crate).
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Error type for the [`pacaptr`](crate) library.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// Error when parsing CLI arguments.
    #[error("Failed to parse arguments: {msg}")]
    #[allow(missing_docs)]
    ArgParseError { msg: String },

    /// Error when handling a [`Config`](crate::config::Config).
    #[error("Failed to parse config: {0}")]
    ConfigError(#[from] figment::Error),

    /// A [`Cmd`](crate::exec::Cmd) failed to finish.
    #[error("Failed to finish subprocess execution: {0}")]
    CmdJoinError(JoinError),

    /// A [`Cmd`](crate::exec::Cmd) failed to spawn.
    #[error("Failed to spawn subprocess: {0}")]
    CmdSpawnError(io::Error),

    /// Error when trying to get a handle (e.g. `stdout`, `stderr`) out of a
    /// running [`Cmd`](crate::exec::Cmd).
    #[error("Subprocess didn't have a handle to {handle}")]
    #[allow(missing_docs)]
    CmdNoHandleError { handle: String },

    /// A [`Cmd`](crate::exec::Cmd) failed when waiting for it to finish.
    #[error("Subprocess failed while running: {0}")]
    CmdWaitError(io::Error),

    /// A [`Cmd`](crate::exec::Cmd) exited with an error.
    #[error("Subprocess exited with code {code}")]
    #[allow(missing_docs)]
    CmdStatusCodeError { code: StatusCode, output: Output },

    /// A [`Cmd`](crate::exec::Cmd) was interrupted by a signal.
    #[error("Subprocess interrupted by signal")]
    CmdInterruptedError,

    /// Error while converting a [`Vec<u8>`] to a [`String`].
    #[error(transparent)]
    FromUtf8Error(#[from] FromUtf8Error),

    /// Error while rendering a dialog.
    #[error(transparent)]
    DialogError(#[from] dialoguer::Error),

    /// A non-specific [`io::Error`].
    #[error(transparent)]
    IoError(#[from] io::Error),

    /// A generic [`JoinError`].
    #[error(transparent)]
    JoinError(#[from] JoinError),

    /// A [`Pm`](crate::pm::Pm) operation is not implemented.
    #[error("Operation `{op}` is unimplemented for `{pm}`")]
    #[allow(missing_docs)]
    OperationUnimplementedError { op: String, pm: String },

    /// An error from a non-specified category.
    #[error("{0}")]
    OtherError(String),
}

#[allow(clippy::module_name_repetitions)]
/// A simple [`enum@Error`] wrapper designed to be returned in the `main`
/// function. It delegates its [`Debug`] implementation to the
/// [`std::fmt::Display`] implementation of its underlying error.
pub struct MainError(Error);

impl From<Error> for MainError {
    fn from(e: Error) -> Self {
        Self(e)
    }
}

impl Debug for MainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Erase the default "Error: " message header.
        write!(f, "\r")?;
        print::write(f, &*print::prompt::ERROR, &self.0)
    }
}

impl Termination for MainError {
    fn report(self) -> ExitCode {
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        match self.0 {
            Error::CmdStatusCodeError { code, .. } => code as u8,
            _ => 1,
        }
        .into()
    }
}
