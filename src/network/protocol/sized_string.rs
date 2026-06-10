use crate::network::packets::packet_deserializable::{get_vec, PacketDeserializable};
use crate::network::protocol::var_int::VarInt;
use anyhow::bail;
use bevy::prelude::Deref;
use bytes::{Buf, Bytes};
use std::fmt::{Display, Formatter};

// maybe use a cow string
#[derive(Debug, Clone, Deref)]
pub struct SizedString<const S: usize>(String);

impl<const S: usize> Display for SizedString<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<const S: usize> SizedString<S> {
    pub fn new(s: impl Into<String>) -> anyhow::Result<Self> {
        let s = s.into();
        if s.len() > S {
            bail!("string length {} exceeds maximum {S}", s.len());
        }
        Ok(Self(s))
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
            Ok(string) => Ok(SizedString::new(string)?),
            Err(err) => bail!("failed to read string: {err}")
        }
    }
}