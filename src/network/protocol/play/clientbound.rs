use crate::block::Block;
use crate::entity::entity_metadata::EntityMetadata;
use crate::network::packets::packet_serializable::PacketSerializable;
use crate::network::packets::IdentifiedPacket;
use crate::network::protocol::block_position::BlockPosition;
use crate::network::protocol::sized_string::SizedString;
use crate::network::protocol::var_int::{var_int_size, write_var_int, VarInt};
use crate::player::inventory::item_stack::ItemStack;
use crate::types::chat_component::ChatComponent;
use crate::types::entity_variant::EntityVariant;
use bytes::BytesMut;
use enumset::{EnumSet, EnumSetType};
use glam::{I16Vec3, IVec3};
use macros::{identified_packet, PacketSerializable};

#[identified_packet(id=0x01)]
#[derive(Debug, PacketSerializable)]
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
#[derive(Debug, PacketSerializable)]
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

#[derive(Debug, EnumSetType)]
pub enum Relative {
    X,
    Y,
    Z,
    Yaw,
    Pitch
}

#[identified_packet(id=0x08)]
#[derive(Debug, PacketSerializable)]
pub struct PositionLook {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub flags: EnumSet<Relative>,
}

#[identified_packet(id=0x0f)]
#[derive(Debug, PacketSerializable)]
pub struct SpawnMob {
    pub entity_id: VarInt,
    pub entity_variant: EntityVariant,
    pub position: IVec3,
    pub yaw: i8,
    pub pitch: i8,
    pub head_yaw: i8,
    pub velocity: I16Vec3,
    pub metadata: EntityMetadata,
}

#[identified_packet(id=0x13)]
#[derive(Debug)]
pub struct DestroyEntity {
    pub entity_id: VarInt,
}

impl PacketSerializable for DestroyEntity {
    fn write_size(&self) -> usize {
        var_int_size(1) + self.entity_id.write_size()
    }

    fn write(&self, buf: &mut BytesMut) {
        write_var_int(buf, 1);
        self.entity_id.write(buf);
    }
}

#[identified_packet(id=0x18)]
#[derive(Debug, PacketSerializable)]
pub struct EntityTeleport {
    pub entity_id: VarInt,
    pub position: IVec3,
    pub yaw: i8,
    pub pitch: i8,
    pub on_ground: bool
}

#[identified_packet(id=0x19)]
#[derive(Debug, PacketSerializable)]
pub struct EntityYawRotate {
    pub entity_id: VarInt,
    pub yaw: i8,
}

#[identified_packet(id=0x21)]
#[derive(Debug, PacketSerializable)]
pub struct ChunkData {
    pub chunk_x: i32,
    pub chunk_z: i32,
    pub is_new_chunk: bool,
    pub section_bitmask: u16,
    pub data: Vec<u8>
}

#[identified_packet(id=0x23)]
#[derive(Debug, PacketSerializable)]
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
#[derive(Debug, PacketSerializable)]
pub struct SetSlot {
    pub window_id: i8,
    pub slot: i16,
    pub item_stack: Option<ItemStack>
}

#[identified_packet(id=0x30)]
#[derive(Debug)]
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
#[derive(Debug, PacketSerializable)]
pub struct ConfirmTransaction {
    pub window_id: i8,
    pub action_number: i16,
    pub accepted: bool,
}

#[identified_packet(id=0x3b)]
#[derive(Debug)]
pub struct ScoreboardObjective {
    pub objective_name: SizedString<16>,
    pub mode: i8,
    pub objective_value: SizedString<32>,
    pub render_type: SizedString<16>,
}

impl PacketSerializable for ScoreboardObjective {
    fn write_size(&self) -> usize {
        let mut size = self.objective_name.write_size() + self.mode.write_size();
        if self.mode == 0 /* add */ || self.mode == 2 /* update */ {
            size += self.objective_value.write_size() + self.render_type.write_size();
        }
        size
    }
    fn write(&self, buf: &mut BytesMut) {
        self.objective_name.write(buf);
        self.mode.write(buf);
        if self.mode == 0 /* add */ || self.mode == 2 /* update */ {
            self.objective_value.write(buf);
            self.render_type.write(buf);
        }
    }
}

#[identified_packet(id=0x3c)]
#[derive(Debug)]
pub struct UpdateScore {
    pub name: SizedString<40>,
    pub objective_value: SizedString<16>,
    pub value: VarInt,
    pub action: VarInt,
}

impl PacketSerializable for UpdateScore {
    fn write_size(&self) -> usize {
        let mut size = self.name.write_size() + self.action.write_size() + self.objective_value.write_size();
        if self.action.0 == 0 {
            size += self.value.write_size();
        }
        size
    }
    fn write(&self, buf: &mut BytesMut) {
        self.name.write(buf);
        self.action.write(buf);
        self.objective_value.write(buf);

        if self.action.0 == 0 {
            self.value.write(buf);
        }
    }
}

#[identified_packet(id=0x3d)]
#[derive(Debug, PacketSerializable)]
pub struct DisplayScoreboard {
    pub position: i8,
    pub score_name: SizedString<16>,
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TeamsAction {
    Create,
    Remove,
    Update,
    AddPlayer,
    RemovePlayer,
}

impl PacketSerializable for TeamsAction {
    fn write_size(&self) -> usize {
        (*self as u8).write_size()
    }
    fn write(&self, buf: &mut BytesMut) {
        (*self as u8).write(buf)
    }
}

#[identified_packet(id=0x3e)]
#[derive(Debug)]
pub struct Teams {
    pub name: SizedString<16>,
    pub display_name: SizedString<32>,
    pub prefix: SizedString<16>,
    pub suffix: SizedString<16>,
    pub name_tag_visibility: SizedString<32>,
    pub color: i8,
    pub players: Vec<SizedString<40>>,
    pub action: TeamsAction,
    pub friendly_flags: i8,
}

impl PacketSerializable for Teams {
    fn write_size(&self) -> usize {
        let mut size = self.name.write_size() + self.action.write_size();
        if self.action == TeamsAction::Create || self.action == TeamsAction::Update {
            size +=
                self.display_name.write_size() +
                self.prefix.write_size() +
                self.suffix.write_size() +
                self.friendly_flags.write_size() +
                self.name_tag_visibility.write_size() +
                self.color.write_size()
        }
        if self.action == TeamsAction::Create || self.action == TeamsAction::AddPlayer || self.action == TeamsAction::RemovePlayer {
            size += self.players.write_size();
        }
        size
    }
    fn write(&self, buf: &mut BytesMut) {
        self.name.write(buf);
        self.action.write(buf);

        if self.action == TeamsAction::Create || self.action == TeamsAction::Update {
            self.display_name.write(buf);
            self.prefix.write(buf);
            self.suffix.write(buf);
            self.friendly_flags.write(buf);
            self.name_tag_visibility.write(buf);
            self.color.write(buf);
        }

        if self.action == TeamsAction::Create || self.action == TeamsAction::AddPlayer || self.action == TeamsAction::RemovePlayer {
            self.players.write(buf);
        }
    }
}