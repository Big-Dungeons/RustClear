use std::{path::Path, sync::OnceLock, time::Instant};

use fstr::Fstr;
use tokio::{io, sync::mpsc::{UnboundedSender, unbounded_channel}};

use crate::replays::{record::{record_buffer::RecordBuffer, record_message::RecordMessage}, replay_packet::ReplayPacket};

static RECORDHANDLE: OnceLock<UnboundedSender<RecordMessage>> = OnceLock::new();
pub fn get_handle() -> &'static UnboundedSender<RecordMessage> {
    RECORDHANDLE.get().unwrap()
}

pub async fn run_record_thread() {
    let (tx, mut rx) = unbounded_channel();
    RECORDHANDLE.set(tx).unwrap();
    let mut start: Option<Instant> = None;
    let mut buffer: RecordBuffer = RecordBuffer::open_with("replays").await.unwrap();

    while let Some(message) = rx.recv().await {
        match message {
            RecordMessage::Start { seed, rng_seed, at } => {
                start = Some(at);

                buffer.new_replay().await.unwrap();
                write_init(&mut buffer, seed.as_str(), rng_seed).await.unwrap();
            }
            RecordMessage::Record { received, game_profile, packet } => {
                let Some(start) = start else { continue };
                let since_start = received.duration_since(start);
                let packet = ReplayPacket { game_profile, since_start, packet, }.serialize();

                buffer.write(&packet).await.unwrap();
            }
            RecordMessage::Save => {
                buffer.finish().await.unwrap();
            }
        }
    }

    println!("Record thread died");
}

async fn write_init(buffer: &mut RecordBuffer, seed: &str, rng_seed: u64) -> Result<(), io::Error> {
    buffer.write(seed.as_bytes()).await?; // fine if fixed length?
    buffer.write(&rng_seed.to_be_bytes()).await?;
    Ok(())
}
