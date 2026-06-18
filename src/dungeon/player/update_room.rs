use crate::core::entity::components::transform::Transform;
use crate::core::player::Player;
use crate::dungeon::rooms::{Room, RoomGridLookup};
use bevy::prelude::*;

// maybe also have the room segment entity too in future
#[derive(Component, Deref, Default, PartialEq)]
pub struct CurrentRoom(pub Option<Entity>);

pub fn update_room(
    mut query: Query<(&Transform, &mut CurrentRoom), (Changed<Transform>, With<Player>)>,
    mut room_query: Query<&mut Room>,
    room_lookup: Res<RoomGridLookup>,
) {
    for (transform, mut current_room) in query.iter_mut() {
        let new_room = room_lookup.get_from_world(transform.position);

        if new_room != **current_room {
            if let Some(entity) = **current_room && let Ok(mut old) = room_query.get_mut(entity) {
                old.remove_player(entity);
            }
            if let Some(entity) = new_room && let Ok(mut old) = room_query.get_mut(entity) {
                old.insert_player(entity);
            }
            *current_room = CurrentRoom(new_room);
        }
    }
}