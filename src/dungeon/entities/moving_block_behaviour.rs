use crate::dungeon::dungeon::Dungeon;
use bevy_ecs::prelude::Component;
use glam::IVec3;
use server::block::Block;
use server::constants::Sound;
use server::entity::components::EntityBehaviour;
use server::entity::entity::MinecraftEntity;

#[derive(Component)]
pub struct MovingBlockBehaviour {
    pub block: IVec3,
    pub total_ticks: u32,
    pub difference: f64,
}

impl EntityBehaviour<Dungeon> for MovingBlockBehaviour {
    fn tick(entity: &mut MinecraftEntity<Dungeon>, component: &mut Self) {
        entity.position.y += component.difference;

        if entity.ticks_existed == component.total_ticks {
            let world = entity.world_mut();

            let IVec3 { x, y, z } = component.block;
            world.chunk_grid.set_block_at(Block::Air, x, y, z);
            entity.destroy()
        }
    }
}

// scuffed
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
        if entity.ticks_existed == 50 { 
            entity.destroy()
        }
    }
}