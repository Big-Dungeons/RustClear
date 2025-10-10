use fstr::FString;

use crate::{network::binary::nbt::serialize::TAG_STRING_ID};
use std::collections::HashMap;

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