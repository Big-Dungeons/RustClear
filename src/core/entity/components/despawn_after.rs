use bevy::prelude::*;
use crate::core::entity::SpawnedOnTick;
use crate::core::ServerTick;

// needs TicksExisted (exists for all mobs)
#[derive(Component, Deref)]
pub struct DespawnAfter {
    pub ticks: i64
}

pub fn handle_despawn_after(
    query: Query<(Entity, &SpawnedOnTick, &DespawnAfter)>,
    server_tick: Res<ServerTick>,
    mut commands: Commands,
) {
    for (entity, ticks_existed, despawn_after) in query.iter() {
        if server_tick.elapsed_since(**ticks_existed) > **despawn_after {
            commands
                .entity(entity)
                .despawn();
        }
    }
}

