pub mod inventory;
pub mod known_state;
pub mod movement;
pub mod interact;

use crate::chunk::chunk_grid::ChunkGrid;
use crate::chunk::get_chunk_position;
use crate::entity::components::transform::{OldTransform, Transform};
use crate::entity::Mob;
use crate::network::client::ClientId;
use crate::network::packets::{BytesMutExt, PacketEvent};
use crate::network::protocol::play::clientbound::{ConfirmTransaction, PositionLook};
use crate::network::protocol::play::serverbound;
use crate::network::{recv_network_messages, NetworkSender};
use crate::player::inventory::{InventoryPlugin, SyncInventory};
use crate::player::movement::SetPosition;
use crate::player::interact::{PlayerInteractEntity, PlayerRightClick};
use bevy::app::{App, First, PostUpdate, PreUpdate};
use bevy::prelude::{
    Commands, Component, Deref, DerefMut, Entity, IntoScheduleConfigs, Last, Message,
    MessageReader, Plugin, Query, Res, ResMut, Update, With, Without,
};
use bytes::BytesMut;

// marker
#[derive(Component)]
pub struct Player;

// marker
#[derive(Component)]
pub struct Initialized;

#[derive(Component, Deref)]
pub struct Username(pub String);

#[derive(Component, Deref)]
pub struct Uuid(pub uuid::Uuid);

#[derive(Component, Deref, DerefMut)]
pub struct PlayerPacketBuffer(pub BytesMut);

#[derive(Message)]
pub struct PlayerJoinEvent(pub Entity);

type PacketReader<'a, 'b, T> = MessageReader<'a, 'b, PacketEvent<T>>;

fn process_player_join(
    mut events: MessageReader<'_, '_, PlayerJoinEvent>,
    mut player_query: Query<(&Transform, &mut PlayerPacketBuffer)>,
    mob_query: Query<(&Transform, &Mob)>,
    mut chunks: ResMut<ChunkGrid>,
    mut commands: Commands,
) {
    for PlayerJoinEvent(entity) in events.read() {
        let (transform, mut packet_buffer) = player_query.get_mut(*entity).unwrap();
        // already wrote join game

        // should maybe inline it. however, might as well reuse it
        commands.trigger(SyncInventory { entity: *entity });

        let position = get_chunk_position(transform.position);
        if let Some(chunk) = chunks.get_mut(position) {
            chunk.insert_player(*entity)
        }

        chunks.for_each_in_view(position, 8, |chunk, x, z| {
            chunk.write_chunk_data(x, z, true, &mut packet_buffer);
            for entity in &chunk.entities {
                let (transform, mob) = mob_query.get(*entity).unwrap();
                mob.write_spawn_packet(*entity, transform, &mut packet_buffer);
            }
        });

        packet_buffer.write_packet(&PositionLook {
            x: transform.position.x,
            y: transform.position.y,
            z: transform.position.z,
            yaw: transform.yaw,
            pitch: transform.pitch,
            flags: Default::default(),
        });

        // temp workaround to fix player sending position packets before player is connected
        packet_buffer.write_packet(&ConfirmTransaction {
            window_id: 0,
            action_number: -1,
            accepted: false,
        });

        commands.entity(*entity).insert(OldTransform(*transform));
    }
}

fn copy_chunk_packet_buffers(
    mut player_query: Query<(&Transform, &mut PlayerPacketBuffer), With<Player>>,
    mut chunks: ResMut<ChunkGrid>,
) {
    for (transform, mut packet_buffer) in player_query.iter_mut() {
        let position = get_chunk_position(transform.position);
        chunks.for_each_in_view(position, 8, |chunk, _, _| {
            packet_buffer.extend_from_slice(&chunk.packet_buffer);
        });
    }
}

fn clear_chunk_packet_buffers(mut chunks: ResMut<ChunkGrid>) {
    for chunk in chunks.chunks.iter_mut() {
        chunk.packet_buffer.clear();
    }
}

fn flush_packets(mut query: Query<(&ClientId, &mut PlayerPacketBuffer)>, tx: Res<NetworkSender>) {
    for (client_id, mut buffer) in query.iter_mut() {
        if buffer.is_empty() {
            continue;
        }
        let _ = tx.send(buffer.get_packet_message(*client_id));
    }
}

fn keep_alive(mut query: Query<&mut PlayerPacketBuffer>) {
    for mut buffer in query.iter_mut() {
        buffer.write_packet(&ConfirmTransaction {
            window_id: 0,
            action_number: 0,
            accepted: true,
        });
    }
}

// temp ?
fn recv_transaction(
    mut packet_reader: PacketReader<serverbound::ConfirmTransaction>,
    query: Query<(), (With<Player>, Without<Initialized>)>,
    mut commands: Commands,
) {
    for PacketEvent { client, packet } in packet_reader.read() {
        if packet.action_number == -1 && query.contains(*client) {
            commands
                .entity(*client)
                .insert(Initialized);
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<SetPosition>()
            .add_message::<PlayerRightClick>()
            .add_message::<PlayerInteractEntity>()
            .add_plugins(InventoryPlugin)
            .add_systems(First, process_player_join.after(recv_network_messages))
            .add_systems(
                PreUpdate,
                (
                    keep_alive,
                    recv_transaction,
                    movement::handle_incoming_movement_packets,
                    interact::handle_block_interact,
                    interact::handle_use_entity,
                    known_state::handle_player_action,
                    inventory::handle_click_window,
                ),
            )
            .add_systems(Update, movement::transform_change)
            .add_systems(
                PostUpdate,
                (
                    movement::handle_set_position,
                    interact::clear_sent_interacts,

                ),
            )
            .add_systems(Last, (
                copy_chunk_packet_buffers,
                clear_chunk_packet_buffers,
                flush_packets,
            ).chain());
    }
}
