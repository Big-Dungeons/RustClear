use std::{any, error::Error, fmt::Display};

use tokio::io;

#[derive(Debug)]
pub struct BufferAlreadyOpen;

impl Error for BufferAlreadyOpen {}

impl Display for BufferAlreadyOpen {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Buffer is already open!")
    }
}

#[derive(Debug)]
pub enum ReplayError {
    EndOfFile,
    Pending,
    Other(anyhow::Error)
}

impl From<anyhow::Error> for ReplayError {
    fn from(value: anyhow::Error) -> Self {
        Self::Other(value)
    }
}

impl From<io::Error> for ReplayError {
    fn from(value: io::Error) -> Self {
        Self::Other(anyhow::Error::from(value))
    }
}

impl Error for ReplayError {}

impl Display for ReplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EndOfFile => f.write_str("End of File!"),
            Self::Pending => f.write_str("Pending packets!"),
            Self::Other(e) => e.fmt(f)
        }
    }
}