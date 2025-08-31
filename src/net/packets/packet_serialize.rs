use crate::net::var_int::{var_int_size, write_var_int, VarInt};
use bytes::{BufMut, BytesMut};
use uuid::Uuid;

pub trait PacketSerializable {

    fn write_size(&self) -> usize;

    fn write(&self, buf: &mut BytesMut);
}

impl PacketSerializable for VarInt {
    fn write_size(&self) -> usize {
        var_int_size(self.0)
    }
    fn write(&self, buf: &mut BytesMut) {
        write_var_int(buf, self.0)
    }
}

impl PacketSerializable for bool {
    fn write_size(&self) -> usize {
        const { size_of::<Self>() }
    }
    fn write(&self, buf: &mut BytesMut) {
        buf.put_u8(*self as u8)
    }
}

impl PacketSerializable for u8 {
    fn write_size(&self) -> usize {
        const { size_of::<Self>() }
    }
    fn write(&self, buf: &mut BytesMut) {
        buf.put_u8(*self);
    }
}

impl PacketSerializable for i8 {
    fn write_size(&self) -> usize {
        const { size_of::<Self>() }
    }
    fn write(&self, buf: &mut BytesMut) {
        buf.put_i8(*self);
    }
}

impl PacketSerializable for u16 {
    fn write_size(&self) -> usize {
        const { size_of::<Self>() }
    }
    fn write(&self, buf: &mut BytesMut) {
        buf.put_u16(*self);
    }
}

impl PacketSerializable for i16 {
    fn write_size(&self) -> usize {
        const { size_of::<Self>() }
    }
    fn write(&self, buf: &mut BytesMut) {
        buf.put_i16(*self)
    }
}

impl PacketSerializable for u32 {
    fn write_size(&self) -> usize {
        const { size_of::<Self>() }
    }
    fn write(&self, buf: &mut BytesMut) {
        buf.put_u32(*self)
    }
}

impl PacketSerializable for i32 {
    fn write_size(&self) -> usize {
        const { size_of::<Self>() }
    }
    fn write(&self, buf: &mut BytesMut) {
        buf.put_i32(*self)
    }
}

impl PacketSerializable for u64 {
    fn write_size(&self) -> usize {
        const { size_of::<Self>() }
    }
    fn write(&self, buf: &mut BytesMut) {
        buf.put_u64(*self)
    }
}

impl PacketSerializable for i64 {
    fn write_size(&self) -> usize {
        const { size_of::<Self>() }
    }
    fn write(&self, buf: &mut BytesMut) {
        buf.put_i64(*self)
    }
}

impl PacketSerializable for f32 {
    fn write_size(&self) -> usize {
        const { size_of::<Self>() }
    }
    fn write(&self, buf: &mut BytesMut) {
        buf.put_f32(*self)
    }
}

impl PacketSerializable for f64 {
    fn write_size(&self) -> usize {
        const { size_of::<Self>() }
    }
    fn write(&self, buf: &mut BytesMut) {
        buf.put_f64(*self)
    }
}

impl PacketSerializable for &[u8] {
    fn write_size(&self) -> usize {
        self.len()
    }
    fn write(&self, buf: &mut BytesMut) {
        buf.put_slice(*self)
    }
}

impl<const N: usize> PacketSerializable for &[u8; N] {
    fn write_size(&self) -> usize {
        N
    }
    fn write(&self, buf: &mut BytesMut) {
        buf.put_slice(*self)
    }
}

impl PacketSerializable for &str {
    fn write_size(&self) -> usize {
        var_int_size(self.len() as i32) + self.len()
    }
    fn write(&self, buf: &mut BytesMut) {
        write_var_int(buf, self.len() as i32);
        buf.put_slice(self.as_bytes());
    }
}

impl PacketSerializable for String {
    fn write_size(&self) -> usize {
        var_int_size(self.len() as i32) + self.len()
    }
    fn write(&self, buf: &mut BytesMut) {
        write_var_int(buf, self.len() as i32);
        buf.put_slice(self.as_bytes());
    }
}

// I don't know if this is a good idea,
// maybe have a wrapper type that writes the length
impl<T: PacketSerializable> PacketSerializable for Vec<T> {
    fn write_size(&self) -> usize {
        let mut write_size = var_int_size(self.len() as i32);
        for entry in self {
            write_size += entry.write_size()
        }
        write_size
    }
    fn write(&self, buf: &mut BytesMut) {
        write_var_int(buf, self.len() as i32);
        for entry in self {
            entry.write(buf)
        }
    }
}

impl PacketSerializable for Uuid {
    fn write_size(&self) -> usize {
        const { size_of::<Self>() }
    }
    fn write(&self, buf: &mut BytesMut) {
        let bytes = self.as_u128();
        let most = (bytes >> 64) as i64;
        let least = bytes as i64;
        most.write(buf);
        least.write(buf);
    }
}