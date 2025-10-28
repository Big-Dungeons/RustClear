use std::{future::Future, path::PathBuf, pin::Pin, time::Instant};

use bytes::Bytes;
use tokio::io;

use crate::record::{profile_id::ProfileId, record_buffer::RecordWriter};

pub type AsyncWriteFn = Box<dyn for<'a> FnOnce(RecordWriter<'a>) -> Pin<Box<dyn Future<Output = Result<(), io::Error>> + Send + 'a>> + Send>;
pub type AsyncUploadFn = Box<dyn FnOnce(PathBuf) -> Pin<Box<dyn Future<Output = Result<(), io::Error>> + Send>> + Send>;

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
        upload: AsyncUploadFn
    },
}