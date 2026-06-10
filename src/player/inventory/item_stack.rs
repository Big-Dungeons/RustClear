use crate::network::packets::packet_deserializable::PacketDeserializable;
use crate::network::packets::packet_serializable::PacketSerializable;
use crate::network::protocol::nbt::NBT;
use bytes::{Buf, Bytes, BytesMut};
use macros::PacketSerializable;

#[derive(Debug, Clone, PartialEq, PacketSerializable)]
pub struct ItemStack {
    pub item: i16,
    pub stack_size: i8,
    pub metadata: i16,
    pub tag_compound: Option<NBT>,
}

impl ItemStack {
    pub fn new() -> Self {
        Self {
            item: 0,
            stack_size: 1,
            metadata: 0,
            tag_compound: None,
        }
    }
}

impl PacketSerializable for Option<ItemStack> {
    fn write_size(&self) -> usize {
        match self {
            Some(item_stack) => item_stack.write_size(),
            None => size_of::<i16>()
        }
    }
    fn write(&self, buf: &mut BytesMut) {
        match self {
            Some(item_stack) => item_stack.write(buf),
            None => (-1i16).write(buf)
        };
    }
}

impl PacketDeserializable for Option<ItemStack> {
    fn read(buffer: &mut Bytes) -> anyhow::Result<Self> {
        let id = buffer.get_i16();
        if id >= 0 {
            let item_stack = ItemStack {
                item: id,
                stack_size: buffer.get_i8(),
                metadata: buffer.get_i16(),
                tag_compound: PacketDeserializable::read(buffer)?,
            };
            return Ok(Some(item_stack));
        }
        Ok(None)
    }
}