use crate::core::network::packets::BytesMutExt;
use crate::core::network::protocol::play::clientbound::Chat;
use crate::core::player::GlobalPacketBuffer;
use crate::core::types::chat_component::ChatComponent;
use crate::dungeon::door::DoorLookup;
use crate::dungeon::entities::DungeonEntityPlugin;
use crate::dungeon::menus::DungeonMenuPlugin;
use crate::dungeon::player::DungeonPlayerPlugin;
use crate::dungeon::rooms::room_data::RoomDataLookup;
use crate::dungeon::rooms::secrets::DungeonSecretsPlugin;
use crate::dungeon::rooms::RoomGridLookup;
use crate::TEST_WORLD;
use bevy::app::{App, PreUpdate};
use bevy::prelude::{AppExtStates, Entity, NextState, OnEnter, Plugin, ResMut, Resource, State, States, Update};
use door::door_opening;
use glam::IVec2;

mod player;
mod entities;
pub mod items;
mod menus;
pub mod rooms;
mod door;
mod loading;

pub const DUNGEON_ORIGIN: IVec2 = IVec2::new(-200, -200);

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, States)]
pub enum DungeonState {
    #[default]
    NotStarted,
    Starting {
        starts_in_ticks: usize
    },
    Started {
        ticks: usize,
    }
}

fn update_dungeon_state(
    state: ResMut<State<DungeonState>>,
    mut next_state: ResMut<NextState<DungeonState>>,
    mut global_packet_buffer: ResMut<GlobalPacketBuffer>,
) {
    match state.get() {
        DungeonState::Starting { starts_in_ticks: tick } => {
            let tick = tick - 1;
            next_state.set(DungeonState::Starting { starts_in_ticks: tick });
            if tick == 0 {
                next_state.set(DungeonState::Started { ticks: 0 });
            } else if tick % 20 == 0 {
                let seconds_remaining = tick / 20;
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
            next_state.set(DungeonState::Started { ticks: ticks + 1 });
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
            .init_state::<DungeonState>()
            .add_plugins((
                DungeonPlayerPlugin,
                DungeonEntityPlugin,
                DungeonMenuPlugin,
                DungeonSecretsPlugin,
            ))
            .add_observer(door_opening::open_door)
            .add_systems(OnEnter(DungeonState::Started { ticks: 0 }), door::open_entrance_doors)
            .add_systems(PreUpdate, update_dungeon_state)
            .add_systems(Update, door_opening::handle_opening_door);
    }
}