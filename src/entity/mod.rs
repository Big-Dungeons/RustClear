pub mod entity_metadata;
pub mod components;
mod entity_metadata_serializable;

use crate::chunk::chunk_grid::{ChunkDiff, ChunkGrid};
use crate::chunk::get_chunk_position;
use crate::entity::entity_metadata::EntityMetadata;
use crate::network::packets::BytesMutExt;
use crate::network::protocol::packed::{packed_position, packed_rotation};
use crate::network::protocol::play::clientbound::{DestroyEntity, EntityTeleport, EntityYawRotate, SpawnMob};
use crate::network::protocol::var_int::VarInt;
use crate::player::PlayerPacketBuffer;
use bevy::prelude::*;
use bytes::BytesMut;
use components::transform;
use components::transform::OldTransform;
use components::transform::Transform;
use glam::I16Vec3;

pub trait BevyEntityExt {
    fn mc_id(&self) -> i32;
}

impl BevyEntityExt for Entity {
    fn mc_id(&self) -> i32 {
        self.index_u32() as i32
    }
}

#[derive(Component)]
pub struct Mob {
    pub metadata: EntityMetadata,
}

impl Mob {
    pub fn new(metadata: impl Into<EntityMetadata>) -> Self {
        Self {
            metadata: metadata.into()
        }
    }

    pub fn write_spawn_packet(&self, entity: Entity, transform: &Transform, buffer: &mut BytesMut) {
        buffer.write_packet(&SpawnMob {
            entity_id: VarInt(entity.mc_id()),
            entity_variant: self.metadata.get_variant(),
            position: packed_position(transform.position),
            yaw: packed_rotation(transform.yaw),
            pitch: packed_rotation(transform.pitch),
            head_yaw: packed_rotation(transform.yaw),
            velocity: I16Vec3::ZERO,
            metadata: self.metadata,
        });
        buffer.write_packet(&EntityYawRotate {
            entity_id: VarInt(entity.mc_id()),
            yaw: packed_rotation(transform.yaw),
        });
    }

    pub fn write_despawn_packet(&self, entity: Entity, buffer: &mut BytesMut) {
        buffer.write_packet(&DestroyEntity {
            entity_id: VarInt(entity.mc_id()),
        });
    }

    pub fn write_update_position_packets(&self, entity: Entity, transform: &Transform, buffer: &mut BytesMut) {
        buffer.write_packet(&EntityTeleport {
            entity_id: VarInt(entity.mc_id()),
            position: packed_position(transform.position),
            yaw: packed_rotation(transform.yaw),
            pitch: packed_rotation(transform.pitch),
            on_ground: true,
        });
        buffer.write_packet(&EntityYawRotate {
            entity_id: VarInt(entity.mc_id()),
            yaw: packed_rotation(transform.yaw),
        })
    }
}

fn on_mob_add(
    event: On<Add, Mob>,
    mob_query: Query<&Transform>,
    mut chunks: ResMut<ChunkGrid>,
    mut commands: Commands,
) {
    let transform = mob_query
        .get(event.entity)
        .expect("mob must be added with transform");

    commands
        .entity(event.entity)
        .insert(OldTransform(*transform));

    let chunk_position = get_chunk_position(transform.position);

    if let Some(chunk) = chunks.get_mut(chunk_position) {
        chunk.insert_entity(event.entity)
    }
}

fn on_mob_move(
    mob_query: Query<(Entity, &Mob, &Transform, &OldTransform)>,
    mut player_query: Query<&mut PlayerPacketBuffer>,
    mut chunks: ResMut<ChunkGrid>,
) {
    for (entity, mob, transform, old_transform) in mob_query.iter() {
        if *transform == old_transform.0 {
            continue
        }

        let chunk_position = get_chunk_position(transform.position);
        let old_position = get_chunk_position(old_transform.position);
        let is_diff_chunk = chunk_position != old_position;

        if let Some(new_chunk) = chunks.get_mut(chunk_position) {
            if is_diff_chunk {
                new_chunk.insert_entity(entity)
            }
            mob.write_update_position_packets(entity, transform, &mut new_chunk.packet_buffer);
        };
        if is_diff_chunk {
            if let Some(old_chunk) = chunks.get_mut(old_position) {
                old_chunk.remove_entity(entity)
            }
            
            ChunkGrid::for_each_diff(
                chunks.bounds,
                chunk_position,
                old_position,
                6,
                |x, z, diff| {
                    let Some(chunk) = chunks.get_mut((x, z)) else {
                        return;
                    };
                    
                    match diff {
                        ChunkDiff::New => {
                            for player in chunk.players.iter() {
                                let mut buffer = player_query.get_mut(*player).unwrap();
                                mob.write_spawn_packet(entity, transform, &mut buffer)
                            }
                        },
                        ChunkDiff::Old => {
                            for player in chunk.players.iter() {
                                let mut buffer = player_query.get_mut(*player).unwrap();
                                mob.write_despawn_packet(entity, &mut buffer)
                            }
                        }
                    }
                }
            )
        };
    }
}

pub struct MobPlugin;

impl Plugin for MobPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(on_mob_add)
            .add_systems(Update, on_mob_move)
            .add_systems(Last, transform::set_old_transform);
    }
}
