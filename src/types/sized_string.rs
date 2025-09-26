use crate::network::binary::var_int::{read_var_int, var_int_size, write_var_int};
use crate::network::packets::packet_deserialize::PacketDeserializable;
use crate::network::packets::packet_serialize::PacketSerializable;
use crate::types::sized_string_mut::SizedStringMut;
use anyhow::{bail, Context};
use bytes::{Buf, BufMut, BytesMut};
use std::fmt::{Debug, Display, Formatter};
use std::ops::Deref;
use std::str;

// S is size of bytes not char len

// should have the underlying data size be S * 4,
// and then also ensure that the actual amount of characters doesnt surpass S

#[derive(Debug, PartialEq, Copy, Clone)]
pub struct SizedString<const S: usize> {
    pub length: usize,
    pub(super) data: [u8; S]
}

impl<const S: usize> SizedString<S> {

    pub const EMPTY: SizedString<S> = SizedString {
        length: 0,
        data: [0; S],
    };

    pub const unsafe fn slice_truncated<const N : usize>(slice: [u8; N]) -> Self {
        // need to do this for it to work const
        let mut data = [0; S];
        let mut i = 0;

        while i < N && i < S {
            data[i] = slice[i];
            i += 1;
        }
        Self {
            length: i,
            data,
        }
    }

    pub fn truncated(str: &str) -> Self {
        let mut data = [0u8; S];
        let len = floor_char_boundary(str, S);
        let bytes = str.as_bytes();
        
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

impl<const S: usize> Into<SizedString<S>> for SizedStringMut<S> {
    fn into(self) -> SizedString<S> {
        self.inner
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
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        let len = read_var_int(buffer).context("failed to read string length")? as usize;
        if len > S {
            bail!("String too long. {:?} > {}", len, S);
        }
        let mut data = [0u8; S];
        let read = buffer.copy_to_bytes(len);
        let _ = std::str::from_utf8(&read)?;
        data[..len].copy_from_slice(&read);
        
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
        unsafe { str::from_utf8_unchecked(&self.data[..self.length]) }
    }
}

// std has this but its flagged as unstable and i dont wanna enable nightly for the whole thing just to use this. Update to std if it becomes stable.
#[inline]
fn floor_char_boundary(str: &str, index: usize) -> usize {
    if index >= str.len() {
        str.len()
    } else {
        let lower_bound = index.saturating_sub(3);
        let new_index = str.as_bytes()[lower_bound..=index]
            .iter()
            .rposition(|b| (*b as i8) >= -0x40);

        // SAFETY: we know that the character boundary will be within four bytes
        unsafe { lower_bound + new_index.unwrap_unchecked() }
    }
}

