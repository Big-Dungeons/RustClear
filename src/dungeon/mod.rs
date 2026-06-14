use crate::dungeon::door::DoorLookup;
use crate::dungeon::dungeon_player::{DungeonPlayerPlugin, PlayerReadyEvent};
use crate::dungeon::entities::DungeonEntityPlugin;
use crate::dungeon::menus::DungeonMenuPlugin;
use crate::dungeon::rooms::room_data::RoomDataLookup;
use crate::dungeon::rooms::RoomGridLookup;
use crate::network::packets::{BytesMutExt, PacketEvent};
use crate::network::protocol::play::clientbound::Chat;
use crate::network::protocol::play::serverbound::ChatMessage;
use crate::player::GlobalPacketBuffer;
use crate::types::chat_component::ChatComponent;
use crate::TEST_WORLD;
use bevy::app::{App, PreUpdate, Update};
use bevy::prelude::{Commands, Entity, MessageReader, Plugin, ResMut, Resource};
use glam::IVec2;
use std::ops::DerefMut;

mod door;
mod entities;
mod dungeon_player;
pub mod items;
pub mod rooms;
mod loading;
mod menus;

pub const DUNGEON_ORIGIN: IVec2 = IVec2::new(-200, -200);

#[derive(Resource)]
pub enum DungeonState {
    NotStarted,
    Starting {
        starts_in_ticks: usize
    },
    Started {
        ticks: usize,
    }
}

fn update_dungeon_state(
    mut state: ResMut<DungeonState>,
    mut global_packet_buffer: ResMut<GlobalPacketBuffer>
) {
    match state.deref_mut() {
        DungeonState::Starting { starts_in_ticks: tick } => {
            *tick -= 1;
            if *tick == 0 {
                *state = DungeonState::Started { ticks: 0 }
            } else if *tick % 20 == 0 {
                let seconds_remaining = *tick / 20;
                let s = if seconds_remaining == 1 { "" } else { "s" };
                let str = format!("§aStarting in {} second{}.", seconds_remaining, s);

                // either way sound needs to play locally for all players
                global_packet_buffer.write_packet(&Chat {
                    component: ChatComponent::new(str),
                    chat_type: 0,
                });
            }
        }
        DungeonState::Started { ticks } => {
            *ticks += 1;
        }
        _ => {}
    }
}

#[derive(Resource)]
pub struct EntranceRoom {
    entity: Entity,
}

pub struct DungeonPlugin;

impl Plugin for DungeonPlugin {
    fn build(&self, app: &mut App) {
        if !TEST_WORLD {
            app.add_plugins(
                loading::DungeonLoadingPlugin,
            );
        }

        app
            // temp, while container ui's arent implemented
            .add_systems(Update, |
                mut packets: MessageReader<PacketEvent<ChatMessage>>,
                mut commands: Commands,
            | {
                for PacketEvent { packet, client } in packets.read() {
                    if *packet.string == "/start" {
                        commands.trigger(PlayerReadyEvent { entity: *client })
                    }
                }
            })

            .add_plugins((
                DungeonPlayerPlugin,
                DungeonEntityPlugin,
                DungeonMenuPlugin,
            ))
            .insert_resource(DungeonState::NotStarted)
            .insert_resource(RoomDataLookup::default())
            .insert_resource(RoomGridLookup::default())
            .insert_resource(DoorLookup::default())
            .add_systems(PreUpdate, update_dungeon_state);
    }
}