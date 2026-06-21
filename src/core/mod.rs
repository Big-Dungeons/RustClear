use bevy::prelude::*;
use crate::core::chunk::chunk_grid::{ChunkGrid, ChunkGridBounds};
use crate::core::entity::MobPlugin;
use crate::core::network::NetworkPlugin;
use crate::core::player::PlayerPlugin;

pub mod chunk;
pub mod block;
pub mod entity;
pub mod network;
pub mod types;
pub mod player;

pub struct CorePlugin {
    pub addr: &'static str,
    pub chunk_bounds: ChunkGridBounds,
}

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(ServerTick(0))
            .insert_resource(ChunkGrid::new(self.chunk_bounds))
            .add_plugins((
                NetworkPlugin {
                    addr: self.addr
                },
                MobPlugin,
                PlayerPlugin,
            ))
            .add_systems(Last, increment_server_tick)
            .add_observer(block::block_entity::on_add_block_entity)
            .add_observer(block::block_entity::on_remove_block_entity)
            .add_systems(PostUpdate, block::block_entity::on_block_entity_change)
        ;
    }
}

#[derive(Resource)]
pub struct ServerTick(i64);

pub fn run_every_ticks<const N: i64>(tick: Res<ServerTick>) -> bool {
    tick.now() % N == 0
}

impl ServerTick {
    pub fn now(&self) -> i64 {
        self.0
    }

    pub fn elapsed_since(&self, tick: i64) -> i64 {
        self.0.saturating_sub(tick)
    }
}

fn increment_server_tick(mut ticks: ResMut<ServerTick>) {
    *ticks = ServerTick(ticks.0 + 1)
}