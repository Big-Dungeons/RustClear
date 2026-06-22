use crate::core::entity::components::transform::Transform;
use crate::core::player::Player;
use crate::dungeon::rooms::{Room, RoomGridLookup};
use bevy::prelude::*;

// maybe also have the room segment entity too in future
#[derive(Component, Deref, Default, PartialEq)]
pub struct CurrentRoom(pub Option<Entity>);

pub fn update_room(
    mut query: Query<(Entity, &Transform, &mut CurrentRoom), (Changed<Transform>, With<Player>)>,
    mut room_query: Query<&mut Room>,
    room_lookup: Res<RoomGridLookup>,
) {
    for (player, transform, mut current_room) in query.iter_mut() {
        let new_room = room_lookup.get_from_world(transform.position);

        if new_room != **current_room {
            if let Some(entity) = **current_room && let Ok(mut old) = room_query.get_mut(entity) {
                old.remove_player(player);
            }
            if let Some(entity) = new_room && let Ok(mut old) = room_query.get_mut(entity) {
                old.insert_player(player);
            }
            *current_room = CurrentRoom(new_room);
        }
    }
}

pub fn on_player_remove(
    event: On<Remove, Player>,
    player_query: Query<&CurrentRoom>,
    mut room_query: Query<&mut Room>,
) {
    let Ok(current_room) = player_query.get(event.entity) else {
        return;
    };

    if let Some(room_entity) = **current_room && let Ok(mut room) = room_query.get_mut(room_entity) {
        room.remove_player(event.entity);
    }
}