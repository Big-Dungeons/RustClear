use std::{collections::VecDeque, time::Instant};

use anyhow::{Context, anyhow, bail};
use bytes::{Buf, BytesMut};
use fstr::FString;
use tokio::{fs::File, io::AsyncReadExt, time::sleep_until};

use crate::{network::binary::var_int::peek_var_int, replays::{error::ReplayError, replay_packet::ReplayPacket}};

pub struct ReplayBuffer {
    reader: File,

    buffer: BytesMut,
    packets: VecDeque<ReplayPacket>,
    pending: usize,

    pub start: Instant
}

impl ReplayBuffer {
    pub fn new(file: File) -> Self {
        Self {
            reader: file,

            buffer: BytesMut::with_capacity(8 * 1024),
            packets: VecDeque::with_capacity(30),
            pending: 30,

            start: Instant::now(),
        }
    }

    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        self.reader.read_buf(&mut self.buffer).await?;
        if self.buffer.remaining() < size_of::<u64>() {
            bail!("Buffer too short to read version length!")
        }
        let length = self.buffer.get_u64() as usize;
        if self.buffer.remaining() < length {
            bail!("Buffer too short to read version!")
        }

        let version = self.buffer.split_to(length);
        println!("{}", version.len());
        let str = str::from_utf8(&version)?;
        println!("version: {}", str);

        Ok(())
    }
    pub async fn get_seeds(&mut self) -> anyhow::Result<(FString, u64)> {
        const SEED_LEN: usize = 132usize;
        if self.buffer.remaining() < SEED_LEN {
            bail!("Not enough bytes in buffer!");
        };

        let bytes = self.buffer.split_to(SEED_LEN);
        let seed = FString::from_bytes(&bytes)?;
        let rng_seed = self.buffer.get_u64();
        Ok((seed, rng_seed))
    }

    /// this should be cancel safe (hopefully)
    pub async fn fill_pending(&mut self) -> Result<(), ReplayError> {
        if self.pending == 0 {
            return Ok(());
        }

        loop {
            while self.pending > 0 {
                let Some((size, var_int_size)) = peek_var_int(&mut self.buffer) else { break };
                if self.buffer.remaining() < size as usize + var_int_size { break }
                self.buffer.advance(var_int_size);
                let Some(packet) = ReplayPacket::deserialize(&mut self.buffer) else { break };
                self.packets.push_back(packet);
                self.pending -= 1;
            }

            if self.pending == 0 { return Ok(()) }

            let n = self.reader.read_buf(&mut self.buffer).await?;

            if n == 0 {
                return Err(ReplayError::EndOfFile)
            }
        }
    }

    /// this should be cancel safe
    pub async fn get_packet(&mut self) -> Result<ReplayPacket, ReplayError> {
        let Some(packet) = self.packets.front() else {
            self.fill_pending().await?;
            return Err(ReplayError::Pending)
        };
        sleep_until(tokio::time::Instant::from_std(self.start + packet.since_start)).await;
        self.pending = self.pending.saturating_add(1);

        self.fill_pending().await?;

        self.packets.pop_front().ok_or(ReplayError::Pending)
    }
}

#[test]
fn test() {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    rt.block_on(async {
        let file = tokio::fs::File::open("C:\\Users\\Michael\\RustroverProjects\\RustClear\\replays\\0.1.0_partial_09.10.25_10-57-06.rcrp").await.unwrap();
        let mut buf = ReplayBuffer::new(file);
        buf.initialize().await.unwrap()
    })
}
