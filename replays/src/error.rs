use std::{error::Error, fmt::Display, str::Utf8Error};
use tokio::io;


#[derive(Debug)]
pub enum BufferError {
    AlreadyOpen,
    EndOfFile,
    Pending,
    Other(anyhow::Error)
}

impl From<anyhow::Error> for BufferError {
    fn from(value: anyhow::Error) -> Self {
        Self::Other(value)
    }
}

impl From<io::Error> for BufferError {
    fn from(value: io::Error) -> Self {
        Self::Other(anyhow::Error::from(value))
    }
}

impl From<Utf8Error> for BufferError {
    fn from(value: Utf8Error) -> Self {
        Self::Other(anyhow::Error::from(value))
    }
}

impl From<BufferError> for io::Error {
    fn from(value: BufferError) -> Self {
        io::Error::other(value)
    }
}

impl Error for BufferError {}

impl Display for BufferError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AlreadyOpen => f.write_str("Buffer already open!"),
            Self::EndOfFile => f.write_str("End of File!"),
            Self::Pending => f.write_str("Pending packets!"),
            Self::Other(e) => e.fmt(f)
        }
    }
}