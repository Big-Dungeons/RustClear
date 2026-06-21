use crate::core::block::Block;
use crate::core::entity::entity_metadata::EntityMetadata;
use crate::core::network::packets::packet_serializable::PacketSerializable;
use crate::core::network::packets::IdentifiedPacket;
use crate::core::network::protocol::block_position::BlockPosition;
use crate::core::network::protocol::nbt::NBT;
use crate::core::network::protocol::sized_string::SizedString;
use crate::core::network::protocol::var_int::{var_int_size, write_var_int, VarInt};
use crate::core::player::inventory::item_stack::ItemStack;
use crate::core::types::chat_component::ChatComponent;
use crate::core::types::entity_variant::{EntityVariant, ObjectVariant};
use crate::core::types::sound::Sound;
use bytes::BytesMut;
use enumset::{EnumSet, EnumSetType};
use glam::{DVec3, I16Vec3, IVec3};
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

#[identified_packet(id=0x04)]
#[derive(Debug, PacketSerializable)]
pub struct EntityEquipment {
    pub entity_id: VarInt,
    pub item_slot: i16,
    pub item_stack: Option<ItemStack>,
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

#[identified_packet(id=0x0d)]
#[derive(Debug, PacketSerializable)]
pub struct CollectItem {
    pub item_entity_id: VarInt,
    pub player_entity_id: VarInt,
}

#[identified_packet(id=0x0e)]
#[derive(Debug)]
pub struct SpawnObject {
    pub entity_id: VarInt,
    pub variant: ObjectVariant,
    pub position: IVec3,
    pub pitch: i8,
    pub yaw: i8,
    pub data: i32,
    pub velocity: I16Vec3,
}

impl PacketSerializable for SpawnObject {
    fn write_size(&self) -> usize {
        let mut size = 
            self.entity_id.write_size() +
            self.variant.write_size() +
            self.position.write_size() +
            self.pitch.write_size() +
            self.yaw.write_size() +
            self.data.write_size();
        
        if self.data > 0 {
            size += self.velocity.write_size()
        }
        size
    }

    fn write(&self, buf: &mut BytesMut) {
        self.entity_id.write(buf);
        self.variant.write(buf);
        self.position.write(buf);
        self.pitch.write(buf);
        self.yaw.write(buf);
        self.data.write(buf);
        if self.data > 0 { 
            self.velocity.write(buf)
        }
    }
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

#[identified_packet(id=0x12)]
#[derive(Debug, PacketSerializable)]
pub struct EntityVelocity {
    pub entity_id: VarInt,
    pub velocity: I16Vec3
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

#[identified_packet(id=0x1b)]
#[derive(Debug, PacketSerializable)]
pub struct EntityAttach {
    pub entity_id: i32,
    pub vehicle_id: i32,
    pub leash: bool,
}

#[identified_packet(id=0x1c)]
#[derive(Debug, PacketSerializable)]
pub struct PacketEntityMetadata<T: PacketSerializable> {
    pub entity_id: VarInt,
    pub metadata: T,
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

#[identified_packet(id=0x24)]
#[derive(Debug, PacketSerializable)]
pub struct BlockAction {
    pub position: BlockPosition,
    pub event_id: u8,
    pub event_data: u8,
    pub block_id: VarInt
}

impl BlockAction {
    pub fn new(position: IVec3, event_id: u8, event_data: u8, block: Block) -> Self {
        Self {
            position: BlockPosition(position),
            event_id,
            event_data,
            block_id: VarInt(((block.get_blockstate_id() >> 4) & 4095) as i32),
        }
    }
}

#[identified_packet(id=0x29)]
#[derive(Debug, PacketSerializable)]
pub struct SoundEffect {
    sound: Sound,
    position: IVec3, // (dvec3 * 8.0).as_ivec3()
    volume: f32,
    pitch: u8, //
}

impl SoundEffect {
    pub fn new(sound: Sound, position: DVec3, volume: f32, pitch: f32) -> Self {
        Self {
            sound,
            position: (position * 8.0).as_ivec3(),
            volume,
            pitch: (pitch * 63.0).clamp(0.0, 255.0) as u8,
        }
    }
}

#[identified_packet(id=0x2d)]
#[derive(Debug, PacketSerializable)]
pub struct OpenWindow {
    pub window_id: i8,
    pub inventory_type: &'static str,
    pub window_title: ChatComponent,
    pub slot_count: i8,
}

#[identified_packet(id=0x2e)]
#[derive(Debug, PacketSerializable)]
pub struct CloseWindow {
    pub window_id: i8,
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
pub struct WindowItems<'a> {
    pub window_id: i8,
    pub items: &'a [Option<ItemStack>],
}

// why couldnt mojang use var int for length :(
impl<'a> PacketSerializable for WindowItems<'a> {
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

#[identified_packet(id=0x35)]
#[derive(Debug, PacketSerializable)]
pub struct UpdateBlockEntity {
    pub position: BlockPosition,
    pub block_entity_id: u8,
    pub nbt: Option<NBT>,
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