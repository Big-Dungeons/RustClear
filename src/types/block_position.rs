use crate::network::packets::packet_deserialize::PacketDeserializable;
use crate::network::packets::packet_serialize::PacketSerializable;
use bytes::{Buf, BytesMut};
use glam::{ivec3, IVec3};
use std::ops::{Deref, DerefMut};

/// new-type wrapper for IVec3,
/// which implements PacketSerializable and PacketDeserializable.
///
/// Note: You should try to always directly use IVec3
pub struct BlockPosition(pub IVec3);

const XZ_BITS: i32 = 26;
const Y_BITS: i32 = 12;

const X_SHIFT: i32 = 38;
const Y_SHIFT: i32 = 26;

const XZ_MASK: i64 = 0x3FFFFFF;
const Y_MASK: i64 = 0xFFF;

impl PacketSerializable for BlockPosition {
    fn write_size(&self) -> usize {
        const { size_of::<i64>() }
    }
    fn write(&self, buf: &mut BytesMut) {
        let long: i64 = (self.0.x as i64 & XZ_MASK) << X_SHIFT
            | (self.0.y as i64 & Y_MASK) << Y_SHIFT
            | (self.0.z as i64 & XZ_MASK);
        long.write(buf);
    }
}

impl PacketDeserializable for BlockPosition {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        let long = buffer.get_i64();
        Ok(BlockPosition(ivec3(
            (long << (64 - X_SHIFT - XZ_BITS) >> (64 - XZ_BITS)) as i32,
            (long << (64 - Y_SHIFT - Y_BITS) >> (64 - Y_BITS)) as i32,
            (long << (64 - XZ_BITS) >> (64 - XZ_BITS)) as i32,
        )))
    }
}

impl Deref for BlockPosition {
    type Target = IVec3;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for BlockPosition {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
