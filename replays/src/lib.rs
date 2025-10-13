mod record;
mod replay;
mod error;
mod replay_packet;

pub use record::profile_id::ProfileId as ProfileId;

pub use replay::run_replay::ReplayHandler as ReplayHandler;
pub use record::run_record::RecordHandler as RecordHandler;

pub use record::record_message::RecordMessage as RecordMessage;
pub use replay::replay_message::ReplayMessage as ReplayMessage;

pub use replay::replay_callback::ReplayCallback as ReplayCallback;

pub use replay_packet::ReplayPacket as ReplayPacket;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, time::{Duration, Instant}};

    use bytes::{Buf, Bytes};
    use fstr::ToFString;
    use tokio::sync::oneshot;
    use uuid::Uuid;

    use crate::{ProfileId, RecordHandler, RecordMessage, ReplayCallback, ReplayHandler, ReplayMessage, ReplayPacket};

    pub struct Callback {
        last_packet: Option<ReplayPacket>
    }
    
    impl ReplayCallback for Callback {
        async fn callback(&mut self, packet: ReplayPacket) {
            println!("packet data: {:?}", str::from_utf8(packet.packet.chunk()));
            self.last_packet = Some(packet)
        }
    }
    
    #[test]
    fn start_replay() {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let (tx, mut rx) = oneshot::channel::<PathBuf>();
        rt.block_on(async {
            let handler = RecordHandler::spawn("test_replays");
            handler.send(RecordMessage::start(Box::new(|mut buf| {
                Box::pin(async move {
                    buf.write(&111u64.to_be_bytes()).await
                })
            }), Instant::now())).unwrap();
            
            tokio::time::sleep(Duration::from_secs(1)).await;
            
            handler.send(RecordMessage::Record { 
                received: Instant::now(), 
                profile: ProfileId::new(Uuid::new_v4()), 
                packet: Bytes::from_static(b"1") 
            }).unwrap();

            tokio::time::sleep(Duration::from_secs(1)).await;
            
            handler.send(RecordMessage::Record { 
                received: Instant::now(), 
                profile: ProfileId::new(Uuid::new_v4()), 
                packet: Bytes::from_static(b"2") 
            }).unwrap();

            tokio::time::sleep(Duration::from_secs(1)).await;
            
            handler.send(RecordMessage::Record { 
                received: Instant::now(), 
                profile: ProfileId::new(Uuid::new_v4()), 
                packet: Bytes::from_static(b"3") 
            }).unwrap();

            tokio::time::sleep(Duration::from_secs(1)).await;
            
            handler.send(RecordMessage::Record { 
                received: Instant::now(), 
                profile: ProfileId::new(Uuid::new_v4()), 
                packet: Bytes::from_static(b"4") 
            }).unwrap();

            tokio::time::sleep(Duration::from_secs(1)).await;
            
            handler.send(RecordMessage::Save { upload: Box::new(move |path_buf| {
                Box::pin(async move {
                    println!("saved to {:?}", path_buf);
                    tx.send(path_buf).unwrap();
                    Ok(())
                })
            }) }).unwrap();
            
            tokio::time::sleep(Duration::from_secs(1)).await;
        });
        
        rt.block_on(async {
            let path = rx.try_recv().unwrap();
            
            let handler = ReplayHandler::spawn(|buf| {
                Ok(buf.get_u64())
            }, Callback { last_packet: None });
            
            let (tx, rx) = oneshot::channel();
            handler.send(ReplayMessage::Load { file: path.to_str().unwrap().to_fstring(), sender: tx }).unwrap();
            let res: u64 = rx.await.unwrap().unwrap();
            
            assert_eq!(res, 111);
            handler.send(ReplayMessage::Start { at: Instant::now() }).unwrap();
            println!("started...");
            tokio::time::sleep(Duration::from_secs(3)).await;
            
            handler.send(ReplayMessage::End).unwrap();
            tokio::time::sleep(Duration::from_secs(3)).await;
            println!("ended...");
        })
    }
}