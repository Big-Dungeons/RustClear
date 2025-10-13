use std::{collections::VecDeque, time::Instant};

use bytes::{Buf, BytesMut};
use tokio::{fs::File, io::{self, AsyncReadExt}, time::sleep_until};

use crate::{VERSION, error::BufferError, replay_packet::ReplayPacket};

#[derive(Debug)]
pub struct ReplayBuffer {
    reader: File,

    buffer: BytesMut,
    end_of_file: bool,
    packets: VecDeque<ReplayPacket>,
    pending: usize,
    
    pub start: Instant,
}

impl ReplayBuffer {
    pub fn new(file: File) -> Self {
        Self {
            reader: file,

            buffer: BytesMut::with_capacity(8 * 1024),
            end_of_file: false,
            packets: VecDeque::with_capacity(30),
            pending: 30,

            start: Instant::now(),
        }
    }

    pub async fn initialize(&mut self) -> Result<(), BufferError> {
        self.reader.read_buf(&mut self.buffer).await?;
        let length = self.buffer.try_get_u64()? as usize;
        if self.buffer.remaining() < length {
            return Err(BufferError::Pending)
        }

        let version_bytes = self.buffer.split_to(length);
        let version = str::from_utf8(&version_bytes).map_err(|e| io::Error::other(e))?;
        
        if version != VERSION {
            eprintln!("Versions dont match!");
        }

        Ok(())
    }
    
    /// This will fill the buffer if the error is ReplayError::Pending, returning any other errors.
    /// 
    /// # Example:
    /// ```
    /// let test: Result<FString, ReplayError> = self.with_buffer(|buf| {
    ///     if buf.remaining() < 12 {
    ///         Err(ReplayError::Pending)
    ///     } else {
    ///         let bytes = buf.split_to(12);
    ///         let str = FString::from_bytes(&bytes)?;
    ///         Ok(str)
    ///     }
    /// }).await;
    /// ```
    pub async fn with_buffer<T>(&mut self, f: fn(&mut BytesMut) -> Result<T, BufferError>) -> Result<T, BufferError> {
        loop {
            let res = f(&mut self.buffer);
            let Err(BufferError::Pending) = res else { return res };
            let read = self.reader.read_buf(&mut self.buffer).await?;
            if read == 0 {
                self.end_of_file = true;
                return Err(BufferError::EndOfFile)
            }
            continue
        }
    }

    /// this should be cancel safe (hopefully)
    pub async fn fill_pending(&mut self) -> Result<(), BufferError> {
        while self.pending > 0 {
            if self.buffer.remaining() < ReplayPacket::LEN_SIZE { break };
            let size = (&self.buffer.chunk()[..ReplayPacket::LEN_SIZE]).get_u32() as usize; // we need to peek this u32, which is why we get a reference and then get from the reference instead of the buffer directly
            if self.buffer.remaining() < ReplayPacket::LEN_SIZE + size { break };
            self.buffer.advance(ReplayPacket::LEN_SIZE);
            self.packets.push_back(ReplayPacket::deserialize(&mut self.buffer));
            self.pending -= 1;
        }
        
        if self.pending == 0 { return Ok(()) }
        
        if !self.end_of_file {
            let read: usize = self.reader.read_buf(&mut self.buffer).await?;
            if read == 0 { self.end_of_file = true; }
        }
        
        if self.packets.is_empty() {
            if self.end_of_file { return Err(BufferError::EndOfFile) }
            else { return Err(BufferError::Pending) }
        }
        
        return Ok(())
    }

    /// this should be cancel safe
    pub async fn get_packet(&mut self) -> Result<ReplayPacket, BufferError> {
        self.fill_pending().await?;
        let packet = self.packets.front().expect("fill_pending() should error with pending or EOF if theres no packets available.");
        sleep_until(tokio::time::Instant::from_std(self.start + packet.since_start)).await;
        // we grab packet again here instead of popping it before to prevent consuming it before the .await, which could be cancelled.
        let packet = self.packets.pop_front().unwrap();
        self.pending = self.pending.saturating_add(1);
        Ok(packet)
    }
}