use crate::core::block::Block;
use crate::core::chunk::chunk_grid::ChunkGrid;
use crate::core::entity::components::transform::Transform;
use crate::core::player::Player;
use bevy::app::{App, Startup};
use bevy::prelude::{Add, DetectChangesMut, On, Plugin, Query, ResMut};
use glam::dvec3;

// purpose: flat world to test stuff without loading a dungeon
pub struct DebugWorld;


impl Plugin for DebugWorld {
    fn build(&self, app: &mut App) {
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