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
            .insert_resource(ChunkGrid::new(self.chunk_bounds))
            .add_plugins((
                NetworkPlugin {
                    addr: self.addr
                },
                MobPlugin,
                PlayerPlugin,
            ));
    }
}