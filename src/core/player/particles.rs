use crate::core::chunk::chunk_grid::ChunkGrid;
use crate::core::network::packets::BytesMutExt;
use crate::core::network::protocol::play::clientbound::Particles;
use crate::core::types::particles::Particle;
use bevy::prelude::*;
use glam::Vec3;

#[derive(Message, Copy, Clone)]
pub struct ParticleEvent {
    pub particle: Particle,
    pub position: Vec3,
    pub offset: Vec3,
    pub speed: f32,
    pub count: i32,
}

pub(super) fn handle_particles(
    mut events: MessageReader<ParticleEvent>,
    mut chunks: ResMut<ChunkGrid>,
) {
    for ParticleEvent { particle, position, offset, speed, count, } in events.read() {
        if let Some(chunk) = chunks.get_mut_from_world(position.floor().as_ivec3()) {
            chunk.packet_buffer.write_packet(&Particles {
                particle: *particle,
                has_arguments: false,
                position: *position,
                offset: *offset,
                speed: *speed,
                count: *count,
                arguments: &[],
            })
        }
    }
}