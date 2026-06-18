use crate::core::network::packets::packet_deserializable::{get_vec, PacketDeserializable};
use crate::core::network::packets::packet_serializable::PacketSerializable;
use crate::core::network::protocol::var_int::{var_int_size, write_var_int, VarInt};
use anyhow::bail;
use bevy::prelude::Deref;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::fmt::{Display, Formatter};

// maybe use a cow string
#[derive(Deref, Default, Debug, Clone, Eq, PartialEq)]
pub struct SizedString<const S: usize>(String);

impl<const S: usize> Display for SizedString<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<const S: usize> SizedString<S> {
    pub fn new(s: impl Into<String>) -> Self {
        let s = s.into();
        if s.len() > S {
            return Self(s.chars().take(S).collect());
        }
        Self(s)
    }
}

impl<const S: usize> PacketSerializable for SizedString<S> {
    fn write_size(&self) -> usize {
        var_int_size(self.len() as i32) + self.len()
    }

    fn write(&self, buf: &mut BytesMut) {
        write_var_int(buf, self.len() as i32);
        buf.put_slice(self.as_bytes());
    }
}

impl<const S: usize> PacketDeserializable for SizedString<S> {
    fn read(buffer: &mut Bytes) -> anyhow::Result<Self> {
        let length = *VarInt::read(buffer)? as usize;
        if buffer.remaining() < length {
            bail!("not enough bytes for sized string")
        }
        if length > S {
            bail!("String too long. {:?} > {}", length, S);
        }

        match String::from_utf8(get_vec(buffer, length)) {
            Ok(string) => Ok(SizedString::new(string)),
            Err(err) => bail!("failed to read string: {err}")
        }
    }
}

impl<const S: usize> From<String> for SizedString<S> {
    fn from(value: String) -> Self {
        SizedString::new(value)
    }
}