mod record;
mod replay;
mod error;
mod replay_packet;

pub use replay::run_replay::ReplayHandler as ReplayHandler;
pub use record::run_record::RecordHandler as RecordHandler;

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

    use crate::{RecordHandler, ReplayCallback, ReplayHandler, ReplayPacket};

    pub struct Callback {
        state: u64
    }
    
    impl ReplayCallback for Callback {
        async fn callback(&mut self, mut packet: ReplayPacket) {
            let data = packet.packet.get_u64();
            println!("recieved: {}", data);
            assert_eq!(data, self.state);
            self.state += 1;
        }
    }
    
    #[test]
    fn start_replay() {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        let (tx, mut rx) = oneshot::channel::<PathBuf>();
        rt.block_on(async {
            let recording = RecordHandler::spawn("test_replays");
            
            recording.start(Box::new(|mut buf| {
                Box::pin(async move {
                    buf.write(&111u64.to_be_bytes()).await
                })
            }), Instant::now()).unwrap();
        
            tokio::time::sleep(Duration::from_secs(1)).await;
            
            for i in 0usize..500usize {
                recording.record(
                    Instant::now(), 
                    Uuid::new_v4(), 
                    Bytes::copy_from_slice(&i.to_be_bytes())
                ).unwrap();
                tokio::time::sleep(Duration::from_micros(100)).await;
            }
            
            tokio::time::sleep(Duration::from_secs(1)).await;
            
            recording.save(Box::new(move |path_buf| {
                Box::pin(async move {
                    println!("saved to {:?}", path_buf);
                    tx.send(path_buf).unwrap();
                    Ok(())
                })
            })).unwrap();
            
            tokio::time::sleep(Duration::from_secs(1)).await;
        });
        
        rt.block_on(async {
            let path = rx.try_recv().unwrap();
            
            let replay = ReplayHandler::spawn(|buf| {
                Ok(buf.get_u64())
            }, Callback { state: 0 });
            
            let res: u64 = replay.load(path.to_str().unwrap().to_fstring()).await.unwrap();
            assert_eq!(res, 111);
            
            replay.start(Instant::now()).unwrap();
            tokio::time::sleep(Duration::from_secs(3)).await;
            
            replay.end().unwrap();
            tokio::time::sleep(Duration::from_secs(3)).await;
            println!("ended...");
        })
    }
}