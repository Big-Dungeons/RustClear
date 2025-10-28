use bytes::{Buf, BufMut};
use std::ops::Deref;

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq)]
pub struct VarInt(pub i32);

impl Deref for VarInt {
    type Target = i32;
    fn deref(&self) -> &i32 {
        &self.0
    }
}

pub fn peek_var_int(buf: &impl Buf) -> Option<(i32, usize)> {
    let mut num_read = 0;
    let mut result = 0;

    loop {
        if num_read >= 5 || num_read >= buf.remaining() {
            return None;
        }

        let byte = buf.chunk()[num_read];
        let value = (byte & 0x7f) as i32;
        result |= value << (7 * num_read);
        num_read += 1;

        if byte & 0x80 == 0 {
            break;
        }
    }

    Some((result, num_read))
}

pub fn read_var_int(buf: &mut impl Buf) -> Option<i32> {
    let (int, len) = peek_var_int(buf)?;
    buf.advance(len);
    Some(int)
}

pub fn write_var_int(buf: &mut impl BufMut, mut value: i32) {
    loop {
        if (value & !0x7F) == 0 {
            // buf.reserve(1);
            buf.put_u8(value as u8);
            return;
        }
        // buf.reserve(1);
        buf.put_u8(((value & 0x7F) | 0x80) as u8);
        value >>= 7;
    }
}

pub const fn var_int_size(value: i32) -> usize {
    if value == 0 {
        1
    } else {
        (31 - value.leading_zeros() as usize) / 7 + 1
    } 
}