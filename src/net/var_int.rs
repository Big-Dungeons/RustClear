use bytes::{Buf, BufMut, BytesMut};

#[derive(PartialEq, Eq)]
pub struct VarInt(pub i32);

pub fn peek_var_int(buf: &BytesMut) -> Option<(i32, usize)> {
    let mut num_read = 0;
    let mut result = 0;

    loop {
        if num_read >= 5 || num_read >= buf.len() {
            return None;
        }

        let byte = buf[num_read];
        let value = (byte & 0x7f) as i32;
        result |= value << (7 * num_read);
        num_read += 1;

        if byte & 0x80 == 0 {
            break;
        }
    }

    Some((result, num_read))
}

pub fn read_var_int(buf: &mut BytesMut) -> Option<i32> {
    let (int, len) = peek_var_int(buf)?;
    buf.advance(len);
    Some(int)
}

pub fn write_var_int(buf: &mut BytesMut, mut value: i32) {
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

pub fn var_int_size(value: i32) -> usize {
    if value == 0 {
        1
    } else {
        (31 - value.leading_zeros() as usize) / 7 + 1
    } 
}