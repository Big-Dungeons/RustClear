use crate::inventory::item_stack::ItemStack;
use crate::network::binary::var_int::VarInt;
use crate::network::packets::packet::ProcessPacket;
use crate::network::packets::packet_deserialize::PacketDeserializable;
use crate::network::protocol::play::serverbound::ClientStatus::{OpenInventory, PerformRespawn, RequestStats};
use crate::register_serverbound_packets;
use crate::types::block_position::BlockPosition;
use crate::types::sized_string::SizedString;
use anyhow::bail;
use blocks::packet_deserializable;
use bytes::Buf;
use glam::{IVec3, Vec3};

register_serverbound_packets! {
    Play;
    KeepAlive = 0x00;
    ChatMessage = 0x01;
    UseEntity = 0x02;
    PlayerUpdate = 0x03;
    PlayerPosition = 0x04;
    PlayerLook = 0x05;
    PlayerPositionLook = 0x06;
    PlayerDigging = 0x07;
    PlayerBlockPlacement = 0x08;
    HeldItemChange = 0x09;
    ArmSwing = 0x0a;
    PlayerAction = 0x0b;
    // SteerVehicle = 0x0c;
    CloseWindow = 0x0d;
    ClickWindow = 0x0e;
    ConfirmTransaction = 0x0f;
    // CreativeInventoryAction = 0x10;
    // EnchantItem = 0x11;
    // SetSign = 0x12;
    // ClientAbilities = 0x13;
    TabComplete = 0x14;
    ClientSettings = 0x15;
    ClientStatus = 0x16;
    // CustomPayload = 0x17;
    // SpectateTeleport = 0x18;
    // ResourcePackStatus = 0x19;
}

packet_deserializable! {
    pub struct KeepAlive {
        pub id: i32,
    }
}

packet_deserializable! {
    pub struct ChatMessage {
        pub message: SizedString<100>
    }
}

packet_deserializable! {
    #[derive(Debug, PartialEq, Copy, Clone)]
    pub enum EntityInteractionType {
        Interact,
        Attack,
        InteractAt, // used in armor stands
    }
}

pub struct UseEntity {
    pub entity_id: VarInt,
    pub action: EntityInteractionType,
    pub hit_vec: Option<Vec3>
}

impl PacketDeserializable for UseEntity {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        let entity_id: VarInt = VarInt::read(buffer)?;
        let action: EntityInteractionType = EntityInteractionType::read(buffer)?;
        let hit_vec = if action == EntityInteractionType::InteractAt { 
            Some(Vec3::new(
                f32::read(buffer)?,
                f32::read(buffer)?,
                f32::read(buffer)?,
            ))
        } else {
            None
        };
        Ok(Self {
            entity_id,
            action,
            hit_vec, 
        })
    }
}

packet_deserializable! {
    pub struct PlayerUpdate {
         pub on_ground: bool
    }
}

packet_deserializable! {
    pub struct PlayerPosition {
        pub x: f64,
        pub y: f64,
        pub z: f64,
        pub on_ground: bool,
    }    
}

packet_deserializable! {
    pub struct PlayerLook {
        pub yaw: f32,
        pub pitch: f32,
        pub on_ground: bool,
    }
}

packet_deserializable! {
    pub struct PlayerPositionLook {
        pub x: f64,
        pub y: f64,
        pub z: f64,
        pub yaw: f32,
        pub pitch: f32,
        pub on_ground: bool,
    }
}

packet_deserializable! {
    pub enum PlayerDiggingAction {
        StartDestroyBlock,
        AbortDestroyBlock,
        FinishDestroyBlock,
        DropAllItem, // < probably won't need these
        DropItem,    // <
        ReleaseUseItem // bow
    }
}


packet_deserializable! {
    pub struct PlayerDigging {
        pub action: PlayerDiggingAction,
        pub position: BlockPosition,
        pub direction: i8,
    }
}

packet_deserializable! {
    pub struct PlayerBlockPlacement {
        pub position: BlockPosition,
        pub placed_direction: i8,
        pub item_stack: Option<ItemStack>,
        pub facing_x: i8,
        pub facing_y: i8,
        pub facing_z: i8,
    }
}

packet_deserializable! {
    pub struct HeldItemChange {
        // for some reason this is a short
        pub slot_id: i16,
    }
}

packet_deserializable! {
    pub struct ArmSwing;
}

pub enum PlayerActionType {
    StartSneaking,
    StopSneaking,
    StopSleeping,
    StartSprinting,
    StopSprinting,
    RidingJump,
    OpenInventory,
}

impl PacketDeserializable for PlayerActionType {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        let var_int: VarInt = PacketDeserializable::read(buffer)?;
        Ok({
            match var_int.0 {
                0 => PlayerActionType::StartSneaking,
                1 => PlayerActionType::StopSneaking,
                2 => PlayerActionType::StopSleeping,
                3 => PlayerActionType::StartSprinting,
                4 => PlayerActionType::StopSprinting,
                5 => PlayerActionType::RidingJump,
                6 => PlayerActionType::OpenInventory,
                _ => bail!("failed to read player digging action, invalid index: {}", var_int.0)
            }
        })
    }
}

packet_deserializable! {
    pub struct PlayerAction {
        pub entity_id: VarInt,
        pub action: PlayerActionType,
        pub data: VarInt,
    }
}


packet_deserializable! {
    pub struct CloseWindow {
        pub window_id: u8
    }
}

packet_deserializable! {
    pub enum ClickMode {
        NormalClick,
        ShiftClick,
        NumberKey,
        MiddleClick,
        Drop,
        Drag,
        DoubleClick,
    }
}

packet_deserializable! {
    pub struct ClickWindow {
        pub window_id: i8,
        pub slot_id: i16,
        pub used_button: i8,
        pub action_number: i16,
        pub mode: ClickMode,
        pub clicked_item: Option<ItemStack>,
    }
}

packet_deserializable! {
    pub struct ConfirmTransaction {
        pub window_id: i8,
        pub action_number: i16,
        pub accepted: bool,
    }
}

pub struct TabComplete {
    pub message: String,
    pub target_block: Option<IVec3>
}

impl PacketDeserializable for TabComplete {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        Ok(Self {
            message: PacketDeserializable::read(buffer)?,
            target_block: {
                if u8::read(buffer)? != 0 {
                    Some(BlockPosition::read(buffer)?.0)
                } else { 
                    None 
                }
            },
        })
    }
}

packet_deserializable! {
    pub struct ClientSettings {
        pub lang: SizedString<7>,
        pub view_distance: i8,
        pub chat_mode: i8,
        pub chat_colors: bool,
        pub skin_parts: u8,
    }
}

pub enum ClientStatus {
    PerformRespawn,
    RequestStats,
    OpenInventory,
}

impl PacketDeserializable for ClientStatus {
    fn read(buffer: &mut impl Buf) -> anyhow::Result<Self> {
        let var_int: VarInt = PacketDeserializable::read(buffer)?;
        Ok({
            match var_int.0 {
                0 => PerformRespawn,
                1 => RequestStats,
                2 => OpenInventory,
                _ => bail!("failed to read client status, invalid index: {}", var_int.0)
            }
        })
    }
}