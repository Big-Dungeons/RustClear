use crate::dungeon::rooms::Room;
use bevy::prelude::*;

// inserted when player enters room for the first time
// is set in player::update_room
#[derive(Component)]
pub struct RoomEntered;

pub fn update_room_entered(
    query: Query<(Entity, &Room), Without<RoomEntered>>,
    mut commands: Commands,
) {
    for (entity, room) in query.iter() {
        if !room.players.is_empty() {
            commands
                .entity(entity)
                .insert(RoomEntered);
        }
    }
}