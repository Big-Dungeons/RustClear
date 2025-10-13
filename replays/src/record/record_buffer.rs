use std::path::PathBuf;

use chrono::{Local, format::{DelayedFormat, StrftimeItems}};
use tokio::{fs::{self, File}, io::{self, AsyncWriteExt, BufWriter}};

use crate::{VERSION, error::BufferError};

pub(super) struct RecordBuffer {
    buffer: Option<BufWriter<File>>,

    path: PathBuf,
    temp_path: PathBuf,
}

impl RecordBuffer {
    pub async fn open_with(path: PathBuf) -> Result<Self, io::Error> {
        let mut buffer = Self::new(path);
        buffer.initialize().await?;
        Ok(buffer)
    }
    
    pub(crate) fn get_writer<'a>(&'a mut self) -> Option<RecordWriter<'a>> {
        Some(RecordWriter::new(self.buffer.as_mut()?))
    }

    pub fn new(parent: PathBuf) -> Self {
        let path = parent.join("replay.tmp");
        Self {
            buffer: None,

            path: parent,
            temp_path: path,
        }
    }

    pub async fn initialize(&mut self) -> Result<(), io::Error> {
        if self.buffer.is_some() { panic!("Buffer should not be Some!")};
        fs::create_dir_all(&self.path).await?;
        if let Err(e) = fs::rename(&self.temp_path, &self.path.join(partial_name())).await {
            if e.kind() != io::ErrorKind::NotFound { return Err(e) }
        }
        Ok(())
    }

    pub async fn new_replay(&mut self) -> Result<(), io::Error> {
        if self.buffer.is_some() { return Err(io::Error::other(BufferError::AlreadyOpen)) };

        let file = File::create(&self.temp_path).await?;
        let mut buffer = BufWriter::new(file);

        let version_bytes = const {
            const BYTES: &[u8] = VERSION.as_bytes();
            const LEN: usize = BYTES.len();
            const LEN_BYTES: [u8; size_of::<u64>()] = (LEN as u64).to_be_bytes();
            let mut buffer = [0u8; size_of::<u64>() + LEN];
            let mut i = 0;
            while i < LEN_BYTES.len() {
                buffer[i] = LEN_BYTES[i];
                i += 1;
            }

            let mut j = 0;
            while j < LEN {
                buffer[LEN_BYTES.len() + j] = BYTES[j];
                j += 1;
            }

            buffer
        };

        buffer.write_all(&version_bytes).await?;
        self.buffer = Some(buffer);
        Ok(())
    }

    pub async fn write(&mut self, data: &[u8]) -> Result<(), io::Error> {
        if let Some(buffer) = self.buffer.as_mut() {
            buffer.write_all(data).await?;
        }
        Ok(())
    }

    pub async fn finish(&mut self) -> Result<PathBuf, io::Error> {
        if let Some(mut buffer) = self.buffer.take() {
            buffer.flush().await?;
        }
        let path = self.path.join(replay_name());
        fs::rename(&self.temp_path, &path).await?;

        Ok(path)
    }
}

pub struct RecordWriter<'a> {
    buffer: &'a mut BufWriter<File>,
}

impl<'a> RecordWriter<'a> {
    pub fn new(buffer: &'a mut BufWriter<File>) -> Self {
        Self { buffer }
    }
    
    pub async fn write(&mut self, data: &[u8]) -> Result<(), io::Error> {
        self.buffer.write_all(data).await
    }
}


fn now<'a>() -> DelayedFormat<StrftimeItems<'a>> {
    Local::now().format("%d.%m.%y_%H-%M-%S")
}

fn partial_name() -> String {
    format!("{}_partial_{}.rcrp", VERSION, now())
}

fn replay_name() -> String {
    format!("{}_replay_{}.rcrp", VERSION, now())
}

#[cfg(test)]
#[test]
fn test() {
    let version_bytes = const {
        const BYTES: &[u8] = VERSION.as_bytes();
        const LEN: usize = BYTES.len();
        const LEN_BYTES: [u8; size_of::<u64>()] = (LEN as u64).to_be_bytes();
        let mut buffer = [0u8; size_of::<u64>() + LEN];
        let mut i = 0;
        while i < LEN_BYTES.len() {
            buffer[i] = LEN_BYTES[i];
            i += 1;
        }

        let mut j = 0;
        while j < LEN {
            buffer[LEN_BYTES.len() + j] = BYTES[j];
            j += 1;
        }

        buffer
    };
    println!("bytes: {:?}", version_bytes)
}
