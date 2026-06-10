use crate::block::Block;
use crate::network::packets::packet_serializable::PacketSerializable;
use crate::network::packets::IdentifiedPacket;
use crate::network::protocol::block_position::BlockPosition;
use crate::network::protocol::var_int::VarInt;
use crate::player::inventory::item_stack::ItemStack;
use crate::types::chat_component::ChatComponent;
use bytes::BytesMut;
use enumset::{EnumSet, EnumSetType};
use glam::IVec3;
use macros::{identified_packet, PacketSerializable};

#[identified_packet(id=0x01)]
#[derive(PacketSerializable)]
pub struct JoinGame<'a> {
    pub entity_id: i32,
    pub gamemode: u8,
    pub dimension: u8,
    pub difficulty: u8,
    pub max_players: u8,
    pub level_type: &'a str,
    pub reduced_debug_info: bool,
}

#[identified_packet(id=0x02)]
#[derive(PacketSerializable)]
pub struct Chat {
    pub component: ChatComponent,
    pub chat_type: i8,
}

impl Chat {
    pub fn new(str: &str) -> Self {
        Self {
            component: ChatComponent::new(str),
            chat_type: 0,
        }
    }
}

#[derive(EnumSetType)]
pub enum Relative {
    X,
    Y,
    Z,
    Yaw,
    Pitch
}

#[identified_packet(id=0x08)]
#[derive(PacketSerializable)]
pub struct PositionLook {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub flags: EnumSet<Relative>,
}

#[identified_packet(id=0x21)]
#[derive(PacketSerializable)]
pub struct ChunkData {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub is_new_chunk: bool,
    pub section_bitmask: u16,
    pub data: Vec<u8>
}

#[identified_packet(id=0x23)]
#[derive(PacketSerializable)]
pub struct BlockChange {
    pub position: BlockPosition,
    pub block_state: VarInt,
}

impl BlockChange {
    pub fn new(position: IVec3, block: Block) -> Self {
        Self {
            position: BlockPosition(position),
            block_state: VarInt(block.get_blockstate_id() as i32)
        }
    }
}


#[identified_packet(id=0x2f)]
#[derive(PacketSerializable)]
pub struct SetSlot {
    pub window_id: i8,
    pub slot: i16,
    pub item_stack: Option<ItemStack>
}

#[identified_packet(id=0x30)]
pub struct WindowItems {
    pub window_id: i8,
    pub items: Vec<Option<ItemStack>>,
}

// why couldnt mojang use var int for length :(
impl PacketSerializable for WindowItems {
    fn write_size(&self) -> usize {
        let mut size = self.window_id.write_size() + (self.items.len() as i16).write_size();
        for item in self.items.iter() {
            size += item.write_size()
        }
        size
    }
    fn write(&self, buf: &mut BytesMut) {
        self.window_id.write(buf);
        (self.items.len() as i16).write(buf);
        for item in self.items.iter() {
            item.write(buf);
        }
    }
}

#[identified_packet(id=0x32)]
#[derive(PacketSerializable)]
pub struct ConfirmTransaction {
    pub window_id: i8,
    pub action_number: i16,
    pub accepted: bool,
}