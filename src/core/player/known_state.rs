use crate::core::network::packets::PacketEvent;
use crate::core::network::protocol::play::serverbound::{PlayerAction, PlayerActionType};
use crate::core::player::PacketReader;
use bevy::prelude::{Component, Query};

// add future stuff, other ac related stuff
#[derive(Default, Component)]
pub struct KnownState {
    pub sprinting: bool,
    pub sneaking: bool,
}

pub(super) fn handle_player_action(
    mut packets: PacketReader<PlayerAction>,
    mut query: Query<&mut KnownState>
) {
    for PacketEvent { client, packet } in packets.read() {
        let mut state = query.get_mut(*client).unwrap();
        match packet.action {
            PlayerActionType::StartSneaking => state.sneaking = true,
            PlayerActionType::StopSneaking => state.sneaking = false,
            PlayerActionType::StartSprinting => state.sprinting = true,
            PlayerActionType::StopSprinting => state.sprinting = false,
            // PlayerActionType::OpenInventory => {}
            _ => {}
        }
    }
}