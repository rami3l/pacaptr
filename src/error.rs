//! Basic error definitions specific to this crate.

use std::{
    fmt::{self, Debug},
    process::{ExitCode, Termination},
};

use thiserror::Error;
use tokio::{io, task::JoinError};

use crate::{
    exec::{Output, StatusCode},
    print,
};

/// A specialized [`Result`](std::result::Result) type used by
/// [`pacaptr`](crate).
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Error type for the [`pacaptr`](crate) library.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// Error when parsing CLI arguments.
    #[allow(missing_docs)]
    #[error("failed to parse arguments: {msg}")]
    ArgParseError { msg: String },

    /// Error when handling a [`Config`](crate::config::Config).
    #[error("failed to parse config")]
    ConfigError(#[from] figment::Error),

    /// A [`Cmd`](crate::exec::Cmd) failed to finish.
    #[error("failed to get exit code of subprocess")]
    CmdJoinError(#[from] JoinError),

    /// A [`Cmd`](crate::exec::Cmd) failed to spawn.
    #[error("failed to spawn subprocess")]
    CmdSpawnError(#[source] io::Error),

    /// Error when trying to get a handle (e.g. `stdout`, `stderr`) out of a
    /// running [`Cmd`](crate::exec::Cmd).
    #[allow(missing_docs)]
    #[error("subprocess didn't have a handle to {handle}")]
    CmdNoHandleError { handle: String },

    /// A [`Cmd`](crate::exec::Cmd) failed when waiting for it to finish.
    #[error("subprocess failed while running")]
    CmdWaitError(#[source] io::Error),

    /// A [`Cmd`](crate::exec::Cmd) exited with an error.
    #[allow(missing_docs)]
    #[error("subprocess exited with code {code}")]
    CmdStatusCodeError { code: StatusCode, output: Output },

    /// A [`Cmd`](crate::exec::Cmd) was interrupted by a signal.
    #[error("subprocess interrupted by signal")]
    CmdInterruptedError,

    /// Error while converting a [`Vec<u8>`] to a [`String`].
    #[error(transparent)]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    /// Error while rendering a dialog.
    #[error(transparent)]
    DialogError(#[from] dialoguer::Error),

    /// A non-specific [`io::Error`].
    #[error(transparent)]
    IoError(#[from] io::Error),

    /// A [`Pm`](crate::pm::Pm) operation is not implemented.
    #[allow(missing_docs)]
    #[error("operation `{op}` is unimplemented for `{pm}`")]
    OperationUnimplementedError { op: String, pm: String },

    /// Miscellaneous other error.
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
