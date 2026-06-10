use crate::dungeon::door::DoorLookup;
use crate::dungeon::dungeon_player::DungeonPlayerPlugin;
use crate::dungeon::rooms::room_data::RoomDataLookup;
use crate::dungeon::rooms::RoomGridLookup;
use crate::TEST_WORLD;
use bevy::app::App;
use bevy::prelude::{Entity, Plugin, Resource, };
use glam::IVec2;

mod door;
mod dungeon_player;
pub mod items;
pub mod rooms;
mod loading;

pub struct DungeonPlugin;

impl Plugin for DungeonPlugin {
    fn build(&self, app: &mut App) {
        if !TEST_WORLD {
            app.add_plugins(
                loading::DungeonLoadingPlugin,
            );
        }

        app
            .add_plugins(
                (
                    DungeonPlayerPlugin,
                )
            )
            .insert_resource(RoomDataLookup::default())
            .insert_resource(RoomGridLookup::default())
            .insert_resource(DoorLookup::default());

    }
}

pub const DUNGEON_ORIGIN: IVec2 = IVec2::new(-200, -200);

#[derive(Resource)]
pub struct EntranceRoom {
    entity: Entity,
    // idk if this is needed
    _segment_entity: Entity,
}
