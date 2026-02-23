use crate::dungeon::dungeon::Dungeon;
use bevy_ecs::prelude::Component;
use server::constants::Sound;
use server::entity::components::EntityBehaviour;
use server::entity::entity::MinecraftEntity;

#[derive(Component)]
pub struct DoorSoundEmitter {
    pub sound: Sound,
    pub volume: f32,
    pub pitch: f32,
}

impl EntityBehaviour<Dungeon> for DoorSoundEmitter {
    fn tick(entity: &mut MinecraftEntity<Dungeon>, component: &mut Self) {
        if entity.ticks_existed.is_multiple_of(5) {
            entity.world_mut().play_sound_at(
                component.sound,
                component.volume,
                component.pitch,
                entity.position,
            );
        }
    }
}