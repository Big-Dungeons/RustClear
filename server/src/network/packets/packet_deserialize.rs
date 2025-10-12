use crate::{network::binary::var_int::{read_var_int, VarInt}, utils::get_vec};
use anyhow::{bail, Context};
use bytes::Buf;
use fstr::FString;

pub trait PacketDeserializable: Sized {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self>;
}

impl PacketDeserializable for u8 {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        if !buffer.has_remaining() {
            bail!("buffer doesn't contain enough bytes")
        }
        let byte = buffer.chunk()[0];
        buffer.advance(1);
        Ok(byte)
    }
}

impl PacketDeserializable for bool {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        Ok(u8::read(buffer)? != 0)
    }
}


impl PacketDeserializable for i8 {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        Ok(u8::read(buffer)? as i8)
    }
}

impl PacketDeserializable for u16 {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        const SIZE: usize = size_of::<u16>();

        if buffer.remaining() < SIZE {
            bail!("buffer doesn't contain enough bytes")
        }

        let value = unsafe {
            // should be safe since buffer size is ensured
            u16::from_be_bytes(*(&buffer.chunk()[..SIZE] as *const _ as *const [u8; 2]))
        };
        buffer.advance(SIZE);
        Ok(value)
    }
}

impl PacketDeserializable for i16 {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        const SIZE: usize = size_of::<i16>();

        if buffer.remaining() < SIZE {
            bail!("buffer doesn't contain enough bytes")
        }

        let value = unsafe {
            // should be safe since buffer size is ensured
            i16::from_be_bytes(*(&buffer.chunk()[..SIZE] as *const _ as *const [u8; 2]))
        };
        buffer.advance(SIZE);
        Ok(value)
    }
}

impl PacketDeserializable for u32 {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        const SIZE: usize = size_of::<u32>();

        if buffer.remaining() < SIZE {
            bail!("buffer doesn't contain enough bytes")
        }

        let value = unsafe {
            // should be safe since buffer size is ensured
            u32::from_be_bytes(*(&buffer.chunk()[..SIZE] as *const _ as *const [u8; 4]))
        };
        buffer.advance(SIZE);
        Ok(value)
    }
}

impl PacketDeserializable for i32 {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        const SIZE: usize = size_of::<i32>();

        if buffer.remaining() < SIZE {
            bail!("buffer doesn't contain enough bytes")
        }

        let value = unsafe {
            // should be safe since buffer size is ensured
            i32::from_be_bytes(*(&buffer.chunk()[..SIZE] as *const _ as *const [u8; 4]))
        };
        buffer.advance(SIZE);
        Ok(value)
    }
}

impl PacketDeserializable for u64 {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        const SIZE: usize = size_of::<u64>();

        if buffer.remaining() < SIZE {
            bail!("buffer doesn't contain enough bytes")
        }

        let value = unsafe {
            // should be safe since buffer size is ensured
            u64::from_be_bytes(*(&buffer.chunk()[..SIZE] as *const _ as *const [u8; 8]))
        };
        buffer.advance(SIZE);
        Ok(value)
    }
}

impl PacketDeserializable for i64 {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        const SIZE: usize = size_of::<i64>();

        if buffer.remaining() < SIZE {
            bail!("buffer doesn't contain enough bytes")
        }

        let value = unsafe {
            // should be safe since buffer size is ensured
            i64::from_be_bytes(*(&buffer.chunk()[..SIZE] as *const _ as *const [u8; 8]))
        };
        buffer.advance(SIZE);
        Ok(value)
    }
}

impl PacketDeserializable for f32 {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        const SIZE: usize = size_of::<f32>();

        if buffer.remaining() < SIZE {
            bail!("buffer doesn't contain enough bytes")
        }

        let value = unsafe {
            // should be safe since buffer size is ensured
            f32::from_be_bytes(*(&buffer.chunk()[..SIZE] as *const _ as *const [u8; 4]))
        };
        buffer.advance(SIZE);
        Ok(value)
    }
}

impl PacketDeserializable for f64 {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        const SIZE: usize = size_of::<f64>();

        if buffer.remaining() < SIZE {
            bail!("buffer doesn't contain enough bytes")
        }

        let value = unsafe {
            // should be safe since buffer size is ensured
            f64::from_be_bytes(*(&buffer.chunk()[..SIZE] as *const _ as *const [u8; 8]))
        };
        buffer.advance(SIZE);
        Ok(value)
    }
}

impl PacketDeserializable for VarInt {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        match read_var_int(buffer) {
            Some(int) => Ok(VarInt(int)),
            None => bail!("Failed to read VarInt"),
        }
    }
}

impl PacketDeserializable for String {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        let len = read_var_int(buffer).context("failed to read string length")? as usize;
        if len > 32767 {
            bail!("String too long. {:?} > 32767", len);
        }
        match String::from_utf8(get_vec(buffer, len)) {
            Ok(string) => Ok(string),
            Err(_) => bail!("failed to read string"),
        }
    }
}

impl PacketDeserializable for FString {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        let len = read_var_int(buffer).context("failed to read string length")? as usize;
        if len > 32767 {
            bail!("String too long. {:?} > 32767", len);
        }
        let str = Self::from_bytes(&buffer.chunk()[..len])?;
        buffer.advance(len);
        Ok(str)
    }
}