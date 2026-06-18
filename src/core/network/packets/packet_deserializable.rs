use crate::core::network::protocol::var_int::VarInt;
use anyhow::bail;
use bytes::{Buf, Bytes};

pub trait PacketDeserializable: Sized {
    fn read(buffer: &mut Bytes) -> anyhow::Result<Self>;
}

impl PacketDeserializable for u8 {
    fn read(buffer: &mut Bytes) -> anyhow::Result<Self> {
        Ok(buffer.try_get_u8()?)
    }
}

impl PacketDeserializable for bool {
    fn read(buffer: &mut Bytes) -> anyhow::Result<Self> {
        Ok(u8::read(buffer)? != 0)
    }
}


impl PacketDeserializable for i8 {
    fn read(buffer: &mut Bytes) -> anyhow::Result<Self> {
        Ok(u8::read(buffer)? as i8)
    }
}

impl PacketDeserializable for u16 {
    fn read(buffer: &mut Bytes) -> anyhow::Result<Self> {
        Ok(buffer.try_get_u16()?)
    }
}

impl PacketDeserializable for i16 {
    fn read(buffer: &mut Bytes) -> anyhow::Result<Self> {
        Ok(buffer.try_get_i16()?)
    }
}

impl PacketDeserializable for u32 {
    fn read(buffer: &mut Bytes) -> anyhow::Result<Self> {
        Ok(buffer.try_get_u32()?)
    }
}

impl PacketDeserializable for i32 {
    fn read(buffer: &mut Bytes) -> anyhow::Result<Self> {
        Ok(buffer.try_get_i32()?)
    }
}

impl PacketDeserializable for u64 {
    fn read(buffer: &mut Bytes) -> anyhow::Result<Self> {
        Ok(buffer.try_get_u64()?)
    }
}

impl PacketDeserializable for i64 {
    fn read(buffer: &mut Bytes) -> anyhow::Result<Self> {
        Ok(buffer.try_get_i64()?)
    }
}

impl PacketDeserializable for f32 {
    fn read(buffer: &mut Bytes) -> anyhow::Result<Self> {
        Ok(buffer.try_get_f32()?)
    }
}

impl PacketDeserializable for f64 {
    fn read(buffer: &mut Bytes) -> anyhow::Result<Self> {
        Ok(buffer.try_get_f64()?)
    }
}

impl PacketDeserializable for String {
    fn read(buffer: &mut Bytes) -> anyhow::Result<Self> {
        let length = *VarInt::read(buffer)? as usize;
        if buffer.remaining() < length {
            bail!("not enough bytes for string")
        }
        if length > 32767 {
            bail!("String too long. {:?} > 32767", length);
        }
        match String::from_utf8(get_vec(buffer, length)) {
            Ok(string) => Ok(string),
            Err(_) => bail!("failed to read string"),
        }
    }
}

pub fn get_vec(buf: &mut Bytes, take: usize) -> Vec<u8> {
    let len = take.min(buf.remaining());
    let mut data = vec![0u8; len];
    buf.copy_to_slice(&mut data);
    data
}