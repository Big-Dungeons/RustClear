use fstr::FString;

use crate::network::packets::packet_deserialize::PacketDeserializable;
use crate::network::packets::packet_serialize::PacketSerializable;
use crate::utils::get_vec;
use anyhow::bail;
use bytes::{Buf, BufMut, BytesMut};
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
#[derive(Debug, Clone, PartialEq)]
pub struct NBT {
    pub root_name: FString,
    pub nodes: HashMap<FString, NBTNode>,
}

impl NBT {

    /// Creates a new [NBT] struct with the provided nodes.
    /// It does not have a root name
    pub fn with_nodes(nodes: Vec<(FString, NBTNode)>) -> NBT {
        let mut map = HashMap::new();
        for (name, node) in nodes {
            map.insert(name, node);
        }
        NBT {
            root_name: "".into(),
            nodes: map,
        }
    }

    /// creates a string nbt node
    /// used for [NBTNode::Compound] and [NBT]
    pub fn string(name: impl Into<FString>, value: impl Into<FString>) -> (FString, NBTNode) {
        (name.into(), NBTNode::String(value.into()))
    }

    /// creates a compound node
    /// used for [NBTNode::Compound] and [NBT]
    pub fn compound(name: impl Into<FString>, nodes: Vec<(FString, NBTNode)>) -> (FString, NBTNode) {
        let mut compound_map = HashMap::new();
        for (name, node) in nodes {
            compound_map.insert(name, node);
        }
        (name.into(), NBTNode::Compound(compound_map))
    }

    /// creates a list node
    /// used for [NBTNode::Compound] and [NBT]
    pub fn list(name: impl Into<FString>, type_id: u8, list: Vec<NBTNode>) -> (FString, NBTNode) {
        (name.into(), NBTNode::List { type_id, children: list })
    }

    pub fn byte(name: impl Into<FString>, value: i8) -> (FString, NBTNode) {
        (name.into(), NBTNode::Byte(value))
    }

    pub fn short(name: impl Into<FString>, value: i16) -> (FString, NBTNode) {
        (name.into(), NBTNode::Short(value))
    }

    pub fn int(name: impl Into<FString>, value: i32) -> (FString, NBTNode) {
        (name.into(), NBTNode::Int(value))
    }

    pub fn long(name: impl Into<FString>, value: i64) -> (FString, NBTNode) {
        (name.into(), NBTNode::Long(value))
    }

    /// takes a string,
    /// splits it into lines and creates a list nbt node representing strings.
    pub fn list_from_string(name: impl Into<FString>, string: impl Into<FString>) -> (FString, NBTNode) {
        let list = string
            .into()
            .with_iter(str::lines)
            .map(|line| NBTNode::String(line.to_owned()))
            .collect();

        (name.into(), NBTNode::List { type_id: TAG_STRING_ID, children: list })
    }
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
    String(FString),
    List { type_id: u8, children: Vec<NBTNode> },
    Compound(HashMap<FString, NBTNode>),
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
            buffer.put_slice(&value)
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
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        let name = read_string_nbt(buffer);
        let node = read_node(buffer, TAG_COMPOUND_ID);

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
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        if buffer.get_u8() != TAG_COMPOUND_ID {
            return Ok(None);
        };
        Ok(Some(NBT::read(buffer)?))
    }
}

fn read_string_nbt(buffer: &mut impl Buf) -> FString {
    let size = buffer.get_u16() as usize;
    let str = FString::from_bytes(&buffer.chunk()[..size]).unwrap();
    buffer.advance(size);
    str
}

fn read_node(buffer: &mut impl Buf, tag: u8) -> NBTNode {
    match tag {
        TAG_BYTE_ID => {
            let value = buffer.get_i8();
            NBTNode::Byte(value)
        }
        TAG_SHORT_ID => {
            let value = buffer.get_i16();
            NBTNode::Short(value)
        }
        TAG_INT_ID => {
            let value = buffer.get_i32();
            NBTNode::Int(value)
        }
        TAG_LONG_ID => {
            let value = buffer.get_i64();
            NBTNode::Long(value)
        }
        TAG_FLOAT_ID => {
            let value = buffer.get_f32();
            NBTNode::Float(value)
        }
        TAG_DOUBLE_ID => {
            let value = buffer.get_f64();
            NBTNode::Double(value)
        }

        TAG_BYTE_ARRAY_ID => {
            let array_len = buffer.get_i32() as usize;
            let vec = get_vec(buffer, array_len);
            NBTNode::ByteArray(vec)
        }
        TAG_STRING_ID => {
            let value = read_string_nbt(buffer);
            NBTNode::String(value)
        }
        TAG_LIST_ID => {
            let type_id = buffer.get_u8();
            let list_len = buffer.get_i32();
            let mut nodes: Vec<NBTNode> = Vec::new();
            for _ in 0..list_len {
                let node = read_node(buffer, type_id);
                nodes.push(node)
            }
            NBTNode::List { type_id, children: nodes }
        }
        TAG_COMPOUND_ID => {
            let mut nodes: HashMap<FString, NBTNode> = HashMap::new();
            loop {
                let tag = buffer.get_u8();
                if tag == TAG_END_ID {
                    break;
                } else {
                    let name = read_string_nbt(buffer);
                    let node = read_node(buffer, tag);
                    nodes.insert(name, node);
                }
            }
            NBTNode::Compound(nodes)
        }
        TAG_INT_ARRAY_ID => {
            let array_len = buffer.get_i32() as usize;
            let mut vec: Vec<i32> = Vec::with_capacity(array_len);
            for _ in 0..array_len {
                vec.push(buffer.get_i32())
            }
            NBTNode::IntArray(vec)
        }
        TAG_LONG_ARRAY_ID => {
            let array_len = buffer.get_i32() as usize;
            let mut vec: Vec<i64> = Vec::with_capacity(array_len);
            for _ in 0..array_len {
                vec.push(buffer.get_i64())
            }
            NBTNode::LongArray(vec)
        }
        _ => unreachable!()
    }
}