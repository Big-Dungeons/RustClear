use crate::dungeon::dungeon::Dungeon;
use bevy_ecs::prelude::Component;
use server::entity::components::EntityBehaviour;
use server::entity::entity::MinecraftEntity;

/// entities with this will only live for the provided amount of time
#[derive(Component)]
pub struct Lifetime {
    pub ticks: u32
}

impl EntityBehaviour<Dungeon> for Lifetime {
    fn tick(entity: &mut MinecraftEntity<Dungeon>, component: &mut Self) {
        if entity.ticks_existed > component.ticks {
            entity.destroy()
        }
    }
}
