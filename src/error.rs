use thiserror::Error;
use tokio::io;
use tokio::task::JoinError;

pub type Result<T> = std::result::Result<T, Error>;

/// Error type for the `pacaptr` library.
#[derive(Debug, Error)]
pub enum Error {
    /// Error while parsing CLI arguments.
    #[error("Failed to parse arguments: {msg}")]
    ArgParseError { msg: String },

    /// Error when handling config.
    #[error("Failed to handle config: {msg}")]
    ConfigError { msg: String },

    /// A subprocess fails to finish.
    #[error("Failed to get exit code of subprocess: {0}")]
    CmdJoinError(JoinError),

    /// A subprocess fails to spawn.
    #[error("Failed to spawn subprocess: {0}")]
    CmdSpawnError(io::Error),

    /// Error when trying to get the `stdout`/`stderr`/... handler out of the subprocess.
    #[error("Subprocess didn't have a handle to {handle}")]
    CmdNoHandleError { handle: String },

    /// A subprocess fails to finish.
    #[error("Subprocess failed while running: {0}")]
    CmdWaitError(io::Error),

    /// Error while converting a UTF-8 byte vector to a `String`.
    #[error(transparent)]
    FromUtf8Error(#[from] std::string::FromUtf8Error),

    /// All other cases of `io::Error`.
    #[error(transparent)]
    IOError(#[from] io::Error),

    /// A package manager operation is not implemented.
    #[error("Operation `{op}` is unimplemented for `{pm}`")]
    OperationUnimplementedError { op: String, pm: String },

    /// Miscellaneous other errors.
    #[error("{0}")]
    OtherError(String),
}
