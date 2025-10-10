use crate::network::binary::nbt::deserialize::deserialize_nbt;
use crate::network::binary::nbt::serialize::{nbt_write_size, serialize_nbt};
use crate::network::binary::nbt::NBT;
use crate::network::packets::packet_deserialize::PacketDeserializable;
use crate::network::packets::packet_serialize::PacketSerializable;
use bytes::{Buf, BytesMut};

#[derive(Debug, Clone, PartialEq)]
pub struct ItemStack {
    pub item: i16,
    pub stack_size: i8,
    pub metadata: i16,
    pub tag_compound: Option<NBT>,
}

impl PacketSerializable for ItemStack {
    fn write_size(&self) -> usize {
        self.item.write_size() +
            self.stack_size.write_size() +
            self.metadata.write_size() +
            match &self.tag_compound {
                None => const { size_of::<u8>() },
                Some(nbt) => nbt_write_size(nbt),
            }
    }
    fn write(&self, buf: &mut BytesMut) {
        self.item.write(buf);
        self.stack_size.write(buf);
        self.metadata.write(buf);

        match &self.tag_compound {
            None => 0u8.write(buf),
            Some(nbt) => buf.extend(serialize_nbt(nbt)),
        }
    }
}

impl PacketSerializable for Option<ItemStack> {
    fn write_size(&self) -> usize {
        match self {
            Some(item_stack) => item_stack.write_size(),
            None => const { size_of::<i16>() }
        }
    }
    fn write(&self, buf: &mut BytesMut) {
        if let Some(item_stack) = self {
            item_stack.write(buf)
        } else {
            (-1i16).write(buf)
        }
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
                tag_compound: deserialize_nbt(buffer),
            };
            return Ok(Some(item_stack));
        }
        Ok(None)
    }
}