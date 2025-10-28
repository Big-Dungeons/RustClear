use std::{path::PathBuf, time::Instant};

use bytes::Bytes;
use tokio::{sync::mpsc::{UnboundedReceiver, UnboundedSender, error::SendError, unbounded_channel}, task::AbortHandle};
use uuid::Uuid;

use crate::{record::{profile_id::ProfileId, record_buffer::RecordBuffer, record_message::{AsyncUploadFn, AsyncWriteFn, RecordMessage}}, replay_packet::ReplayPacket};

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
    
    /// starts the recording with an initializer.
    /// 
    /// # Example
    /// ```
    /// handler.start(Box::new(|mut buf| {
    ///     Box::pin(async move {
    ///         buf.write(&111u64.to_be_bytes()).await
    ///     })
    /// }), Instant::now()).unwrap();
    /// ```
    pub fn start(&self, initializer_fn: AsyncWriteFn, at: Instant) -> Result<(), SendError<RecordMessage>> {
        self.tx.send(RecordMessage::Start { initializer: initializer_fn, at })
    }
    
    /// records data to the replay.
    /// 
    /// recorded data should be sent sequentially from its timing. If a packet is recorded after another but happened before,
    /// it will wait for the prior packet(s) and then immedietly send.
    pub fn record(&self, at: Instant, profile: Uuid, packet: Bytes) -> Result<(), SendError<RecordMessage>> {
        self.tx.send(RecordMessage::Record { received: at, profile: ProfileId::new(profile), packet })
    }
    
    /// saves the replay and gives an async closure with the path the replay was saved to.
    /// 
    /// # Examples
    /// ```
    /// handler.save(Box::new(move |path_buf| {
    ///     Box::pin(async move {
    ///         println!("saved to {:?}", path_buf);
    ///         tx.send(path_buf).unwrap();
    ///         Ok(())
    ///     })
    /// })).unwrap();
    pub fn save(&self, upload_fn: AsyncUploadFn) -> Result<(), SendError<RecordMessage>> {
        self.tx.send(RecordMessage::Save { upload: upload_fn })
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