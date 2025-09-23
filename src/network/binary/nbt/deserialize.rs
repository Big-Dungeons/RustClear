use crate::network::binary::nbt::{NBTNode, NBT};
use crate::network::binary::nbt::serialize::*;
use bytes::{Buf, BytesMut};
use std::collections::HashMap;

// should use anyhow, client can maliciously send invalid nbt data
// does assume its all correct.
pub fn deserialize_nbt(buffer: &mut BytesMut) -> Option<NBT> {
    let tag = buffer.get_u8();
    // might issue if it isn't
    if tag != TAG_COMPOUND_ID {
        return None;
    }

    let name = read_string(buffer);
    let node = read_entry(buffer, tag);

    if let NBTNode::Compound(nodes) = node {
        return Some(NBT {
            root_name: name,
            nodes,
        })
    }
    None
}

fn read_entry(buffer: &mut BytesMut, tag: u8) -> NBTNode {
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
            let vec = buffer.split_to(array_len).to_vec();
            NBTNode::ByteArray(vec)
        }
        TAG_STRING_ID => {
            let value = read_string(buffer);
            NBTNode::String(value)
        }
        TAG_LIST_ID => {
            let type_id = buffer.get_u8();
            let list_len = buffer.get_i32();
            let mut nodes: Vec<NBTNode> = Vec::new();
            for _ in 0..list_len {
                let node = read_entry(buffer, type_id);
                nodes.push(node)
            }
            NBTNode::List { type_id, children: nodes }
        }
        TAG_COMPOUND_ID => {
            let mut nodes: HashMap<String, NBTNode> = HashMap::new();
            loop {
                let tag = buffer.get_u8();
                if tag == TAG_END_ID {
                    break;
                } else {
                    let name = read_string(buffer);
                    let node = read_entry(buffer, tag);
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

fn read_string(buffer: &mut BytesMut) -> String {
    let size = buffer.get_u16();
    String::from_utf8(buffer.split_to(size as usize).to_vec()).unwrap()
}