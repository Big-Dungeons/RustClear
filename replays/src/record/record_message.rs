use std::{future::Future, path::PathBuf, pin::Pin, time::Instant};

use bytes::Bytes;
use tokio::io;

use crate::record::{profile_id::ProfileId, record_buffer::RecordWriter};

pub type AsyncWriteFn = Box<dyn for<'a> FnOnce(RecordWriter<'a>) -> Pin<Box<dyn Future<Output = Result<(), io::Error>> + Send + 'a>> + Send>;

pub enum RecordMessage {
    Start {
        initializer: AsyncWriteFn,
        at: Instant,
    },
    Record {
        received: Instant,
        profile: ProfileId,
        packet: Bytes,
    },
    Save {
        // this is so we can upload to the server directly on the existing record task.
        upload: Box<dyn FnOnce(PathBuf) -> Pin<Box<dyn Future<Output = Result<(), io::Error>> + Send>> + Send>
    },
}

impl RecordMessage {
    ///
    /// weird box::new(Box::pin()) syntax cant be extracted into a helper function :(
    /// 
    /// # Example:
    /// ```
    /// let example = RecordMessage::start(Box::new(|mut w| {
    ///     Box::pin(async move {
    ///         w.write(VERSION.as_bytes()).await?;
    ///         Ok(())
    ///     })
    /// }), Instant::now());
    /// ```
    pub fn start(initializer: AsyncWriteFn, at: Instant) -> Self {
        Self::Start { initializer, at, }
    }
}