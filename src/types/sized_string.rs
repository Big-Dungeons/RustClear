use crate::network::binary::var_int::{read_var_int, var_int_size, write_var_int};
use crate::network::packets::packet_deserialize::PacketDeserializable;
use crate::network::packets::packet_serialize::PacketSerializable;
use anyhow::{bail, Context};
use bytes::{BufMut, BytesMut};
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;

#[derive(Debug)]
pub struct SizedString<const S: usize> {
    length: usize,
    data: [u8; S]
}

impl<const S: usize> SizedString<S> {

    pub fn truncated(str: &str) -> Self {
        let mut data = [0u8; S];
        let bytes = str.as_bytes();
        let len = bytes.len().min(S);
        data[..len].copy_from_slice(&bytes[..len]);
        Self { length: len, data }
    }

    fn to_string(&self) -> String {
        unsafe {
            String::from_utf8_unchecked(self.data[..self.length].to_vec())
        }
    }
}

impl<const S: usize> Display for SizedString<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.deref(), f)
    }
}

impl<const S: usize> Into<SizedString<S>> for &str {
    fn into(self) -> SizedString<S> {
        SizedString::truncated(self)
    }
}

impl<const S: usize> Into<SizedString<S>> for String {
    fn into(self) -> SizedString<S> {
        SizedString::truncated(self.as_str())
    }
}

impl<const S: usize> PacketSerializable for SizedString<S> {
    fn write_size(&self) -> usize {
        var_int_size(self.length as i32) + self.length
    }
    fn write(&self, buf: &mut BytesMut) {
        write_var_int(buf, self.length as i32);
        buf.put_slice(&self.data[..self.length])
    }
}

impl<const S : usize> PacketDeserializable for SizedString<S> {
    fn read(buffer: &mut BytesMut) -> anyhow::Result<Self> {
        let len = read_var_int(buffer).context("failed to read string length")? as usize;
        if len > S {
            bail!("String too long. {:?} > {}", len, S);
        }
        let mut data = [0u8; S];
        data[..len].copy_from_slice(&buffer.split_to(len));
        Ok(SizedString {
            length: len,
            data,
        })
    }
}

impl<const S : usize> Deref for SizedString<S> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        // should be fine, since it shouldn't be initialized without using a str
        unsafe { std::str::from_utf8_unchecked(&self.data[..self.length]) }
    }
}