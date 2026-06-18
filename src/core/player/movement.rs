use crate::core::chunk::chunk_grid::{ChunkDiff, ChunkGrid};
use crate::core::chunk::{get_chunk_position, Chunk};
use crate::core::entity::components::transform::{OldTransform, Transform};
use crate::core::entity::MobSpawnQueries;
use crate::core::network::packets::{BytesMutExt, PacketEvent};
use crate::core::network::protocol::play::clientbound::{PositionLook, Relative};
use crate::core::network::protocol::play::serverbound::{PlayerLook, PlayerPosition, PlayerPositionLook};
use crate::core::player::{Initialized, PacketReader, PlayerPacketBuffer};
use bevy::prelude::{Changed, DetectChangesMut, Entity, Message, MessageReader, Query, ResMut, With};
use enumset::EnumSet;

#[derive(Message)]
pub struct SetPosition {
    pub client: Entity,
    pub transform: Transform,
    pub relative_flags: EnumSet<Relative>,
}

// in the future, add extra anti-cheat related features here
pub(super) fn handle_set_position(
    mut events: MessageReader<SetPosition>,
    mut query: Query<&mut PlayerPacketBuffer>
) {
    for SetPosition { client, transform, relative_flags } in events.read() {
        let mut packet_buffer = query.get_mut(*client).unwrap();
        packet_buffer.write_packet(&PositionLook {
            x: transform.position.x,
            y: transform.position.y,
            z: transform.position.z,
            yaw: transform.yaw,
            pitch: transform.pitch,
            flags: *relative_flags,
        })
    }
}

// in the far future,
// validate movement so it is legit, then write a message with new position
pub(super) fn handle_incoming_movement_packets(
    // mut player_updates: PacketReader<PlayerUpdate>,
    mut player_positions: PacketReader<PlayerPosition>,
    mut player_looks: PacketReader<PlayerLook>,
    mut player_position_looks: PacketReader<PlayerPositionLook>,
    mut query: Query<&mut Transform>
) {
    // for PacketEvent { client, packet } in update.read() {
    //
    // }
    for PacketEvent { client, packet } in player_looks.read() {
        let mut transform = query.get_mut(*client).unwrap();
        // it won't automatically detect,
        // since im mutating the inner and not replacing it
        transform.set_changed();
        transform.yaw = packet.yaw;
        transform.pitch = packet.pitch;
    }
    for PacketEvent { client, packet } in player_positions.read() {
        let mut transform = query.get_mut(*client).unwrap();
        transform.set_changed();
        transform.position.x = packet.x;
        transform.position.y = packet.y;
        transform.position.z = packet.z;
    }
    for PacketEvent { client, packet } in player_position_looks.read() {
        let mut transform = query.get_mut(*client).unwrap();
        transform.set_changed();
        transform.position.x = packet.x;
        transform.position.y = packet.y;
        transform.position.z = packet.z;
        transform.yaw = packet.yaw;
        transform.pitch = packet.pitch;
    }
}

pub(super) fn transform_change(
    mut player_query: Query<
        (Entity, &Transform, &OldTransform, &mut PlayerPacketBuffer),
        (Changed<Transform>, With<Initialized>)
    >,
    mut chunks: ResMut<ChunkGrid>,
    mob_spawn_queries: MobSpawnQueries,
) {
    for (entity, transform, old_transform, mut packet_buffer) in player_query.iter_mut() {
        let old = get_chunk_position(old_transform.position);
        let new = get_chunk_position(transform.position);

        if old != new {
            if let Some(old_chunk) = chunks.get_mut(old) {
                old_chunk.remove_player(entity)
            }

            if let Some(new_chunk) = chunks.get_mut(new) {
                new_chunk.insert_player(entity)
            }

            ChunkGrid::for_each_diff(
                chunks.bounds,
                new,
                old,
                6,
                |x, z, diff| {
                    let Some(chunk) = chunks.get_mut((x, z)) else {
                        return;
                    };
                    if diff == ChunkDiff::New {
                        chunk.write_chunk_data(x, z, true, &mut packet_buffer);
                        chunk.write_spawn_entities(&mob_spawn_queries);
                    } else {
                        packet_buffer.write_packet(&Chunk::unload_packet(x, z));
                        chunk.write_despawn_entities(&mob_spawn_queries);
                    }
                }
            );
        }
    }
}