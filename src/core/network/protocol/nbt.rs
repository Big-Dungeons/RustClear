#![allow(clippy::upper_case_acronyms)]

use crate::core::network::packets::packet_deserializable::{get_vec, PacketDeserializable};
use crate::core::network::packets::packet_serializable::PacketSerializable;
use anyhow::bail;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::collections::HashMap;

pub const TAG_END_ID: u8 = 0;
pub const TAG_BYTE_ID: u8 = 1;
pub const TAG_SHORT_ID: u8 = 2;
pub const TAG_INT_ID: u8 = 3;
pub const TAG_LONG_ID: u8 = 4;
pub const TAG_FLOAT_ID: u8 = 5;
pub const TAG_DOUBLE_ID: u8 = 6;
pub const TAG_BYTE_ARRAY_ID: u8 = 7;
pub const TAG_STRING_ID: u8 = 8;
pub const TAG_LIST_ID: u8 = 9;
pub const TAG_COMPOUND_ID: u8 = 10;
pub const TAG_INT_ARRAY_ID: u8 = 11;
pub const TAG_LONG_ARRAY_ID: u8 = 12;


/// NBT
///
/// This struct represents the root NBT Tag Compound.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct NBT {
    pub root_name: String,
    pub nodes: HashMap<String, NBTNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NBTNode {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<u8>),
    String(String),
    List { type_id: u8, children: Vec<NBTNode> },
    Compound(HashMap<String, NBTNode>),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl PacketSerializable for NBT {
    fn write_size(&self) -> usize {
        let mut size = 1;
        size += string_size(self.root_name.as_str());
        for (name, node) in self.nodes.iter() {
            size += 1 + string_size(name.as_str()) + node_size(node);
        }
        size += 1;
        size
    }
    fn write(&self, buf: &mut BytesMut) {
        buf.put_u8(TAG_COMPOUND_ID);
        write_string_nbt(buf, self.root_name.as_str());
        for (str, node) in self.nodes.iter() {
            write_node(buf, Some(str.as_str()), node);
        }
        buf.put_u8(TAG_END_ID)
    }
}

impl PacketSerializable for Option<NBT> {
    fn write_size(&self) -> usize {
        match self {
            None => size_of::<u8>(),
            Some(nbt) => nbt.write_size()
        }
    }
    fn write(&self, buf: &mut BytesMut) {
        match self {
            None => 0u8.write(buf),
            Some(nbt) => nbt.write(buf)
        }
    }
}

pub fn write_node(buffer: &mut BytesMut, name: Option<&str>, node: &NBTNode) {
    match node {
        NBTNode::Byte(value) => {
            if let Some(name) = name {
                buffer.put_u8(TAG_BYTE_ID);
                write_string_nbt(buffer, name);
            }
            buffer.put_i8(*value)
        }
        NBTNode::Short(value) => {
            if let Some(name) = name {
                buffer.put_u8(TAG_SHORT_ID);
                write_string_nbt(buffer, name);
            }
            buffer.put_i16(*value)
        }
        NBTNode::Int(value) => {
            if let Some(name) = name {
                buffer.put_u8(TAG_INT_ID);
                write_string_nbt(buffer, name);
            }
            buffer.put_i32(*value)
        }
        NBTNode::Long(value) => {
            if let Some(name) = name {
                buffer.put_u8(TAG_LONG_ID);
                write_string_nbt(buffer, name);
            }
            buffer.put_i64(*value)
        }
        NBTNode::Float(value) => {
            if let Some(name) = name {
                buffer.put_u8(TAG_FLOAT_ID);
                write_string_nbt(buffer, name);
            }
            buffer.put_f32(*value)
        }
        NBTNode::Double(value) => {
            if let Some(name) = name {
                buffer.put_u8(TAG_DOUBLE_ID);
                write_string_nbt(buffer, name);
            }
            buffer.put_f64(*value)
        }
        NBTNode::ByteArray(value) => {
            if let Some(name) = name {
                buffer.put_u8(TAG_BYTE_ARRAY_ID);
                write_string_nbt(buffer, name);
            }
            buffer.put_slice(value)
        }
        NBTNode::String(string) => {
            if let Some(name) = name {
                buffer.put_u8(TAG_STRING_ID);
                write_string_nbt(buffer, name);
            }
            write_string_nbt(buffer, string.as_str())
        }
        NBTNode::List { type_id, children } => {
            if let Some(name) = name {
                buffer.put_u8(TAG_LIST_ID);
                write_string_nbt(buffer, name);
            }
            buffer.put_u8(*type_id);
            buffer.put_i32(children.len() as i32);
            for child in children {
                write_node(buffer, None, child)
            }
        }
        NBTNode::Compound(nodes) => {
            if let Some(name) = name {
                buffer.put_u8(TAG_COMPOUND_ID);
                write_string_nbt(buffer, name);
            }
            for (string, node) in nodes {
                write_node(buffer, Some(string.as_str()), node);
            }
            buffer.put_u8(TAG_END_ID)
        }
        NBTNode::IntArray(values) => {
            if let Some(name) = name {
                buffer.put_u8(TAG_INT_ARRAY_ID);
                write_string_nbt(buffer, name);
            }
            buffer.put_i32(values.len() as i32);
            for value in values {
                buffer.put_i32(*value);
            }
        }
        NBTNode::LongArray(values) => {
            if let Some(name) = name {
                buffer.put_u8(TAG_LONG_ARRAY_ID);
                write_string_nbt(buffer, name);
            }
            buffer.put_i32(values.len() as i32);
            for value in values {
                buffer.put_i64(*value);
            }
        }
    }
}

// different format from normal strings
fn write_string_nbt(buffer: &mut BytesMut, name: &str) {
    buffer.put_u16(name.len() as u16);
    buffer.put_slice(name.as_bytes());
}

fn string_size(str: &str) -> usize {
    2 + str.len()
}

fn node_size(node: &NBTNode) -> usize {
    match node {
        NBTNode::Byte(_) => 1,
        NBTNode::Short(_) => 2,
        NBTNode::Int(_) => 4,
        NBTNode::Long(_) => 8,
        NBTNode::Float(_) => 4,
        NBTNode::Double(_) => 8,
        NBTNode::ByteArray(bytes) => 4 + bytes.len(),
        NBTNode::String(s) => string_size(s.as_str()),
        NBTNode::List { type_id: _, children } => {
            let mut size = 1 + 4;
            for child in children {
                size += node_size(child);
            }
            size
        }
        NBTNode::Compound(nodes) => {
            let mut size = 0;
            for (name, node) in nodes {
                size += 1 + string_size(name.as_str()) + node_size(node);
            }
            size += 1;
            size
        }
        NBTNode::IntArray(values) => 4 + (values.len() * 4),
        NBTNode::LongArray(values) => 4 + (values.len() * 8),
    }
}

impl PacketDeserializable for NBT {
    fn read(buffer: &mut Bytes) -> anyhow::Result<Self> {
        let name = read_string_nbt(buffer)?;
        let node = read_node(buffer, TAG_COMPOUND_ID)?;

        if let NBTNode::Compound(nodes) = node {
            return Ok(NBT {
                root_name: name,
                nodes,
            })
        }
        bail!("Somehow read something other than NBTNode::Compound")
    }
}

impl PacketDeserializable for Option<NBT> {
    fn read(buffer: &mut Bytes) -> anyhow::Result<Self> {
        if u8::read(buffer)? != TAG_COMPOUND_ID {
            return Ok(None);
        };
        Ok(Some(NBT::read(buffer)?))
    }
}

fn read_string_nbt(buffer: &mut Bytes) -> anyhow::Result<String> {
    let length = u16::read(buffer)? as usize;
    if buffer.remaining() < length {
        bail!("Not enough bytes for string")
    }
    Ok(String::from_utf8(get_vec(buffer, length))?)
}

fn read_node(buffer: &mut Bytes, tag: u8) -> anyhow::Result<NBTNode> {
    let node = match tag {
        TAG_BYTE_ID => NBTNode::Byte(PacketDeserializable::read(buffer)?),
        TAG_SHORT_ID => NBTNode::Short(PacketDeserializable::read(buffer)?),
        TAG_INT_ID => NBTNode::Int(PacketDeserializable::read(buffer)?),
        TAG_LONG_ID => NBTNode::Long(PacketDeserializable::read(buffer)?),
        TAG_FLOAT_ID => NBTNode::Float(PacketDeserializable::read(buffer)?),
        TAG_DOUBLE_ID => NBTNode::Double(PacketDeserializable::read(buffer)?),


        TAG_BYTE_ARRAY_ID => {
            // would be faster to do one check for the whole array
            // and do an unchecked read for u8, but im too lazy
            let length = u8::read(buffer)? as usize;
            let mut vec: Vec<u8> = Vec::with_capacity(length);
            for _ in 0..length {
                vec.push(u8::read(buffer)?)
            }
            NBTNode::ByteArray(vec)
        }
        TAG_STRING_ID => {
            let value = read_string_nbt(buffer)?;
            NBTNode::String(value)
        }
        TAG_LIST_ID => {
            let type_id = u8::read(buffer)?;
            let list_len = i32::read(buffer)?;
            let mut nodes: Vec<NBTNode> = Vec::new();
            for _ in 0..list_len {
                let node = read_node(buffer, type_id)?;
                nodes.push(node)
            }
            NBTNode::List { type_id, children: nodes }
        }
        TAG_COMPOUND_ID => {
            let mut nodes: HashMap<String, NBTNode> = HashMap::new();
            loop {
                let tag = u8::read(buffer)?;
                if tag == TAG_END_ID {
                    break;
                } else {
                    let name = read_string_nbt(buffer)?;
                    let node = read_node(buffer, tag)?;
                    nodes.insert(name, node);
                }
            }
            NBTNode::Compound(nodes)
        }
        TAG_INT_ARRAY_ID => {
            let length = i32::read(buffer)? as usize;
            let mut vec: Vec<i32> = Vec::with_capacity(length);
            for _ in 0..length {
                vec.push(i32::read(buffer)?)
            }
            NBTNode::IntArray(vec)
        }
        TAG_LONG_ARRAY_ID => {
            let length = i64::read(buffer)? as usize;
            let mut vec: Vec<i64> = Vec::with_capacity(length);
            for _ in 0..length {
                vec.push(i64::read(buffer)?)
            }
            NBTNode::LongArray(vec)
        }
        _ => unreachable!()
    };
    Ok(node)
}