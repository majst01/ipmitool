//! Error types shared across the crates.

use std::fmt;

/// Errors produced while talking to a BMC.
#[derive(Debug)]
pub enum Error {
    /// The underlying transport failed (socket error, resolution, ...).
    Io(std::io::Error),
    /// A DNS/hostname resolution failure.
    Resolution(String),
    /// The remote did not answer within the retry/timeout budget.
    Timeout,
    /// Session negotiation failed.
    Session(String),
    /// The BMC returned a non-zero completion code.
    Completion { code: u8, command: Option<String> },
    /// The request could not be built (e.g. payload too large, bad cipher).
    Protocol(String),
    /// A user-supplied value was invalid.
    Invalid(String),
    /// The selected interface is not supported by this build.
    Unsupported(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "{e}"),
            Error::Resolution(h) => write!(f, "could not resolve host {h}"),
            Error::Timeout => write!(f, "no response from BMC (timeout)"),
            Error::Session(m) => write!(f, "session error: {m}"),
            Error::Completion { code, command } => match command {
                Some(c) => write!(f, "{} failed: {}", c, crate::cc::completion_code_str(*code)),
                None => write!(f, "{}", crate::cc::completion_code_str(*code)),
            },
            Error::Protocol(m) => write!(f, "protocol error: {m}"),
            Error::Invalid(m) => write!(f, "invalid argument: {m}"),
            Error::Unsupported(m) => write!(f, "unsupported: {m}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
