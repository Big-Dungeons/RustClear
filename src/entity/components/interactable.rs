use crate::player::interact::PlayerInteractEntity;
use bevy::prelude::{Commands, Component, Entity, MessageReader, Query};

#[derive(Component)]
pub struct Interactable {
    callback: fn(Entity, player: Entity, commands: &mut Commands)
}

impl Interactable {
    pub fn new(callback: fn(Entity, player: Entity, commands: &mut Commands)) -> Self {
        Self {
            callback
        }
    }
}

pub fn handle_entity_interactable(
    mut events: MessageReader<PlayerInteractEntity>,
    query: Query<(Entity, &Interactable)>,
    mut commands: Commands
) {
    for PlayerInteractEntity { client, entity } in events.read() {
        if let Ok((entity, interactable)) = query.get(*entity) {
            (interactable.callback)(entity, *client, &mut commands)
        }
    }
}