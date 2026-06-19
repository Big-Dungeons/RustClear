use crate::core::block::block_parameters::Direction;
use crate::core::block::block_rotation::{Rotate, Rotation};
use crate::core::block::Block;
use crate::core::chunk::chunk_grid::ChunkGrid;
use crate::dungeon::rooms::secrets::SecretSpawned;
use bevy::prelude::*;
use glam::IVec3;

#[derive(Component)]
pub struct ChestSecret {
    pub spawn_location: IVec3,
    pub rotation: Rotation,
}

pub(super) fn on_secret_spawn(
    event: On<Insert, SecretSpawned>,
    query: Query<&ChestSecret>,
    mut chunks: ResMut<ChunkGrid>,
    mut commands: Commands,
) {
    if let Ok(secret) = query.get(event.entity) {
        let direction = Direction::North.rotate(secret.rotation);
        chunks.set_block_at(Block::Chest { direction }, secret.spawn_location);
    }
}