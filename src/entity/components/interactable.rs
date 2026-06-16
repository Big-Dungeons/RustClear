use crate::player::interact::PlayerInteractEntity;
use bevy::prelude::{Commands, Component, Entity, MessageReader, Query, World};

#[derive(Component)]
pub struct Interactable {
    callback: fn(world: &mut World, player: Entity, entity: Entity)
}

impl Interactable {
    pub fn new(callback: fn(world: &mut World, player: Entity, entity: Entity)) -> Self {
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
            let callback = interactable.callback;
            let player = *client;

            commands.queue(move |world: &mut World| {
                callback(world, player, entity);
            });
        }
    }
}