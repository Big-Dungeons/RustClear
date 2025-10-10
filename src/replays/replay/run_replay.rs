use std::{sync::OnceLock, time::Instant};

use fstr::FString;
use tokio::{fs::File, sync::{mpsc::{UnboundedSender, unbounded_channel}, oneshot}};

use crate::replays::{error::ReplayError, replay::{replay_buffer::ReplayBuffer, replay_message::ReplayMessage}, replay_packet::ReplayPacket};


static REPLAYHANDLE: OnceLock<UnboundedSender<ReplayMessage>> = OnceLock::new();

pub fn get_replay_handle() -> &'static UnboundedSender<ReplayMessage> {
    REPLAYHANDLE.get().unwrap()
}

pub async fn run_replay_thread(file: File) {
    let (tx, mut rx) = unbounded_channel();
    REPLAYHANDLE.set(tx).unwrap();

    let mut buffer: ReplayBuffer = ReplayBuffer::new(file);
    let mut play: Option<Instant> = None;

    // let mut clients: HashMap<u128, UnboundedSender<>> = HashMap::new();

    loop {
        tokio::select! {
            Some(res) = rx.recv() => {
                match res {
                    ReplayMessage::Load { file, seeds } => {
                        load(&mut buffer, file.as_str(), seeds).await.unwrap();
                    },
                    ReplayMessage::Start { at } => {
                        buffer.start = at;
                        play = Some(at)
                    },
                    ReplayMessage::End => {
                        play = None;
                    }
                }
            }

            Ok(packet) = buffer.get_packet(), if play.is_some() => {
                handle_packet(packet);
            }
        }
    }
}

async fn load(buf: &mut ReplayBuffer, file: &str, seeds: oneshot::Sender<(FString, u64)>) -> anyhow::Result<()> {
    let file = File::open(file).await?;
    *buf = ReplayBuffer::new(file);
    buf.initialize().await?;
    let res = buf.get_seeds().await?;
    seeds.send(res).unwrap();
    Ok(())
}

fn handle_packet(packet: ReplayPacket, /*clients: &mut clients */) {

}


#[test]
fn test() {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async {
        let file = tokio::fs::File::open("C:\\Users\\Michael\\RustroverProjects\\RustClear\\replays\\0.1.0_partial_09.10.25_10-57-06.rcrp").await.unwrap();
        let mut buf = ReplayBuffer::new(file);
        buf.initialize().await.unwrap();
        let _ = buf.get_seeds().await.unwrap();
        
        buf.fill_pending().await.unwrap();
        
        loop {
            match buf.get_packet().await {
                Ok(packet) => {
                    println!("got packet at {:?}", Instant::now())
                },
                Err(ReplayError::Pending) => continue,
                Err(_) => break,
            }
        }
    })
}