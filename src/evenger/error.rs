
use std::ops::Deref;

#[derive(Debug)]
pub enum Error {
    Description(String, Box<dyn std::error::Error>),
    Errno(nix::errno::Errno),
    IOError(std::io::Error),
    Message(String),
}

impl Error {
    pub fn msg<S: Into<String>>(msg: S) -> Self {
        Self::Message(msg.into())
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Error::Description(desc, err) => write!(fmt, "{}: {}", desc, err.deref()),
            Error::Errno(err) => err.fmt(fmt),
            Error::IOError(err) => err.fmt(fmt),
            Error::Message(msg) => msg.fmt(fmt),
        }
    }
}

impl std::error::Error for Error {
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Error::Message(msg)
    }
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::IOError(error)
    }
}

impl From<nix::errno::Errno> for Error {
    fn from(error: nix::errno::Errno) -> Self {
        Error::Errno(error)
    }
}

impl From<nix::Error> for Error {
    fn from(error: nix::Error) -> Self {
        match error {
            nix::Error::Sys(errno) => errno.into(),
            _ => Error::Message("Unknown Error".into()),
        }
    }
}