use crate::net::packets::packet_serialize::PacketSerializable;
use bytes::BytesMut;
use std::ops::Deref;

/// [String] with a character size limit of N.
/// All characters after that size will be truncated on creation.
///
/// This implements deref, which means any str operation will work on it.
/// if you need the owned string, use .into_owned() or .to_owned()
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct SizedString<const N: usize>(String);
// if string is made public, you could change it and the size limit can't be enforced.

impl<const N: usize> SizedString<N> {
    pub fn truncated_owned(mut text: String) -> Self {
        if let Some((byte_position, _)) = text.char_indices().nth(N) {
            text.truncate(byte_position);
        }
        Self(text)
    }

    pub fn truncated(text: &str) -> Self {
        Self(text.chars().take(N).collect::<String>())
    }

    pub fn into_owned(self) -> String {
        self.0
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<const N: usize> Deref for SizedString<N> {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl<const N: usize> From<&str> for SizedString<N> {
    fn from(text: &str) -> Self {
        Self::truncated(text)
    }
}

impl<const N: usize> From<String> for SizedString<N> {
    fn from(text: String) -> Self {
        Self::truncated_owned(text)
    }
}

impl<const N: usize> PacketSerializable for SizedString<N> {
    fn write_size(&self) -> usize {
        self.0.write_size()
    }
    fn write(&self, buf: &mut BytesMut) {
        self.0.write(buf);
    }
}


impl<const N: usize> From<SizedString<N>> for String {
    fn from(text: SizedString<N>) -> Self {
        text.into_owned()
    }
}