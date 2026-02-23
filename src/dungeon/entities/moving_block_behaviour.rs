use crate::dungeon::dungeon::Dungeon;
use bevy_ecs::prelude::Component;
use glam::IVec3;
use server::block::Block;
use server::entity::components::EntityBehaviour;
use server::entity::entity::MinecraftEntity;

#[derive(Component)]
pub struct MovingBlockBehaviour {
    pub block: IVec3,
    pub remove_block_in_tick: u32,
    pub difference: f64,
}

impl EntityBehaviour<Dungeon> for MovingBlockBehaviour {
    fn tick(entity: &mut MinecraftEntity<Dungeon>, component: &mut Self) {
        entity.position.y += component.difference;

        if entity.ticks_existed == component.remove_block_in_tick {
            let world = entity.world_mut();

            let IVec3 { x, y, z } = component.block;
            world.chunk_grid.set_block_at(Block::Air, x, y, z);
        }
    }
}