//! This module defines the basic error type used in this crate.

use thiserror::Error;
use tokio::{io, task::JoinError};

pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Error type for the `pacaptr` library.
#[derive(Debug, Error)]
pub enum Error {
    /// Error while parsing CLI arguments.
    #[error("Failed to parse arguments: {msg}")]
    ArgParseError { msg: String },

    /// Error when handling a [`dispatch::Config`].
    #[error("Failed to handle config: {msg}")]
    ConfigError { msg: String },

    /// An [`exec::Cmd`] fails to finish.
    #[error("Failed to get exit code of subprocess: {0}")]
    CmdJoinError(JoinError),

    /// An [`exec::Cmd`] fails to spawn.
    #[error("Failed to spawn subprocess: {0}")]
    CmdSpawnError(io::Error),

    /// Error when trying to get the `stdout`/`stderr`/... handler out of a running an [`exec::Cmd`].
    #[error("Subprocess didn't have a handle to {handle}")]
    CmdNoHandleError { handle: String },

    /// An [`exec::Cmd`] fails to finish.
    #[error("Subprocess failed while running: {0}")]
    CmdWaitError(io::Error),

    /// Error while converting a [`Vec<u8>`] to a [`String`].
    #[error(transparent)]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    /// An unmentioned case of [`io::Error`].
    #[error(transparent)]
    IoError(#[from] io::Error),

    /// A [`pm::Pm`] operation is not implemented.
    #[error("Operation `{op}` is unimplemented for `{pm}`")]
    OperationUnimplementedError { op: String, pm: String },

    /// Miscellaneous other error.
    #[error("{0}")]
    OtherError(String),
}
