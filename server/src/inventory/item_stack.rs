use crate::network::binary::nbt::NBT;
use crate::network::packets::packet_deserialize::PacketDeserializable;
use crate::network::packets::packet_serialize::PacketSerializable;
use blocks::packet_serializable;
use bytes::{Buf, BytesMut};

packet_serializable! {
    #[derive(Debug, Clone, PartialEq)]
    pub struct ItemStack {
        pub item: i16,
        pub stack_size: i8,
        pub metadata: i16,
        pub tag_compound: Option<NBT>,
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
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
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