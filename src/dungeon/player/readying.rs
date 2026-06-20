use crate::core::network::packets::BytesMutExt;
use crate::core::network::protocol::play::clientbound::Chat;
use crate::core::player::{GlobalPacketBuffer, Username};
use crate::dungeon::{DungeonState, DungeonTimer, NextDungeonState};
use bevy::prelude::*;

#[derive(Component, Deref, Default)]
pub struct ReadyStatus(pub bool);

#[derive(EntityEvent)]
pub struct PlayerReadyEvent {
    pub entity: Entity,
}

pub(super) fn on_player_ready(
    event: On<PlayerReadyEvent>,
    mut player_query: Query<(
        Entity,
        &Username,
        &mut ReadyStatus,
    )>,
    mut global_packet_buffer: ResMut<GlobalPacketBuffer>,
    mut state: NextDungeonState,
    mut dungeon_timer: DungeonTimer,
) {
    let mut should_start = true;

    for (entity,  username, mut ready) in player_query.iter_mut() {
        if entity == event.entity {
            let is_ready = !ready.0;
            let message = format!("§7{} {}!", username.0, if is_ready { "§ais now ready" } else { "§cis no longer ready" });
            global_packet_buffer.write_packet(&Chat::new(&message));
            *ready = ReadyStatus(is_ready);
        }

        if !ready.0 {
            should_start = false
        }
    }

    if should_start {
        state.set(DungeonState::Starting);
        dungeon_timer.update_to_now();
    } else {
        state.set(DungeonState::NotStarted);
    }
}