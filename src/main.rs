use crate::chunk::ChunkPlugin;
use crate::debug_world::DebugWorld;
use crate::dungeon::DungeonPlugin;
use crate::entity::MobPlugin;
use crate::network::NetworkPlugin;
use crate::player::PlayerPlugin;
use bevy::app::{App, ScheduleRunnerPlugin};
use glam::IVec2;
use std::time::Duration;
use bevy::state::app::StatesPlugin;

mod block;
mod chunk;
mod debug_world;
mod dungeon;
mod entity;
mod network;
mod player;
mod types;

// temp
const TEST_WORLD: bool = false;

fn main() {
    App::new()
        .add_plugins((
            ScheduleRunnerPlugin::run_loop(Duration::from_millis(50)),
            StatesPlugin,
            NetworkPlugin {
                addr: "127.0.0.1:8080",
            },
            ChunkPlugin {
                size: 16,
                offset: IVec2::splat(13),
            },
            MobPlugin,
            PlayerPlugin,
            DungeonPlugin,
            DebugWorld,
        ))
        .run();
}