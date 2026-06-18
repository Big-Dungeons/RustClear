#![allow(unused)]

use crate::core::network::packets::packet_deserializable::PacketDeserializable;
use crate::core::network::packets::IdentifiedPacket;
use crate::core::network::packets::PacketEvent;
use crate::core::network::protocol::block_position::BlockPosition;
use crate::core::network::protocol::sized_string::SizedString;
use crate::core::network::protocol::var_int::{read_var_int, VarInt};
use crate::core::network::Bytes;
use crate::core::network::Entity;
use crate::core::network::MessageWriter;
use crate::core::player::inventory::item_stack::ItemStack;
use crate::register_serverbound_packets;
use crate::App;
use anyhow::bail;
use glam::Vec3;
use macros::{identified_packet, PacketDeserializable};

register_serverbound_packets! {
    PlayPacket;
    
    KeepAlive,
    ChatMessage,
    UseEntity,
    PlayerUpdate,
    PlayerPosition,
    PlayerLook,
    PlayerPositionLook,
    PlayerBlockPlacement,
    HeldItemChange,
    ArmSwing,
    PlayerAction,
    CloseWindow,
    ClickWindow,
    ConfirmTransaction,
    ClientStatus,
}

#[identified_packet(id=0x00)]
#[derive(Debug, Clone, PacketDeserializable)]
pub struct KeepAlive {
    pub id: i32
}

#[identified_packet(id=0x01)]
#[derive(Debug, Clone, PacketDeserializable)]
pub struct ChatMessage {
    pub string: SizedString<256>
}

#[derive(Debug, PartialEq, Copy, Clone, PacketDeserializable)]
pub enum EntityInteractionType {
    Interact,
    Attack,
    InteractAt, // used in armor stands
}

#[identified_packet(id=0x02)]
#[derive(Debug)]
pub struct UseEntity {
    pub entity_id: VarInt,
    pub action: EntityInteractionType,
    pub hit_vec: Option<Vec3>
}

impl PacketDeserializable for UseEntity {
    fn read(buffer: &mut Bytes) -> anyhow::Result<Self> {
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

#[identified_packet(id=0x03)]
#[derive(Debug, Copy, Clone, PacketDeserializable)]
pub struct PlayerUpdate {
    pub on_ground: bool
}

#[identified_packet(id=0x04)]
#[derive(Debug, Copy, Clone, PacketDeserializable)]
pub struct PlayerPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub on_ground: bool
}

#[identified_packet(id=0x05)]
#[derive(Debug, Copy, Clone, PacketDeserializable)]
pub struct PlayerLook {
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool
}

#[identified_packet(id=0x06)]
#[derive(Debug, Copy, Clone, PacketDeserializable)]
pub struct PlayerPositionLook {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool
}


#[identified_packet(id=0x08)]
#[derive(Debug, Clone, PacketDeserializable)]
pub struct PlayerBlockPlacement {
    pub position: BlockPosition,
    pub placed_direction: u8,
    pub item_stack: Option<ItemStack>,
    pub facing_x: i8,
    pub facing_y: i8,
    pub facing_z: i8,
}

#[identified_packet(id=0x09)]
#[derive(Debug, Copy, Clone, PacketDeserializable)]
pub struct HeldItemChange {
    pub slot_id: i16
}

#[identified_packet(id=0x0a)]
#[derive(Debug, Copy, Clone, PacketDeserializable)]
pub struct ArmSwing;

#[derive(Debug)]
pub enum PlayerActionType {
    StartSneaking,
    StopSneaking,
    StopSleeping,
    StartSprinting,
    StopSprinting,
    RidingJump,
    OpenEntityInventory,
}

impl PacketDeserializable for PlayerActionType {
    fn read(buffer: &mut Bytes) -> anyhow::Result<Self> {
        let var_int: VarInt = PacketDeserializable::read(buffer)?;
        Ok({
            match var_int.0 {
                0 => PlayerActionType::StartSneaking,
                1 => PlayerActionType::StopSneaking,
                2 => PlayerActionType::StopSleeping,
                3 => PlayerActionType::StartSprinting,
                4 => PlayerActionType::StopSprinting,
                5 => PlayerActionType::RidingJump,
                6 => PlayerActionType::OpenEntityInventory,
                _ => bail!("failed to read player digging action, invalid index: {}", var_int.0)
            }
        })
    }
}

#[identified_packet(id=0x0b)]
#[derive(Debug, PacketDeserializable)]
pub struct PlayerAction {
    pub entity_id: VarInt,
    pub action: PlayerActionType,
    pub data: VarInt,
}

#[identified_packet(id=0x0d)]
#[derive(Debug, PacketDeserializable)]
pub struct CloseWindow {
    pub window_id: i8
}

#[derive(Debug, Copy, Clone, PacketDeserializable)]
pub enum ClickMode {
    NormalClick,
    ShiftClick,
    NumberKey,
    MiddleClick,
    Drop,
    Drag,
    DoubleClick,
}

#[identified_packet(id=0x0e)]
#[derive(Debug, PacketDeserializable)]
pub struct ClickWindow {
    pub window_id: i8,
    pub slot_id: i16,
    pub used_button: i8,
    pub action_number: i16,
    pub mode: ClickMode,
    pub clicked_item: Option<ItemStack>,
}

#[identified_packet(id=0x0f)]
#[derive(Debug, PacketDeserializable)]
pub struct ConfirmTransaction {
    pub window_id: i8,
    pub action_number: i16,
    pub accepted: bool,
}

#[identified_packet(id=0x16)]
pub enum ClientStatus {
    PerformRespawn,
    RequestStats,
    OpenInventory,
}

impl PacketDeserializable for ClientStatus {
    fn read(buffer: &mut Bytes) -> anyhow::Result<Self> {
        let var_int: VarInt = PacketDeserializable::read(buffer)?;
        Ok({
            match var_int.0 {
                0 => Self::PerformRespawn,
                1 => Self::RequestStats,
                2 => Self::OpenInventory,
                _ => bail!("failed to read client status, invalid index: {}", var_int.0)
            }
        })
    }
}