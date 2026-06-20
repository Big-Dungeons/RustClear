use crate::TEST_WORLD;
use crate::core::ServerTick;
use crate::core::network::packets::BytesMutExt;
use crate::core::network::protocol::play::clientbound::Chat;
use crate::core::player::GlobalPacketBuffer;
use crate::core::types::chat_component::ChatComponent;
use crate::dungeon::entities::DungeonEntityPlugin;
use crate::dungeon::menus::DungeonMenuPlugin;
use crate::dungeon::player::DungeonPlayerPlugin;
use crate::dungeon::rng::SeededRng;
use crate::dungeon::rooms::secrets::DungeonSecretsPlugin;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use door::door_opening;
use glam::IVec2;

mod door;
mod entities;
pub mod items;
mod loading;
mod menus;
mod player;
pub mod rng;
pub mod rooms;

pub struct DungeonPlugin {
    pub seed: u64,
}

impl Plugin for DungeonPlugin {
    fn build(&self, app: &mut App) {
        println!("Loading dungeon with seed: {}", self.seed);
        app.insert_resource(SeededRng::from_seed(self.seed));

        if !TEST_WORLD {
            app.add_plugins(loading::DungeonLoadingPlugin);
        }

        app.insert_resource(DungeonTick(0))
            .init_state::<DungeonState>()
            .add_plugins((
                DungeonPlayerPlugin,
                DungeonEntityPlugin,
                DungeonMenuPlugin,
                DungeonSecretsPlugin,
            ))
            .add_observer(door_opening::open_door)
            .add_systems(OnEnter(DungeonState::Started), door::open_entrance_doors)
            .add_systems(PostUpdate, update_dungeon_state)
            .add_systems(
                Update,
                (
                    door_opening::handle_opening_door,
                    rooms::room_enter::update_room_entered.run_if(in_state(DungeonState::Started)),
                ),
            );
    }
}

pub const DUNGEON_ORIGIN: IVec2 = IVec2::new(-200, -200);

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash, States)]
pub enum DungeonState {
    #[default]
    NotStarted,
    Starting,
    Started,
}

const STARTING_TIME: i64 = 100;

type CurrentDungeonState<'w> = Res<'w, State<DungeonState>>;
type NextDungeonState<'w> = ResMut<'w, NextState<DungeonState>>;

fn update_dungeon_state(
    state: CurrentDungeonState,
    mut next_state: NextDungeonState,
    mut global_packet_buffer: ResMut<GlobalPacketBuffer>,
    mut dungeon_timer: DungeonTimer,
) {
    if let DungeonState::Starting = state.get() {
        let elapsed = dungeon_timer.elapsed();

        if elapsed == 100 {
            next_state.set(DungeonState::Started);
            dungeon_timer.update_to_now();
        } else if elapsed % 20 == 0 {
            let seconds_remaining = (STARTING_TIME - elapsed) / 20;
            let s = if seconds_remaining == 1 { "" } else { "s" };
            let str = format!("§aStarting in {} second{}.", seconds_remaining, s);

            // either way sound needs to play locally for all players
            global_packet_buffer.write_packet(&Chat {
                component: ChatComponent::new(str),
                chat_type: 0,
            });
        }
    }
}

// set when changing from starting and started etc
#[derive(Resource, Deref)]
pub struct DungeonTick(pub i64);

#[derive(SystemParam)]
pub struct DungeonTimer<'w> {
    pub dungeon_tick: ResMut<'w, DungeonTick>,
    pub server: Res<'w, ServerTick>,
}

impl<'w> DungeonTimer<'w> {
    pub fn update_to_now(&mut self) {
        *self.dungeon_tick = DungeonTick(self.server.now())
    }
    pub fn elapsed(&self) -> i64 {
        self.server.elapsed_since(**self.dungeon_tick)
    }
}

#[derive(Resource)]
pub struct EntranceRoom {
    entity: Entity,
}