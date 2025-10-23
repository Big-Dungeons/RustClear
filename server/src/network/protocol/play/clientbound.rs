use crate::constants::particle::Particle;
use crate::constants::potions::PotionEffect;
use crate::constants::{EntityVariant, ObjectVariant, Sound};
use crate::entity::entity_metadata::{EntityMetadata, PlayerMetadata};
use crate::inventory::item_stack::ItemStack;
use crate::network::binary::var_int::{var_int_size, write_var_int, VarInt};
use crate::network::packets::packet::IdentifiedPacket;
use crate::network::packets::packet_serialize::PacketSerializable;
use crate::player::attribute::AttributeMap;
use crate::player::player::GameProfile;
use crate::register_packets;
use crate::types::block_position::BlockPosition;
use crate::types::chat_component::ChatComponent;
use crate::types::sized_string::SizedString;
use blocks::packet_serializable;
use bytes::BytesMut;
use enumset::{EnumSet, EnumSetType};
use glam::{IVec3, Vec3};
use uuid::Uuid;

register_packets! {
    KeepAlive = 0x00;
    JoinGame<'_> = 0x01;
    Chat = 0x02;
    UpdateTime = 0x03;
    EntityEquipment = 0x04;
    // SpawnPosition = 0x05;
    // UpdateHealth = 0x06;
    // Respawn = 0x07;
    PositionLook = 0x08;
    // SetHotbarSlot = 0x09;
    // EntityUsedBed = 0x0a;
    // SwingAnimation = 0x0b
    SpawnPlayer = 0x0c;
    CollectItem = 0x0d;
    SpawnObject = 0x0e;
    SpawnMob = 0x0f;
    // SpawnPainting = 0x10;
    // SpawnExperienceOrb = 0x11;
    EntityVelocity = 0x12;
    DestroyEntites = 0x13;
    // Entity => 0x14;
    EntityRelativeMove = 0x15;
    EntityRotate = 0x16;
    EntityMoveRotate = 0x17;
    EntityTeleport = 0x18;
    EntityYawRotate = 0x19;
    EntityStatus = 0x1a;
    EntityAttach = 0x1b;
    // implements identified packet manually due to generics
    // PacketEntityMetadata = 0x1c;
    AddEffect = 0x1d;
    RemoveEffect = 0x1e;
    // SetExperience 0x1f;
    EntityProperties = 0x20;
    ChunkData = 0x21;
    // MultiBlockChange = 0x22;
    BlockChange = 0x23;
    BlockAction = 0x24;
    // BlockBreakAnimation = 0x25;
    // ChunkDataBulk = 0x26;
    // Explosion = 0x27;
    // Effect = 0x28;
    SoundEffect = 0x29;
    Particles = 0x2a;
    // ChangeGameState = 0x2b;
    // SpawnGlobalEntity = 0x2c;
    OpenWindow = 0x2d;
    CloseWindow = 0x2e;
    SetSlot = 0x2f;
    WindowItems = 0x30;
    // WindowProperty = 0x31;
    ConfirmTransaction = 0x32;
    // UpdateSign = 0x33;
    Maps = 0x34;
    // UpdateBlockEntity = 0x35;
    // SignEditorOpen = 0x36;
    // Statistics = 0x37;
    PlayerListItem<'_> = 0x38;
    PlayerAbilities = 0x39;
    TabCompleteReply = 0x3a;
    ScoreboardObjective = 0x3b;
    UpdateScore = 0x3c;
    DisplayScoreboard = 0x3d;
    Teams = 0x3e;
    CustomPayload<'_> = 0x3f;
    Disconnect = 0x40;
    // ServerDifficulty = 0x41;
    // CombatEvent = 0x42;
    // Camera = 0x43;
    // WorldBorder = 0x44;
    // Title => 0x45;
    // SetCompression = 0x46;
    PlayerListHeaderFooter = 0x47;
    // ResourcePackSend = 0x48;
    // EntityUpdateNBT = 0x49
}

packet_serializable! {
    pub struct JoinGame<'a> {
        pub entity_id: i32, // not VarInt,
        pub gamemode: u8,
        pub dimension: u8,
        pub difficulty: u8,
        pub max_players: u8,
        pub level_type: &'a str,
        pub reduced_debug_info: bool,
    }
}

packet_serializable! {
    pub struct KeepAlive {
        pub current_time: i32,
    }
}

packet_serializable! {
    pub struct Chat {
        pub component: ChatComponent,
        pub chat_type: i8,
    }
}

packet_serializable! {
    pub struct UpdateTime {
        pub world_age: i64,
        pub world_time: i64,
    }
}

packet_serializable! {
    pub struct EntityEquipment {
        pub entity_id: VarInt,
        pub item_slot: i16,
        pub item_stack: Option<ItemStack>
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

packet_serializable! {
    pub struct PositionLook {
        pub x: f64,
        pub y: f64,
        pub z: f64,
        pub yaw: f32,
        pub pitch: f32,
        pub flags: EnumSet<Relative> => &self.flags.as_u8(),
    }
}

packet_serializable! {
    pub struct SpawnPlayer {
        pub entity_id: i32 => &VarInt(self.entity_id),
        pub uuid: Uuid,
        pub x: f64 => &((self.x * 32.0).floor() as i32),
        pub y: f64 => &((self.y * 32.0).floor() as i32),
        pub z: f64 => &((self.z * 32.0).floor() as i32),
        pub yaw: f32 => &((self.yaw * 256.0 / 360.0) as i8),
        pub pitch: f32 => &((self.pitch * 256.0 / 360.0) as i8),
        pub current_item: i16,
        pub metadata: PlayerMetadata,
    }
}

packet_serializable! {
    pub struct CollectItem {
        pub item_entity_id: VarInt,
        pub entity_id: VarInt,
    }
}

const MOTION_CLAMP: f64 = 3.9;

packet_serializable!{
    pub struct SpawnObject {
        pub entity_id: i32 => &VarInt(self.entity_id),
        pub variant: ObjectVariant,
        pub x: f64 => &((self.x * 32.0).floor() as i32),
        pub y: f64 => &((self.y * 32.0).floor() as i32),
        pub z: f64 => &((self.z * 32.0).floor() as i32),
        pub pitch: f32 => &((self.pitch * 256.0 / 360.0) as i8),
        pub yaw: f32 => &((self.yaw * 256.0 / 360.0) as i8),
        // IDK how to serialize this, in correlation to the metadata provided
        pub data: i32,
        pub velocity_x: f64 => &((self.velocity_x.clamp(-MOTION_CLAMP, MOTION_CLAMP) * 8000.0) as i16),
        pub velocity_y: f64 => &((self.velocity_y.clamp(-MOTION_CLAMP, MOTION_CLAMP) * 8000.0) as i16),
        pub velocity_z: f64 => &((self.velocity_z.clamp(-MOTION_CLAMP, MOTION_CLAMP) * 8000.0) as i16),
    }
}

packet_serializable! {
    pub struct SpawnMob {
        pub entity_id: i32 => &VarInt(self.entity_id),
        pub entity_variant: EntityVariant,
        pub x: f64 => &((self.x * 32.0).floor() as i32),
        pub y: f64 => &((self.y * 32.0).floor() as i32),
        pub z: f64 => &((self.z * 32.0).floor() as i32),
        pub yaw: f32 => &((self.yaw * 256.0 / 360.0) as i8),
        pub pitch: f32 => &((self.pitch * 256.0 / 360.0) as i8),
        pub head_yaw: f32 => &((self.head_yaw * 256.0 / 360.0) as i8),
        pub velocity_x: f64 => &((self.velocity_x.clamp(-MOTION_CLAMP, MOTION_CLAMP) * 8000.0) as i16),
        pub velocity_y: f64 => &((self.velocity_y.clamp(-MOTION_CLAMP, MOTION_CLAMP) * 8000.0) as i16),
        pub velocity_z: f64 => &((self.velocity_z.clamp(-MOTION_CLAMP, MOTION_CLAMP) * 8000.0) as i16),
        pub metadata: EntityMetadata,
    }
}

packet_serializable! {
    pub struct EntityVelocity {
        pub entity_id: VarInt,
        pub velocity_x: i16,
        pub velocity_y: i16,
        pub velocity_z: i16,
    }
}

packet_serializable! {
    pub struct DestroyEntites {
        pub entities: Vec<VarInt>
    }
}

packet_serializable! {
    pub struct EntityRelativeMove {
        pub entity_id: i32 => &VarInt(self.entity_id),
        pub pos_x: f64 => &((self.pos_x * 32.0).floor() as i32 as i8),
        pub pos_y: f64 => &((self.pos_y * 32.0).floor() as i32 as i8),
        pub pos_z: f64 => &((self.pos_z * 32.0).floor() as i32 as i8),
        pub on_ground: bool,
    }
}

packet_serializable! {
    pub struct EntityRotate {
        pub entity_id: i32 => &VarInt(self.entity_id),
        pub yaw: f32 => &((self.yaw * 256.0 / 360.0) as i32 as i8),
        pub pitch: f32 => &((self.pitch * 256.0 / 360.0) as i32 as i8),
        pub on_ground: bool,
    }
}

packet_serializable! {
    pub struct EntityMoveRotate {
        pub entity_id: VarInt,
        pub pos_x: i8,
        pub pos_y: i8,
        pub pos_z: i8,
        pub yaw: i8,
        pub pitch: i8,
        pub on_ground: bool,
    }
}

packet_serializable! {
    pub struct EntityTeleport {
        pub entity_id: i32 => &VarInt(self.entity_id),
        pub pos_x: f64 => &((self.pos_x * 32.0).floor() as i32),
        pub pos_y: f64 => &((self.pos_y * 32.0).floor() as i32),
        pub pos_z: f64 => &((self.pos_z * 32.0).floor() as i32),
        pub yaw: f32 => &((self.yaw * 256.0 / 360.0) as i32 as i8),
        pub pitch: f32 => &((self.pitch * 256.0 / 360.0) as i32 as i8),
        pub on_ground: bool,
    }
}

packet_serializable! {
    pub struct EntityYawRotate {
        pub entity_id: i32 => &VarInt(self.entity_id),
        pub yaw: f32 => &((self.yaw * 256.0 / 360.0) as i32 as i8),
    }
}

packet_serializable! {
    pub struct EntityProperties {
        pub entity_id: VarInt,
        pub properties: AttributeMap,
    }
}

packet_serializable! {
    pub struct EntityStatus {
        pub entity_id: VarInt,
        pub logic_op_code: i8, // better name?
    }
}

packet_serializable! {
    pub struct EntityAttach {
        pub entity_id: i32, // not VarInt for whatever reason
        pub vehicle_id: i32,
        pub leash: bool,
    }
}

packet_serializable! {
    pub struct PacketEntityMetadata<'a, M> where &'a M : PacketSerializable {
        pub entity_id: VarInt,
        pub metadata: &'a M,
    }
}

// should probably make the register_packets macro work with generics, but im too lazy
impl<M : PacketSerializable> IdentifiedPacket for PacketEntityMetadata<'_, M> {
    const PACKET_ID: i32 = 0x1c;
}

packet_serializable! {
    pub struct AddEffect {
        pub entity_id: i32 => &VarInt(self.entity_id),
        pub effect_id: PotionEffect,
        pub amplifier: i8,
        pub duration: i32 => &VarInt(self.duration),
        pub hide_particles: bool,
    }
}

packet_serializable! {
    pub struct RemoveEffect {
        pub entity_id: VarInt,
        pub effect_id: u8,
    }
}

packet_serializable! {
    pub struct ChunkData {
        pub chunk_x: i32,
        pub chunk_z: i32,
        pub is_new_chunk: bool,
        pub bitmask: u16,
        pub data: Vec<u8>,
    }
}

packet_serializable! {
    pub struct BlockChange {
        pub block_pos: IVec3 => &BlockPosition(self.block_pos),
        pub block_state: u16 => &VarInt(self.block_state as i32),
    }
}

packet_serializable! {
    pub struct BlockAction {
        pub block_pos: IVec3 => &BlockPosition(self.block_pos),
        pub event_id: u8,
        pub event_data: u8,
        pub block_id: u16 => &VarInt((self.block_id & 4095) as i32),
    }
}

packet_serializable! {
    pub struct SoundEffect {
        pub sound: Sound,
        pub pos_x: f64 => &((self.pos_x * 8.0) as i32),
        pub pos_y: f64 => &((self.pos_y * 8.0) as i32),
        pub pos_z: f64 => &((self.pos_z * 8.0) as i32),
        pub volume: f32,
        pub pitch: f32 => &(f32::clamp(self.pitch * 63.0, 0.0, 255.0) as u8),
    }
}

packet_serializable! {
    pub struct Particles {
        pub particle: Particle,
        pub long_distance: bool,
        pub position: Vec3,
        pub offset: Vec3,
        pub speed: f32,
        pub count: i32,
        // maybe figure out args,
        // not sure if we'll ever need them
    }
}

packet_serializable! {
    pub struct OpenWindow {
        pub window_id: i8,
        pub inventory_type: SizedString<32>,
        pub window_title: ChatComponent,
        pub slot_count: u8,
    }
}

packet_serializable! {
    pub struct CloseWindow {
        pub window_id: i8,
    }
}

packet_serializable! {
    pub struct SetSlot {
        pub window_id: i8,
        pub slot: i16,
        pub item_stack: Option<ItemStack>,
    }
}

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

packet_serializable! {
    pub struct ConfirmTransaction {
        pub window_id: i8,
        pub action_number: i16,
        pub accepted: bool,
    }
}

#[derive(Debug)]
pub struct PlayerAbilities {
    pub invulnerable: bool,
    pub flying: bool,
    pub allow_flying: bool,
    pub creative_mode: bool,
    pub fly_speed: f32,
    pub walk_speed: f32,
}

impl PacketSerializable for PlayerAbilities {
    fn write_size(&self) -> usize {
        1 + self.fly_speed.write_size() + self.walk_speed.write_size()
    }
    fn write(&self, buf: &mut BytesMut) {
        let byte = (self.invulnerable as i8) | ((self.flying as i8) << 1) | ((self.allow_flying as i8) << 2) | ((self.creative_mode as i8) << 3);
        byte.write(buf);
        self.fly_speed.write(buf);
        self.walk_speed.write(buf);
    }
}

#[derive(Debug)]
pub struct Maps {
    pub id: i32,
    pub scale: i8,
    // pub visible_players: Vec<u8>, // bvec4
    pub columns: u8,
    pub rows: u8,
    pub x: u8,
    pub z: u8,
    pub map_data: Vec<u8>
}

impl PacketSerializable for Maps {
    fn write_size(&self) -> usize {
        let mut size = 0;
        size += var_int_size(self.id) + self.scale.write_size() + var_int_size(0) + self.columns.write_size();
        if self.columns > 0 { 
            size += self.rows.write_size() + self.x.write_size() + self.z.write_size() + self.map_data.write_size()
        }
        size
    }
    fn write(&self, buf: &mut BytesMut) {
        VarInt(self.id).write(buf);
        self.scale.write(buf);
        
        // todo visible players
        VarInt(0).write(buf);

        self.columns.write(buf);
        if self.columns > 0 {
            self.rows.write(buf);
            self.x.write(buf);
            self.z.write(buf);
            self.map_data.write(buf);
        }
    }
}

packet_serializable! {
    pub struct TabCompleteReply {
        pub matches: Vec<String>
    }
}

// hard coded render_type, maybe bad idea??
#[derive(Debug)]
pub struct ScoreboardObjective {
    pub objective_name: SizedString<16>,
    pub objective_value: SizedString<64>,
    // pub render_type: &'a str,
    pub mode: i8,
}

impl PacketSerializable for ScoreboardObjective {
    fn write_size(&self) -> usize {
        let mut size = 0;
        size += self.objective_name.write_size() + self.mode.write_size();

        const ADD_OBJECTIVE: i8 = 0;
        const UPDATE_NAME: i8 = 2;

        if self.mode == ADD_OBJECTIVE || self.mode == UPDATE_NAME {
            // inline integer type since its always this
            const S: usize = (*b"integer").len();
            size += self.objective_value.write_size() + var_int_size(S as i32) + S;
        }
        
        size
    }
    fn write(&self, buf: &mut BytesMut) {
        self.objective_name.write(buf);
        self.mode.write(buf);

        const ADD_OBJECTIVE: i8 = 0;
        const UPDATE_NAME: i8 = 2;

        if self.mode == ADD_OBJECTIVE || self.mode == UPDATE_NAME {
            self.objective_value.write(buf);
            PacketSerializable::write(&(*b"integer"), buf)
        }
    }
}

pub struct UpdateScore {
    pub name: SizedString<40>,
    pub objective: SizedString<16>,
    pub value: VarInt,
    pub action: VarInt,
}

impl PacketSerializable for UpdateScore {
    fn write_size(&self) -> usize {
        let mut size = 0;
        size += self.name.write_size() + self.action.write_size() + self.objective.write_size();
        if self.action.0 == 0 {
            size += self.value.write_size()
        }
        size 
    }
    fn write(&self, buf: &mut BytesMut) {
        self.name.write(buf);
        self.action.write(buf);
        self.objective.write(buf);
        
        if self.action.0 == 0 { 
            self.value.write(buf);
        }
    }
}

packet_serializable! {
    pub struct DisplayScoreboard {
        pub position: i8,
        pub score_name: SizedString<16>
    }
}

#[derive(Debug)]
pub struct Teams {
    pub name: SizedString<16>,
    pub display_name: SizedString<32>,
    pub prefix: SizedString<32>,
    pub suffix: SizedString<32>,
    pub name_tag_visibility: SizedString<32>,
    pub color: i8,
    pub players: Vec<SizedString<40>>,
    pub action: i8,
    pub friendly_flags: i8,
}

impl PacketSerializable for Teams {
    fn write_size(&self) -> usize {
        pub const CREATE_TEAM: i8 = 0;
        pub const REMOVE_TEAM: i8 = 1;
        pub const UPDATE_TEAM: i8 = 2;
        pub const ADD_PLAYER: i8 = 3;
        pub const REMOVE_PLAYER: i8 = 4;
        
        let mut size = self.name.write_size() + self.action.write_size();
        
        if self.action == CREATE_TEAM || self.action == UPDATE_TEAM {
            size += 
                self.display_name.write_size() +
                self.prefix.write_size() + 
                self.suffix.write_size() +
                self.friendly_flags.write_size() +
                self.name_tag_visibility.write_size() +
                self.color.write_size()
        }
        if self.action == CREATE_TEAM || self.action == ADD_PLAYER || self.action == REMOVE_PLAYER {
            size += self.players.write_size();
        }
        size
    }
    fn write(&self, buf: &mut BytesMut) {
        pub const CREATE_TEAM: i8 = 0;
        pub const REMOVE_TEAM: i8 = 1;
        pub const UPDATE_TEAM: i8 = 2;
        pub const ADD_PLAYER: i8 = 3;
        pub const REMOVE_PLAYER: i8 = 4;

        self.name.write(buf);
        self.action.write(buf);

        if self.action == CREATE_TEAM || self.action == UPDATE_TEAM {
            self.display_name.write(buf);
            self.prefix.write(buf);
            self.suffix.write(buf);
            self.friendly_flags.write(buf);
            self.name_tag_visibility.write(buf);
            self.color.write(buf);
        }

        if self.action == CREATE_TEAM || self.action == ADD_PLAYER || self.action == REMOVE_PLAYER {
            self.players.write(buf);
        }
    }
}

// test this, it might not work
packet_serializable! {
    pub struct CustomPayload<'a> {
        pub channel: SizedString<20>,
        pub data: &'a [u8]
    }
}

packet_serializable! {
    pub struct Disconnect {
        pub reason: ChatComponent
    }
}

packet_serializable! {
    pub struct PlayerListHeaderFooter {
        pub header: ChatComponent
        pub footer: ChatComponent,
    }
}

#[derive(Debug)]
pub struct PlayerData<'a> {
    pub ping: i32,
    pub game_mode: i32,
    pub profile: &'a GameProfile,
    pub display_name: Option<ChatComponent>
}

pub struct PlayerListItem<'a> {
    pub action: VarInt,
    pub players: &'a [PlayerData<'a>]
}

impl PacketSerializable for PlayerListItem<'_> {

    fn write_size(&self) -> usize {
        const ADD_PLAYER: i32 = 0;
        const UPDATE_GAME_MODE: i32 = 1;
        const UPDATE_LATENCY: i32 = 2;
        const UPDATE_NAME: i32 = 3;
        const REMOVE_PLAYER: i32 = 4;

        let mut size = self.action.write_size() + var_int_size(self.players.len() as i32);

        for player in self.players.iter() {
            match self.action.0 {
                ADD_PLAYER => {
                    size += player.profile.uuid.write_size() + player.profile.username.write_size();

                    let properties = &player.profile.properties;
                    size +=
                        var_int_size(properties.len() as i32) +
                        var_int_size(player.game_mode) +
                        var_int_size(player.ping);

                    for (key, property) in properties.iter() {
                        size += key.write_size() + property.value.write_size() + 1;

                        if let Some(signature) = &property.signature {
                            size += signature.write_size();
                        }
                    }
                }
                UPDATE_GAME_MODE => {
                    size += player.profile.uuid.write_size() + var_int_size(player.game_mode);
                }
                UPDATE_LATENCY => {
                    size += player.profile.uuid.write_size() + var_int_size(player.ping);
                }
                UPDATE_NAME | REMOVE_PLAYER => {
                    size += player.profile.uuid.write_size();
                }
                _ => unreachable!()
            }
            if self.action.0 == ADD_PLAYER || self.action.0 == UPDATE_NAME {
                size += 1;
                if let Some(name) = &player.display_name {
                    size += name.write_size();
                }
            }
        }

        size
    }
    fn write(&self, buf: &mut BytesMut) {
        const ADD_PLAYER: i32 = 0;
        const UPDATE_GAME_MODE: i32 = 1;
        const UPDATE_LATENCY: i32 = 2;
        const UPDATE_NAME: i32 = 3;
        const REMOVE_PLAYER: i32 = 4;

        self.action.write(buf);
        write_var_int(buf, self.players.len() as i32);

        for player in self.players.iter() {
            match self.action.0 {
                ADD_PLAYER => {
                    player.profile.uuid.write(buf);
                    player.profile.username.write(buf);

                    let properties = &player.profile.properties;
                    write_var_int(buf, properties.len() as i32);
                    for (key, property) in properties.iter() {
                        key.write(buf);
                        property.value.write(buf);

                        if let Some(signature) = &property.signature {
                            true.write(buf);
                            signature.write(buf)
                        } else {
                            false.write(buf)
                        }
                    }
                    write_var_int(buf, player.game_mode);
                    write_var_int(buf, player.ping);
                }
                UPDATE_GAME_MODE => {
                    player.profile.uuid.write(buf);
                    write_var_int(buf, player.game_mode);
                }
                UPDATE_LATENCY => {
                    player.profile.uuid.write(buf);
                    write_var_int(buf, player.ping);
                }
                UPDATE_NAME | REMOVE_PLAYER => {
                    player.profile.uuid.write(buf);
                }
                _ => unreachable!()
            }
            if self.action.0 == ADD_PLAYER || self.action.0 == UPDATE_NAME {
                if let Some(name) = &player.display_name {
                    true.write(buf);
                    name.write(buf);
                } else {
                    false.write(buf);
                }
            }
        }
    }
}