#![allow(unused)]

use crate::network::packets::packet_deserializable::PacketDeserializable;
use crate::network::packets::IdentifiedPacket;
use crate::network::packets::PacketEvent;
use crate::network::protocol::block_position::BlockPosition;
use crate::network::protocol::sized_string::SizedString;
use crate::network::protocol::var_int::{read_var_int, VarInt};
use crate::network::Bytes;
use crate::network::Entity;
use crate::network::MessageWriter;
use crate::player::inventory::item_stack::ItemStack;
use crate::register_serverbound_packets;
use crate::App;
use anyhow::bail;
use macros::{identified_packet, PacketDeserializable};

register_serverbound_packets! {
    PlayPacket;
    ChatMessage,
    PlayerUpdate,
    PlayerPosition,
    PlayerLook,
    PlayerPositionLook,
    PlayerBlockPlacement,
    HeldItemChange,
    ArmSwing,
    PlayerAction,
    ClickWindow,
}

#[identified_packet(id=0x01)]
#[derive(Debug, Clone, PacketDeserializable)]
pub struct ChatMessage {
    pub string: SizedString<256>
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
                6 => PlayerActionType::OpenInventory,
                _ => bail!("failed to read player digging action, invalid index: {}", var_int.0)
            }
        })
    }
}

#[identified_packet(id=0x0b)]
#[derive(PacketDeserializable)]
pub struct PlayerAction {
    pub entity_id: VarInt,
    pub action: PlayerActionType,
    pub data: VarInt,
}

#[derive(PacketDeserializable)]
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
#[derive(PacketDeserializable)]
pub struct ClickWindow {
    pub window_id: i8,
    pub slot_id: i16,
    pub used_button: i8,
    pub action_number: i16,
    pub mode: ClickMode,
    pub clicked_item: Option<ItemStack>,
}