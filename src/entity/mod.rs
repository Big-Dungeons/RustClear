pub mod entity_metadata;
pub mod object_metadata;
pub mod components;
mod entity_metadata_serializable;

use crate::chunk::chunk_grid::{ChunkDiff, ChunkGrid};
use crate::chunk::get_chunk_position;
use crate::entity::components::riding::Riding;
use crate::entity::entity_metadata::EntityMetadata;
use crate::entity::object_metadata::ObjectMetadata;
use crate::network::packets::BytesMutExt;
use crate::network::protocol::packed::{packed_position, packed_rotation};
use crate::network::protocol::play::clientbound::{DestroyEntity, EntityAttach, EntityTeleport, EntityYawRotate, SpawnMob, SpawnObject};
use crate::network::protocol::var_int::VarInt;
use crate::player::PlayerPacketBuffer;
use bevy::ecs::entity::{Entities, EntityIndex};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bytes::BytesMut;
use components::transform;
use components::transform::OldTransform;
use components::transform::Transform;
use glam::I16Vec3;

pub trait BevyEntityExt {
    fn mc_id(&self) -> i32;
    fn from_mc_id(id: i32, entities: &Entities) -> Entity;
}

impl BevyEntityExt for Entity {
    fn mc_id(&self) -> i32 {
        self.index_u32() as i32
    }
    fn from_mc_id(id: i32, entities: &Entities) -> Entity {
        // should maybe err if invalid?
        entities
            .resolve_from_index(EntityIndex::from_raw_u32(id as u32).unwrap())
            .entity()
    }
}

pub enum MobType {
    Entity(EntityMetadata),
    Object(ObjectMetadata),
}

#[derive(Component)]
pub struct Mob {
    pub mob_type: MobType,
}

impl Mob {
    pub fn new(metadata: impl Into<EntityMetadata>) -> Self {
        Self {
            mob_type: MobType::Entity(metadata.into())
        }
    }

    pub fn new_object(metadata: ObjectMetadata) -> Self {
        Self {
            mob_type: MobType::Object(metadata)
        }
    }

    pub fn write_spawn_packet(&self, entity: Entity, transform: &Transform, buffer: &mut BytesMut) {
        match self.mob_type {
            MobType::Entity(metadata) => {
                buffer.write_packet(&SpawnMob {
                    entity_id: VarInt(entity.mc_id()),
                    entity_variant: metadata.get_variant(),
                    position: packed_position(transform.position),
                    yaw: packed_rotation(transform.yaw),
                    pitch: packed_rotation(transform.pitch),
                    head_yaw: packed_rotation(transform.yaw),
                    velocity: I16Vec3::ZERO,
                    metadata,
                });
                buffer.write_packet(&EntityYawRotate {
                    entity_id: VarInt(entity.mc_id()),
                    yaw: packed_rotation(transform.yaw),
                });
            }
            MobType::Object(metadata) => {
                buffer.write_packet(&SpawnObject {
                    entity_id: VarInt(entity.mc_id()),
                    variant: metadata.get_variant(),
                    position: packed_position(transform.position),
                    pitch: packed_rotation(transform.pitch),
                    yaw: packed_rotation(transform.yaw),
                    data: metadata.get_data(),
                    velocity: Default::default(),
                })
            }
        }
    }

    pub fn write_despawn_packet(&self, entity: Entity, buffer: &mut BytesMut) {
        buffer.write_packet(&DestroyEntity {
            entity_id: VarInt(entity.mc_id()),
        });
    }

    pub fn write_update_packets(&self, entity: Entity, transform: &Transform, buffer: &mut BytesMut) {
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

#[derive(SystemParam)]
pub struct MobSpawnQueries<'w, 's> {
    pub mob_query: Query<'w, 's, (&'static Mob, &'static Transform)>,
    pub riding_query: Query<'w, 's, &'static Riding>
}

fn on_mob_add(
    event: On<Add, Mob>,
    mob_query: Query<(&Mob, &Transform, Option<&Riding>)>,
    mut player_query: Query<&mut PlayerPacketBuffer>,
    mut chunks: ResMut<ChunkGrid>,
    mut commands: Commands,
) {
    let (mob, transform, riding) = mob_query
        .get(event.entity)
        .expect("mob must be added with transform");

    commands
        .entity(event.entity)
        .insert(OldTransform(*transform));

    let chunk_position = get_chunk_position(transform.position);

    if let Some(chunk) = chunks.get_mut(chunk_position) {
        chunk.insert_entity(event.entity)
    }

    chunks.for_each_in_view(
        chunk_position,
        6,
        |chunk, _, _| {
            for player in chunk.players.iter() {
                let mut buffer = player_query
                    .get_mut(*player)
                    .unwrap();

                mob.write_spawn_packet(event.entity, transform, &mut buffer);
                if let Some(riding) = riding {
                    buffer.write_packet(&EntityAttach {
                        entity_id: event.entity.mc_id(),
                        vehicle_id: riding.mc_id(),
                        leash: false,
                    })
                }
            }
        }
    )
}

fn on_mob_remove(
    event: On<Remove, Mob>,
    mob_query: Query<(&Mob, &Transform)>,
    mut player_query: Query<&mut PlayerPacketBuffer>,
    mut chunks: ResMut<ChunkGrid>,
) {
    let (mob, transform) = mob_query.get(event.entity).unwrap();
    let chunk_position = get_chunk_position(transform.position);

    if let Some(chunk) = chunks.get_mut(chunk_position) {
        chunk.remove_entity(event.entity)
    }

    chunks.for_each_in_view(
        chunk_position,
        6,
        |chunk, _, _| {
            for player in chunk.players.iter() {
                let mut buffer = player_query.get_mut(*player).unwrap();
                mob.write_despawn_packet(event.entity, &mut buffer);
            }
        }
    )
}

fn on_mob_move(
    mob_query: Query<(Entity, &Mob, &Transform, &OldTransform, Option<&Riding>)>,
    mut player_query: Query<&mut PlayerPacketBuffer>,
    mut chunks: ResMut<ChunkGrid>,
) {
    for (entity, mob, transform, old_transform, riding) in mob_query.iter() {
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
            if riding.is_none() {
                mob.write_update_packets(entity, transform, &mut new_chunk.packet_buffer);
            }
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
                                mob.write_spawn_packet(entity, transform, &mut buffer);
                                // might be issues, because of ordering,
                                // however there isn't plans for riding entities to move I think
                                if let Some(riding) = riding {
                                    buffer.write_packet(&EntityAttach {
                                        entity_id: entity.mc_id(),
                                        vehicle_id: riding.mc_id(),
                                        leash: false,
                                    })
                                }
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
            .add_observer(on_mob_remove)
            .add_systems(Update, (
                components::interactable::handle_entity_interactable,
            ))
            .add_systems(PostUpdate, (
                components::riding::update_transform,
                on_mob_move
            ).chain())
            .add_systems(Last, transform::set_old_transform);
    }
}
