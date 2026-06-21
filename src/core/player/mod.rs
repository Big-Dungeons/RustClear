pub mod inventory;
pub mod known_state;
pub mod movement;
pub mod interact;
pub mod sidebar;
pub mod sound;

use crate::core::block::block_entity::BlockEntity;
use crate::core::chunk::chunk_grid::ChunkGrid;
use crate::core::chunk::get_chunk_position;
use crate::core::entity::components::transform::{OldTransform, Transform};
use crate::core::entity::MobSpawnQueries;
use crate::core::network::client::ClientId;
use crate::core::network::packets::{BytesMutExt, PacketEvent};
use crate::core::network::protocol::play::clientbound::{ConfirmTransaction, PositionLook};
use crate::core::network::protocol::play::serverbound;
use crate::core::network::{recv_network_messages, NetworkSender};
use crate::core::player::interact::{PlayerInteractEntity, PlayerRightClick};
use crate::core::player::inventory::{InventoryPlugin, SyncInventory};
use crate::core::player::movement::SetPosition;
use crate::core::player::sound::LocalSound;
use bevy::app::{App, First, PostUpdate, PreUpdate};
use bevy::prelude::{Commands, Component, Deref, DerefMut, Entity, IntoScheduleConfigs, Last, Message, MessageReader, Plugin, Query, Res, ResMut, Resource, Update, With, Without};
use bytes::BytesMut;

// marker
#[derive(Component)]
pub struct Player;

// marker
#[derive(Component)]
pub struct Initialized;

#[derive(Component, Deref)]
pub struct Username(pub String);

#[derive(Component, Deref, Copy, Clone)]
pub struct Uuid(pub uuid::Uuid);

#[derive(Component, Clone)]
pub struct PlayerSkin {
    pub texture: String,
    pub _signature: Option<String>
}

#[derive(Component, Deref, DerefMut)]
pub struct PlayerPacketBuffer(pub BytesMut);

#[derive(Resource, Deref, DerefMut)]
pub struct GlobalPacketBuffer(pub BytesMut);

#[derive(Message)]
pub struct PlayerJoinEvent(pub Entity);

type PacketReader<'a, 'b, T> = MessageReader<'a, 'b, PacketEvent<T>>;

fn process_player_join(
    mut events: MessageReader<'_, '_, PlayerJoinEvent>,
    mut player_query: Query<(&Transform, &mut PlayerPacketBuffer)>,
    mob_spawn_queries: MobSpawnQueries,
    block_entity_query: Query<&BlockEntity>,
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
            chunk.write_spawn_entities(&mob_spawn_queries);
            chunk.write_spawn_block_entities(&block_entity_query);
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
    mut player_query: Query<(&Transform, &mut PlayerPacketBuffer)>,
    mut chunks: ResMut<ChunkGrid>,
) {
    for (transform, mut packet_buffer) in player_query.iter_mut() {
        let position = get_chunk_position(transform.position);
        chunks.for_each_in_view(position, 8, |chunk, _, _| {
            packet_buffer.extend_from_slice(&chunk.packet_buffer);
        });
    }
    for chunk in chunks.chunks.iter_mut() {
        chunk.packet_buffer.clear();
    }
}

fn copy_global_packet_buffer(
    mut player_query: Query<&mut PlayerPacketBuffer>,
    mut global_packet_buffer: ResMut<GlobalPacketBuffer>,
) {
    for mut packet_buffer in player_query.iter_mut() {
        packet_buffer.extend_from_slice(&global_packet_buffer)
    }
    global_packet_buffer.clear()
}

fn flush_packets(mut query: Query<(&ClientId, &mut PlayerPacketBuffer)>, tx: Res<NetworkSender>) {
    for (client_id, mut buffer) in query.iter_mut() {
        if buffer.is_empty() {
            continue;
        }
        let _ = tx.send(buffer.get_packet_message(*client_id));
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(GlobalPacketBuffer(BytesMut::new()))
            .add_message::<SetPosition>()
            .add_message::<PlayerRightClick>()
            .add_message::<PlayerInteractEntity>()
            .add_message::<LocalSound>()
            .add_plugins(InventoryPlugin)
            .add_observer(sidebar::init_sidebar_packets)

            // give that systems in bevy do not have an order based on insertion (unless using .chain() etc)
            // systems should be put in the schedule based on:

            // First: shouldn't really be used, handles player connecting and receiving packets
            // PreUpdate: use primarily for reading/processing packets
            // Update: Main ticking schedule for ticking game logic etc
            // PostUpdate: Ticking after main ticking happens, usually for flushing packets
            // Last: Cleanup and copying packets from chunk buffers, global and then sending them

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
                ),
            )
            .add_systems(Update, movement::transform_change)
            .add_systems(
                PostUpdate,
                (
                    movement::handle_set_position,
                    interact::clear_sent_interacts,
                    sidebar::flush_sidebar_packets,
                    sound::handle_local_sounds,
                ),
            )
            .add_systems(Last, (
                copy_chunk_packet_buffers,
                copy_global_packet_buffer,
                flush_packets,
            ).chain());
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
