use std::time::Instant;

use fstr::FString;
use tokio::sync::oneshot;


pub enum ReplayMessage {
    Load {
        file: String,
        seeds: oneshot::Sender<(FString, u64)>,
    },
    Start {
        at: Instant,
    },
    End,
}
