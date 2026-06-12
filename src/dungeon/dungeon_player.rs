use crate::block::block_rotation::Rotate;
use crate::dungeon::items::etherwarp::{use_aspect_of_the_void, AspectOfTheVoid};
use crate::dungeon::items::pickaxe::Pickaxe;
use crate::dungeon::items::skyblock_menu::SkyblockMenu;
use crate::dungeon::items::DungeonItem;
use crate::dungeon::rooms::Room;
use crate::dungeon::EntranceRoom;
use crate::entity::components::transform::Transform;
use crate::player::inventory::Inventory;
use crate::player::Player;
use crate::TEST_WORLD;
use bevy::app::{App, Update};
use bevy::prelude::{Add, On, Plugin, Query, Res};
use glam::ivec3;

pub struct DungeonPlayerPlugin;

impl Plugin for DungeonPlayerPlugin {
    fn build(&self, app: &mut App) {
        // no crash trying to access entrance despite it not existing
        if !TEST_WORLD {
            app.add_observer(spawn_at_entrance);
        }
        app
            .add_observer(add_items)
            .add_systems(Update, use_aspect_of_the_void);
    }
}

fn spawn_at_entrance(
    event: On<Add, Player>,
    mut query: Query<&mut Transform>,
    entrance_room: Res<EntranceRoom>,
    room_query: Query<&Room>,
) {
    let mut transform = query.get_mut(event.entity).unwrap();
    let room = room_query.get(entrance_room.entity).unwrap();
    let mut position = room.relative_to_world(ivec3(15, 72, 18)).as_dvec3();
    position.x += 0.5;
    position.z += 0.5;
    let yaw = 180.0.rotate(room.rotation);

    *transform = Transform {
        position,
        yaw,
        pitch: 0.0,
    };
}

fn add_items(event: On<Add, Player>, mut query: Query<&mut Inventory>) {
    let mut inventory = query.get_mut(event.entity).unwrap();
    inventory.set_slot(37, Some(DungeonItem::from(AspectOfTheVoid)));
    inventory.set_slot(39, Some(DungeonItem::from(Pickaxe)));
    inventory.set_slot(44, Some(DungeonItem::from(SkyblockMenu)));
}
