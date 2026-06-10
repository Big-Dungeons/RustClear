use crate::block::block_metadata::BlockFieldMetadata;
use crate::block::block_parameters::HorizontalDirection;
use crate::block::block_rotation::{Rotate, Rotation};
use crate::block::Block;
use crate::chunk::chunk_grid::ChunkGrid;
use crate::entity::Transform;
use crate::player::Player;
use crate::TEST_WORLD;
use bevy::app::{App, Startup};
use bevy::prelude::{Add, DetectChangesMut, On, Plugin, Query, ResMut};
use glam::dvec3;
pub struct DebugWorld;

// temporary
impl Plugin for DebugWorld {
    fn build(&self, app: &mut App) {
        if !TEST_WORLD {
            return;
        }

        app.add_systems(Startup, load).add_observer(spawn_player);
    }
}

fn load(
    mut chunks: ResMut<ChunkGrid>
) {
    for x in -100..0 {
        for z in -100..0 {
            chunks.set_block_at(Block::Stone, (x, 0, z))
        }
    }

    let blocks: Vec<_> = (0..4).map(|meta| {
        Block::FenceGate { direction: HorizontalDirection::from_meta(meta), open: false, powered: false }
    }).collect();

    for (index, block) in blocks.iter().enumerate() {
        chunks.set_block_at(*block, (-2 + index as i32 * -2, 1, -2))
    }

    for (index, block) in blocks.iter().enumerate() {
        let block = block.rotate(Rotation::Clockwise90);
        chunks.set_block_at(block, (-2 + index as i32 * -2, 1, -4));
    }
}

fn spawn_player(
    event: On<Add, Player>,
    mut query: Query<&mut Transform>
) {
    let mut transform = query.get_mut(event.entity).unwrap();
    *transform = Transform {
        position: dvec3(-1.0, 3.0, -1.0),
        yaw: 135.0,
        pitch: 0.0,
    };
    transform.set_changed();
}