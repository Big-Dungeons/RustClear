use crate::core::chunk::chunk_grid::ChunkGridBounds;
use crate::core::CorePlugin;
use crate::debug_world::DebugWorld;
use crate::dungeon::DungeonPlugin;
use bevy::app::{App, ScheduleRunnerPlugin};
use bevy::state::app::StatesPlugin;
use glam::IVec2;
use std::time::Duration;

mod core;
mod debug_world;
mod dungeon;

// temp
const TEST_WORLD: bool = true;

fn main() {
    let mut app = App::new();

    app.add_plugins((
        ScheduleRunnerPlugin::run_loop(Duration::from_millis(50)),
        StatesPlugin,
        CorePlugin {
            addr: "127.0.0.1:8080",
            chunk_bounds: ChunkGridBounds {
                size: 16,
                offset: IVec2::splat(13),
            },
        },
        DungeonPlugin {
            seed: 13687989479541743623
            // seed: rand::random(),
        },
    ));

    if TEST_WORLD {
        app.add_plugins(DebugWorld);
    }

    app.run();
}
