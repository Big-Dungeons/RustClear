use crate::block::block_rotation::Rotate;
use crate::dungeon::items::etherwarp::{use_aspect_of_the_void, AspectOfTheVoid};
use crate::dungeon::items::pickaxe::Pickaxe;
use crate::dungeon::items::skyblock_menu::SkyblockMenu;
use crate::dungeon::items::DungeonItem;
use crate::dungeon::rooms::Room;
use crate::dungeon::EntranceRoom;
use crate::entity::components::transform::Transform;
use crate::player::inventory::Inventory;
use crate::player::sidebar::Sidebar;
use crate::player::Player;
use crate::TEST_WORLD;
use bevy::app::{App, Update};
use bevy::prelude::{Add, Commands, On, Plugin, Query, Res};
use glam::{ivec3, DVec3};

pub struct DungeonPlayerPlugin;

impl Plugin for DungeonPlayerPlugin {
    fn build(&self, app: &mut App) {
        // no crash trying to access entrance despite it not existing
        if !TEST_WORLD {
            app.add_observer(init_player);
        }
        app
            .add_observer(add_items)
            .add_systems(Update, (
                update_sidebar,
                use_aspect_of_the_void
            ));
    }
}

fn init_player(
    event: On<Add, Player>,
    mut player_query: Query<&mut Transform>,
    room_query: Query<&Room>,
    entrance_room: Res<EntranceRoom>,
    mut commands: Commands
) {
    let mut transform = player_query.get_mut(event.entity).unwrap();

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

    commands
        .entity(event.entity)
        .insert(Sidebar::new("SBScoreboard"));
}

fn add_items(event: On<Add, Player>, mut query: Query<&mut Inventory>) {
    let mut inventory = query.get_mut(event.entity).unwrap();
    inventory.set_slot(37, Some(DungeonItem::from(AspectOfTheVoid)));
    inventory.set_slot(39, Some(DungeonItem::from(Pickaxe)));
    inventory.set_slot(44, Some(DungeonItem::from(SkyblockMenu)));
}

fn update_sidebar(mut query: Query<(&mut Sidebar, &Transform)>) {
    for (mut sidebar, transform) in query.iter_mut() {
        // temp
        sidebar.push("skyblock");
        let DVec3 { x, y, z } = transform.position;
        sidebar.push(&format!("position\n x {x}\n y {y}\n z {z}"));
    }
}