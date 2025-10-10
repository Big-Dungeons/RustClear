use crate::network::binary::nbt::nbt::{NBTNode, NBT};

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

pub fn serialize_nbt(nbt: &NBT) -> Vec<u8> {
    let mut vec = Vec::new();
    vec.push(TAG_COMPOUND_ID);
    write_string(&mut vec, &nbt.root_name);
    for (str, node) in &nbt.nodes {
        write_entry(str, node, &mut vec);
    }
    vec.push(TAG_END_ID);
    vec
}

// currently list is not done because im lazy
// it is quite simple tho

pub fn write_entry(name: &str, node: &NBTNode, vec: &mut Vec<u8>) {
    match node {
        NBTNode::Byte(value) => {
            vec.push(TAG_BYTE_ID);
            write_string(vec, name);
            vec.push(*value as u8)
        }
        NBTNode::Short(value) => {
            vec.push(TAG_SHORT_ID);
            write_string(vec, name);
            vec.extend_from_slice(&value.to_be_bytes())
        }
        NBTNode::Int(value) => {
            vec.push(TAG_INT_ID);
            write_string(vec, name);
            vec.extend_from_slice(&value.to_be_bytes())
        }
        NBTNode::Long(value) => {
            vec.push(TAG_LONG_ID);
            write_string(vec, name);
            vec.extend_from_slice(&value.to_be_bytes())
        }
        NBTNode::Float(value) => {
            vec.push(TAG_FLOAT_ID);
            write_string(vec, name);
            vec.extend_from_slice(&value.to_be_bytes())
        }
        NBTNode::Double(value) => {
            vec.push(TAG_DOUBLE_ID);
            write_string(vec, name);
            vec.extend_from_slice(&value.to_be_bytes())
        }
        NBTNode::ByteArray(value) => {
            vec.push(TAG_BYTE_ARRAY_ID);
            write_string(vec, name);
            vec.extend_from_slice(&(value.len() as i32).to_be_bytes());
            vec.extend_from_slice(value);
        }
        NBTNode::String(value) => {
            vec.push(TAG_STRING_ID);
            write_string(vec, name);
            write_string(vec, value);
        }
        NBTNode::List { type_id, children } => {
            vec.push(TAG_LIST_ID);
            write_string(vec, name);
            vec.push(*type_id);
            vec.extend_from_slice(&(children.len() as i32).to_be_bytes());
            for child in children {
                write_unnamed_entry(child, vec);
            }
        }
        NBTNode::Compound(nodes) => {
            vec.push(TAG_COMPOUND_ID);
            write_string(vec, name);
            for (str, node) in nodes {
                write_entry(str, node, vec);
            }
            vec.push(TAG_END_ID);
        }
        NBTNode::IntArray(values) => {
            vec.push(TAG_INT_ARRAY_ID);
            write_string(vec, name);
            vec.extend_from_slice(&(values.len() as i32).to_be_bytes());
            for value in values {
                vec.extend_from_slice(&value.to_be_bytes());
            }
        }
        NBTNode::LongArray(values) => {
            vec.push(TAG_LONG_ARRAY_ID);
            write_string(vec, name);
            vec.extend_from_slice(&(values.len() as i32).to_be_bytes());
            for value in values {
                vec.extend_from_slice(&value.to_be_bytes());
            }
        }
    }
}

fn write_unnamed_entry(node: &NBTNode, vec: &mut Vec<u8>) {
    match node {
        NBTNode::Byte(value) => {
            vec.push(*value as u8);
        }
        NBTNode::Short(value) => {
            vec.extend_from_slice(&value.to_be_bytes());
        }
        NBTNode::Int(value) => {
            vec.extend_from_slice(&value.to_be_bytes());
        }
        NBTNode::Long(value) => {
            vec.extend_from_slice(&value.to_be_bytes());
        }
        NBTNode::Float(value) => {
            vec.extend_from_slice(&value.to_be_bytes());
        }
        NBTNode::Double(value) => {
            vec.extend_from_slice(&value.to_be_bytes());
        }
        NBTNode::ByteArray(value) => {
            vec.extend_from_slice(&(value.len() as i32).to_be_bytes());
            vec.extend_from_slice(value);
        }
        NBTNode::String(value) => {
            write_string(vec, value);
        }
        NBTNode::List { type_id, children, .. } => {
            vec.push(*type_id);
            vec.extend_from_slice(&(children.len() as i32).to_be_bytes());
            for child in children {
                write_unnamed_entry(child, vec);
            }
        }
        NBTNode::Compound(nodes) => {
            for (str, node) in nodes {
                write_entry(str, node, vec);
            }
            vec.push(TAG_END_ID);
        }
        NBTNode::IntArray(values) => {
            vec.extend_from_slice(&(values.len() as i32).to_be_bytes());
            for v in values {
                vec.extend_from_slice(&v.to_be_bytes());
            }
        }
        NBTNode::LongArray(values) => {
            vec.extend_from_slice(&(values.len() as i32).to_be_bytes());
            for v in values {
                vec.extend_from_slice(&v.to_be_bytes());
            }
        }
    }
}

fn write_string(vec: &mut Vec<u8>, name: &str) {
    vec.extend_from_slice(&(name.len() as u16).to_be_bytes());
    vec.extend_from_slice(name.as_bytes());
}

fn string_size(s: &str) -> usize {
    2 + s.as_bytes().len()
}

pub fn nbt_write_size(nbt: &NBT) -> usize {
    let mut size = 1;
    size += string_size(&nbt.root_name);
    for (name, node) in &nbt.nodes {
        size += 1 + string_size(name) + entry_size(node);
    }
    size += 1;
    size
}
fn entry_size(node: &NBTNode) -> usize {
    match node {
        NBTNode::Byte(_) => 1,
        NBTNode::Short(_) => 2,
        NBTNode::Int(_) => 4,
        NBTNode::Long(_) => 8,
        NBTNode::Float(_) => 4,
        NBTNode::Double(_) => 8,
        NBTNode::ByteArray(bytes) => 4 + bytes.len(),
        NBTNode::String(s) => string_size(s),
        NBTNode::List { type_id: _, children } => {
            let mut size = 1 + 4;
            for child in children {
                size += entry_size(child);
            }
            size
        }
        NBTNode::Compound(nodes) => {
            let mut size = 0;
            for (name, node) in nodes {
                size += 1 + string_size(name) + entry_size(node);
            }
            size += 1;
            size
        }
        NBTNode::IntArray(values) => 4 + (values.len() * 4),
        NBTNode::LongArray(values) => 4 + (values.len() * 8),
    }
}