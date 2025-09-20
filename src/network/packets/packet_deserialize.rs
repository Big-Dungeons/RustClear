use crate::network::binary::var_int::{read_var_int, VarInt};
use anyhow::bail;
use bytes::{Buf, BytesMut};

pub trait PacketDeserializable : Sized {
    fn read(buffer: &mut BytesMut) -> anyhow::Result<Self>;
}

impl PacketDeserializable for VarInt {
    fn read(buffer: &mut BytesMut) -> anyhow::Result<Self> {
        match read_var_int(buffer) {
            Some(int) => Ok(VarInt(int)),
            None => bail!("Failed to read VarInt"),
        }
    }
}

impl PacketDeserializable for bool {
    fn read(buffer: &mut BytesMut) -> anyhow::Result<Self> {
        Ok(buffer.get_u8() != 0)
    }
}

impl PacketDeserializable for u8 {
    fn read(buffer: &mut BytesMut) -> anyhow::Result<Self> {
        Ok(buffer.get_u8())
    }
}

impl PacketDeserializable for i8 {
    fn read(buffer: &mut BytesMut) -> anyhow::Result<Self> {
        Ok(buffer.get_i8())
    }
}

impl PacketDeserializable for u16 {
    fn read(buffer: &mut BytesMut) -> anyhow::Result<Self> {
        Ok(buffer.get_u16())
    }
}

impl PacketDeserializable for i16 {
    fn read(buffer: &mut BytesMut) -> anyhow::Result<Self> {
        Ok(buffer.get_i16())
    }
}

impl PacketDeserializable for u32 {
    fn read(buffer: &mut BytesMut) -> anyhow::Result<Self> {
        Ok(buffer.get_u32())
    }
}

impl PacketDeserializable for i32 {
    fn read(buffer: &mut BytesMut) -> anyhow::Result<Self> {
        Ok(buffer.get_i32())
    }
}

impl PacketDeserializable for u64 {
    fn read(buffer: &mut BytesMut) -> anyhow::Result<Self> {
        Ok(buffer.get_u64())
    }
}

impl PacketDeserializable for i64 {
    fn read(buffer: &mut BytesMut) -> anyhow::Result<Self> {
        Ok(buffer.get_i64())
    }
}

impl PacketDeserializable for f32 {
    fn read(buffer: &mut BytesMut) -> anyhow::Result<Self> {
        Ok(buffer.get_f32())
    }
}

impl PacketDeserializable for f64 {
    fn read(buffer: &mut BytesMut) -> anyhow::Result<Self> {
        Ok(buffer.get_f64())
    }
}