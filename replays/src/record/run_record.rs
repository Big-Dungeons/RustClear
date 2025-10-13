use std::{path::PathBuf, time::Instant};

use tokio::{sync::mpsc::{UnboundedReceiver, UnboundedSender, error::SendError, unbounded_channel}, task::AbortHandle};

use crate::{record::{record_buffer::{RecordBuffer}, record_message::RecordMessage}, replay_packet::ReplayPacket};

/// cheaply clonable handle for the record runner task.
#[derive(Debug, Clone)]
pub struct RecordHandler {
    tx: UnboundedSender<RecordMessage>,
    abort: AbortHandle
}

impl RecordHandler {
    /// spawns the recording task and returns a handle to it.
    pub fn spawn(replay_path: &str) -> Self {
        let (tx, rx) = unbounded_channel();
        let runner = RecordRunner::new(rx);
        let handle = tokio::spawn(runner.run(PathBuf::from(replay_path))).abort_handle();
        Self {
            tx,
            abort: handle
        }
    }
    
    pub fn send(&self, message: RecordMessage) -> Result<(), SendError<RecordMessage>> {
        self.tx.send(message)
    }
    
    pub fn abort(&self) {
        self.abort.abort()
    }
}

pub struct RecordRunner {
    rx: UnboundedReceiver<RecordMessage>
}

impl RecordRunner {
    fn new(rx: UnboundedReceiver<RecordMessage>) -> Self {
        Self {
            rx
        }
    }
    
    async fn run(mut self, replay_path: PathBuf) {
        let mut start: Option<Instant> = None;
        let mut buffer: RecordBuffer = RecordBuffer::open_with(replay_path).await.unwrap();
        
        while let Some(message) = self.rx.recv().await {
            match message {
                RecordMessage::Start { initializer, at } => {
                    start = Some(at);
    
                    buffer.new_replay().await.unwrap();
                    initializer(buffer.get_writer().unwrap() /* this unwrap is safe, the rest are for testing */).await.unwrap();
                }
                RecordMessage::Record { received, profile, packet } => {
                    let Some(start) = start else { continue };
                    let since_start = received.duration_since(start);
                    let packet = ReplayPacket { since_start, profile, packet, }.serialize();
    
                    buffer.write(&packet).await.unwrap();
                }
                RecordMessage::Save { upload } => {
                    let path: PathBuf = buffer.finish().await.unwrap();
                    upload(path).await.unwrap();
                }
            }
        }

        println!("Record thread died");
    }
}