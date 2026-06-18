use crate::core::network::packets::packet_deserializable::PacketDeserializable;
use crate::core::network::packets::packet_serializable::PacketSerializable;
use anyhow::bail;
use bevy::prelude::Deref;
use bytes::{Buf, BufMut, Bytes, BytesMut};

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, Deref)]
pub struct VarInt(pub i32);

impl PacketDeserializable for VarInt {
    fn read(buffer: &mut Bytes) -> anyhow::Result<Self> {
        if buffer.is_empty() {
            bail!("not enough bytes")
        }
        let Some((num, len)) = peek_var_int(buffer) else {
            bail!("failed to read var int")
        };
        buffer.advance(len);
        Ok(num)
    }
}

impl PacketSerializable for VarInt {
    fn write_size(&self) -> usize {
        var_int_size(**self)
    }
    fn write(&self, buf: &mut BytesMut) {
        write_var_int(buf, **self)
    }
}

pub fn peek_var_int(buffer: &impl Buf) -> Option<(VarInt, usize)> {
    let chunk = buffer.chunk();
    if chunk.len() >= 8 {
        let (num, size) = unsafe { decode_unchecked(chunk.as_ptr()) };
        Some((VarInt(num), size))
    } else {
        let mut data = [0u8; 8];
        data[..chunk.len()].copy_from_slice(chunk);
        let (num, size) = unsafe { decode_unchecked(data.as_ptr()) };

        if size > chunk.len() {
            None
        } else {
            Some((VarInt(num), size))
        }
    }
}

pub fn read_var_int(buf: &mut Bytes) -> Option<VarInt> {
    let (int, len) = peek_var_int(buf)?;
    buf.advance(len);
    Some(int)
}

pub fn var_int_size(num: i32) -> usize {
    if num == 0 {
        1
    } else {
        (31 - num.leading_zeros() as usize) / 7 + 1
    }
}

// based on: https://github.com/as-com/varint-simd/blob/master/src/encode/mod.rs#L71
pub fn write_var_int(buf: &mut BytesMut, num: i32) {
    let x = num as u32 as u64;
    let stage1 = (x & 0x000000000000007f)
        | ((x & 0x0000000000003f80) << 1)
        | ((x & 0x00000000001fc000) << 2)
        | ((x & 0x000000000fe00000) << 3)
        | ((x & 0x00000000f0000000) << 4);

    let leading = stage1.leading_zeros();
    let unused_bytes = (leading - 1) / 8;
    let bytes_needed = 8 - unused_bytes;
    let msbs = 0x8080808080808080;
    let msbmask = 0xFFFFFFFFFFFFFFFF >> ((8 - bytes_needed + 1) * 8 - 1);
    let merged = stage1 | (msbs & msbmask);

    let bytes: [u8; 8] = u64::to_ne_bytes(merged);
    buf.put_slice(&bytes[..bytes_needed as usize]);
}

// based on: https://github.com/as-com/varint-simd/blob/master/src/decode/mod.rs#L141
#[inline(always)]
unsafe fn decode_unchecked(bytes: *const u8) -> (i32, usize) {
    let b = unsafe { bytes.cast::<u64>().read_unaligned() };
    let msbs = !b & !0x7f7f7f7f7f7f7f7f;
    let len = msbs.trailing_zeros() + 1;
    let varint_part = b & (msbs ^ msbs.wrapping_sub(1));

    let num = (varint_part & 0x7f)
        | ((varint_part & (0x7f << 8)) >> 1)
        | ((varint_part & (0x7f << 16)) >> 2)
        | ((varint_part & (0x7f << 24)) >> 3)
        | ((varint_part & (0x7f << 32)) >> 4);
    (num as u32 as i32, (len / 8) as usize)
}