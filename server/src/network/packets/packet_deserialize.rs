use crate::{network::binary::var_int::{read_var_int, VarInt}, utils::get_vec};
use anyhow::bail;
use bytes::Buf;
use fstr::FString;

pub trait PacketDeserializable: Sized {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self>;
}

impl PacketDeserializable for u8 {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        Ok(buffer.try_get_u8()?)
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
        Ok(buffer.try_get_u16()?)
    }
}

impl PacketDeserializable for i16 {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        Ok(buffer.try_get_i16()?)
    }
}

impl PacketDeserializable for u32 {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        Ok(buffer.try_get_u32()?)
    }
}

impl PacketDeserializable for i32 {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        Ok(buffer.try_get_i32()?)
    }
}

impl PacketDeserializable for u64 {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        Ok(buffer.try_get_u64()?)
    }
}

impl PacketDeserializable for i64 {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        Ok(buffer.try_get_i64()?)
    }
}

impl PacketDeserializable for f32 {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        Ok(buffer.try_get_f32()?)
    }
}

impl PacketDeserializable for f64 {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        Ok(buffer.try_get_f64()?)
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

impl PacketDeserializable for FString {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        let length = *VarInt::read(buffer)? as usize;
        if buffer.remaining() < length {
            bail!("not enough bytes for string")
        }
        if length > 32767 {
            bail!("String too long. {:?} > 32767", length);
        }
        let str = Self::from_bytes(&buffer.chunk()[..length])?;
        buffer.advance(length);
        Ok(str)
    }
}