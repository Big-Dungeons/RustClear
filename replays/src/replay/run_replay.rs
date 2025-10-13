use std::time::Instant;

use bytes::BytesMut;
use tokio::{fs::File, io, sync::mpsc::{UnboundedReceiver, UnboundedSender, error::SendError, unbounded_channel}, task::AbortHandle};

use crate::{ReplayCallback, error::BufferError, replay::{replay_buffer::ReplayBuffer, replay_message::ReplayMessage}};

/// cheaply clonable handle for the Replay runner task.
pub struct ReplayHandler<T: Send + 'static> {
    tx: UnboundedSender<ReplayMessage<T>>,
    abort: AbortHandle,
}

impl<T: Send + 'static> ReplayHandler<T> {
    /// spawns the replay task and returns a handle to it.
    /// 
    /// the init function is used to get data prepended in the replay, such as seed data.
    /// youy may want to validate this.
    /// this must match the init data put in the start message initializer for the saved replay.
    /// the callback will be run every time a packet is recieved at its correct time.
    pub fn spawn<C: ReplayCallback + 'static>(init: fn(&mut BytesMut) -> Result<T, BufferError>, callback: C) -> Self {
        let (tx, rx) = unbounded_channel();
        let runner = ReplayRunner::new(rx, init, callback);
        let handle = tokio::spawn(runner.run()).abort_handle();
        Self { tx, abort: handle }
    }
    
    pub fn send(&self, message: ReplayMessage<T>) -> Result<(), SendError<ReplayMessage<T>>> {
        self.tx.send(message)
    }
    
    pub fn abort(&self) {
        self.abort.abort()
    }
}

struct ReplayRunner<T: Send + 'static, C: ReplayCallback> {
    rx: UnboundedReceiver<ReplayMessage<T>>,
    init: fn(&mut BytesMut) -> Result<T, BufferError>,
    callback: C
}

impl<T: Send + 'static, C: ReplayCallback> ReplayRunner<T, C> {
    fn new(rx: UnboundedReceiver<ReplayMessage<T>>, init: fn(&mut BytesMut) -> Result<T, BufferError>, callback: C) -> Self {
        Self { rx, init, callback }
    }
    
    async fn run(mut self) {
        let mut buffer: Option<ReplayBuffer> = None;
        let mut play: Option<Instant> = None;
    
        loop {
            if let Some(buf) = buffer.as_mut() {
                tokio::select! {
                    res = self.rx.recv() => {
                        let Some(msg) = res else { continue };
                        match msg {
                            ReplayMessage::Load { file: _, sender: _ } => eprintln!("Already running a replay!"),
                            ReplayMessage::Start { at } => {
                                buf.start = at;
                                play = Some(at)
                            }
                            ReplayMessage::End => {
                                buffer = None;
                                play = None;
                            }
                        }
                    }
                    
                    res = buf.get_packet(), if play.is_some() => {
                        match res {
                            Ok(packet) => self.callback.callback(packet).await,
                            Err(BufferError::Pending) => continue,
                            Err(BufferError::EndOfFile) => {
                                buffer = None;
                                play = None;
                            }
                            Err(_) => break,
                        }
                    }
                }
            } else {
                while let Some(res) = self.rx.recv().await {
                    let ReplayMessage::Load { file, sender } = res else {
                        continue;
                    };
                    match load(file.as_str(), self.init).await {
                        Ok((buf, res)) => {
                            buffer = Some(buf);
                            let _ = sender.send(Ok(res));
                        }
                        Err(e) => {
                            let _ = sender.send(Err(e));
                        }
                    }
                    break;
                }
            }
        }
        
        println!("replay thread died");
    }
}

async fn load<T>(file: &str, init: fn(&mut BytesMut) -> Result<T, BufferError>) -> Result<(ReplayBuffer, T), io::Error> {
    let file = File::open(file).await?;
    let mut buffer = ReplayBuffer::new(file);
    buffer.initialize().await?;
    let res: T = buffer.with_buffer(init).await?;
    Ok((buffer, res))
}