use std::time::Instant;

use fstr::FString;
use tokio::{io, sync::oneshot};

pub enum ReplayMessage<T> {
    Load {
        file: FString,
        sender: oneshot::Sender<Result<T, io::Error>>
    },
    Start {
        at: Instant,
    },
    End,
}
