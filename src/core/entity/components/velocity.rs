use crate::core::chunk::chunk_grid::ChunkGrid;
use crate::core::chunk::get_chunk_position;
use crate::core::entity::components::transform::Transform;
use crate::core::entity::BevyEntityExt;
use crate::core::network::packets::BytesMutExt;
use crate::core::network::protocol::packed::packed_velocity;
use crate::core::network::protocol::play::clientbound::EntityVelocity;
use crate::core::network::protocol::var_int::VarInt;
use bevy::prelude::*;
use glam::DVec3;

#[derive(Default, Copy, Clone, Component, Deref, DerefMut)]
pub struct Velocity(pub DVec3);

pub fn write_velocity_packet(
    query: Query<(Entity, &Velocity, &Transform), Changed<Velocity>>,
    mut chunks: ResMut<ChunkGrid>,
) {
    for (entity, velocity, transform) in query.iter() {
        let chunk_position = get_chunk_position(transform.position);
    
        if let Some(chunk) = chunks.get_mut(chunk_position) {
            chunk.packet_buffer.write_packet(&EntityVelocity {
                entity_id: VarInt(entity.mc_id()),
                velocity: packed_velocity(**velocity),
            })
        }
    }
}