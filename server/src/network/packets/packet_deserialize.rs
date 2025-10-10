use crate::{network::binary::var_int::{read_var_int, VarInt}, utils::get_vec};
use anyhow::{bail, Context};
use bytes::Buf;

pub trait PacketDeserializable: Sized {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self>;
}

impl PacketDeserializable for VarInt {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        match read_var_int(buffer) {
            Some(int) => Ok(VarInt(int)),
            None => bail!("Failed to read VarInt"),
        }
    }
}

impl PacketDeserializable for bool {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        Ok(buffer.get_u8() != 0)
    }
}

impl PacketDeserializable for u8 {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        Ok(buffer.get_u8())
    }
}

impl PacketDeserializable for i8 {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        Ok(buffer.get_i8())
    }
}

impl PacketDeserializable for u16 {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        Ok(buffer.get_u16())
    }
}

impl PacketDeserializable for i16 {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        Ok(buffer.get_i16())
    }
}

impl PacketDeserializable for u32 {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        Ok(buffer.get_u32())
    }
}

impl PacketDeserializable for i32 {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        Ok(buffer.get_i32())
    }
}

impl PacketDeserializable for u64 {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        Ok(buffer.get_u64())
    }
}

impl PacketDeserializable for i64 {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        Ok(buffer.get_i64())
    }
}

impl PacketDeserializable for f32 {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        Ok(buffer.get_f32())
    }
}

impl PacketDeserializable for f64 {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        Ok(buffer.get_f64())
    }
}

impl PacketDeserializable for String {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        let len = read_var_int(buffer).context("failed to read string length")? as usize;
        if len > 32767 {
            bail!("String too long. {:?} > 32767", len);
        }
        match String::from_utf8(get_vec(buffer, len)) { // this is actually the same amount of copies as the old since bytesmut::to_vec does a deep copy??
            Ok(string) => Ok(string),
            Err(_) => bail!("failed to read string"),
        }
    }
}