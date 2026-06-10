pub mod movement;
pub mod inventory;
pub mod use_item;
pub mod known_state;

use crate::chunk::chunk_grid::ChunkGrid;
use crate::chunk::get_chunk_position;
use crate::entity::Transform;
use crate::network::client::ClientId;
use crate::network::packets::{BytesMutExt, PacketEvent};
use crate::network::protocol::play::clientbound::{ConfirmTransaction, PositionLook};
use crate::network::NetworkSender;
use crate::player::inventory::{InventoryPlugin, SyncInventory};
use crate::player::movement::SetPosition;
use crate::player::use_item::PlayerRightClick;
use bevy::app::{App, First, PostUpdate, PreUpdate};
use bevy::prelude::{Commands, Component, Deref, DerefMut, Entity, Last, Message, MessageReader, Plugin, Query, Res, ResMut, Update};
use bytes::BytesMut;

// marker
#[derive(Component)]
pub struct Player;
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
    mut query: Query<(&Transform, &mut PlayerPacketBuffer)>,
    mut chunks: ResMut<ChunkGrid>,
    mut commands: Commands,
) {
    for PlayerJoinEvent(entity) in events.read() {
        let (transform, mut packet_buffer) = query.get_mut(*entity).unwrap();
        // already wrote join game

        // should maybe inline it. however, might as well reuse it
        commands.trigger(SyncInventory { entity: *entity });

        packet_buffer.write_packet(&PositionLook {
            x: transform.position.x,
            y: transform.position.y,
            z: transform.position.z,
            yaw: transform.yaw,
            pitch: transform.pitch,
            flags: Default::default(),
        });

        let position = get_chunk_position(transform.position);
        chunks.for_each_in_view(position, 8, |chunk, x, z| {
            chunk.write_chunk_data(x, z, true, &mut packet_buffer);
        });
    }
}

fn flush_packets(
    mut query: Query<(&ClientId, &mut PlayerPacketBuffer)>,
    tx: Res<NetworkSender>,
) {
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

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_message::<SetPosition>()
            .add_message::<PlayerRightClick>()
            .add_plugins(InventoryPlugin)
            .add_systems(First, process_player_join)
            .add_systems(PreUpdate, (
                keep_alive,
                movement::handle_incoming_movement_packets,
                use_item::handle_block_interact,
                known_state::handle_player_action,
                inventory::handle_click_window
            ))
            .add_systems(Update, movement::transform_change)
            .add_systems(PostUpdate, (
                movement::handle_set_position,
                use_item::clear_sent_interacts,
            ))
            .add_systems(Last, flush_packets)
        ;
    }
}