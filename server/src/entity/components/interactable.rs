use crate::{Player, WorldExtension};
use bevy_ecs::component::Component;
use bevy_ecs::world::EntityWorldMut;

#[derive(Component)]
pub struct Interactable<W: WorldExtension> {
    pub callback: fn(EntityWorldMut, player: &mut Player<W::Player>)
}

impl<W: WorldExtension> Interactable<W> {
    pub fn new(callback: fn(EntityWorldMut, player: &mut Player<W::Player>)) -> Self {
        Self {
            callback,
        }
    }
}