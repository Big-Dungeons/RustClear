pub mod readying;
pub mod sidebar;
pub mod update_room;
pub mod block_interaction;

use crate::core::block::block_rotation::Rotate;
use crate::core::entity::components::transform::Transform;
use crate::core::player::inventory::Inventory;
use crate::core::player::sidebar::Sidebar;
use crate::core::player::{interact, Player};
use crate::dungeon::items::etherwarp::{use_aspect_of_the_void, AspectOfTheVoid};
use crate::dungeon::items::pickaxe::Pickaxe;
use crate::dungeon::items::skyblock_menu::SkyblockMenu;
use crate::dungeon::items::DungeonItem;
use crate::dungeon::player::readying::ReadyStatus;
use crate::dungeon::player::update_room::CurrentRoom;
use crate::dungeon::rooms::Room;
use crate::dungeon::EntranceRoom;
use bevy::prelude::*;
use glam::ivec3;

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
        .insert((
            Sidebar::new("SBScoreboard"),
            CurrentRoom::default(),
            ReadyStatus::default(),
        ));
}

fn add_items(event: On<Add, Player>, mut query: Query<&mut Inventory>) {
    let mut inventory = query.get_mut(event.entity).unwrap();
    inventory.set_slot(37, Some(DungeonItem::AspectOfTheVoid(AspectOfTheVoid)));
    inventory.set_slot(39, Some(DungeonItem::Pickaxe(Pickaxe)));
    inventory.set_slot(44, Some(DungeonItem::SkyblockMenu(SkyblockMenu)));
}


pub struct DungeonPlayerPlugin;

impl Plugin for DungeonPlayerPlugin {
    fn build(&self, app: &mut App) {
        // no crash trying to access entrance despite it not existing
        if !crate::TEST_WORLD {
            app.add_observer(init_player);
            app.add_observer(update_room::on_player_remove);
            app.add_systems(Update, update_room::update_room);
        }

        app
            .insert_resource(block_interaction::BlockInteractableLookup::default())
            .add_observer(block_interaction::on_add_block_interactable)
            .add_observer(block_interaction::on_remove_block_interactable)
            .add_observer(add_items)
            .add_observer(readying::on_player_ready)
            .add_systems(PreUpdate, (
                use_aspect_of_the_void.after(interact::handle_block_interact),
                block_interaction::on_player_interact.after(interact::handle_block_interact)
            ))
            .add_systems(Update, (
                sidebar::update_sidebar,
            ));
    }
}