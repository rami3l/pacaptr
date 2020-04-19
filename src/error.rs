#[derive(Debug)]
pub struct Error {
    msg: String,
}

impl std::convert::From<&str> for Error {
    fn from(msg: &str) -> Self {
        Error { msg: msg.into() }
    }
}

impl std::convert::From<String> for Error {
    fn from(msg: String) -> Self {
        Error { msg }
    }
}

impl std::convert::From<std::io::Error> for Error {
    fn from(io_err: std::io::Error) -> Self {
        Error {
            msg: format!("{}", io_err),
        }
    }
}

impl std::convert::From<std::string::FromUtf8Error> for Error {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Error {
            msg: format!("{}", err),
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl std::error::Error for Error {}
