use bevy::prelude::*;
use crate::core::network::packets::BytesMutExt;
use crate::core::network::protocol::play::clientbound::Chat;
use crate::core::player::{GlobalPacketBuffer, Username};
use crate::dungeon::DungeonState;

#[derive(Component, Deref, Default)]
pub struct ReadyStatus(pub bool);

#[derive(EntityEvent)]
pub struct PlayerReadyEvent {
    pub entity: Entity,
}

pub(super) fn on_player_ready(
    event: On<PlayerReadyEvent>,
    mut player_query: Query<(Entity, &mut ReadyStatus, &Username)>,
    mut global_packet_buffer: ResMut<GlobalPacketBuffer>,
    state: ResMut<State<DungeonState>>,
    mut next_state: ResMut<NextState<DungeonState>>,
) {
    assert!(!matches!(state.get(), DungeonState::Started { .. }), "tried to ready up when dungeon has already started");
    let mut should_start = true;

    for (entity, mut ready, username) in player_query.iter_mut() {
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
        next_state.set(DungeonState::Starting { starts_in_ticks: 100 });
    } else {
        next_state.set(DungeonState::NotStarted);
    }
}