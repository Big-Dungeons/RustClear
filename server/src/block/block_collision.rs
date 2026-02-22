use crate::block::Block;
use crate::types::aabb::AABB;
use crate::{World, WorldExtension};
use glam::dvec3;

pub fn check_block_collisions<W : WorldExtension + 'static>(world: &World<W>, aabb: &AABB) -> bool {
    let min_x = aabb.min.x.floor() as i32;
    let min_y = aabb.min.y.floor() as i32;
    let min_z = aabb.min.z.floor() as i32;
    let max_x = aabb.max.x.ceil() as i32;
    let max_y = aabb.max.y.ceil() as i32;
    let max_z = aabb.max.z.ceil() as i32;

    for x in min_x..max_x {
        for y in min_y..max_y {
            for z in min_z..max_z {
                let block = world.chunk_grid.get_block_at(x, y, z);
                let (shapes, len) = block_collision(block);
                for shape in &shapes[..len] {
                    let shifted = shape.offset(dvec3(x as f64, y as f64, z as f64));
                    if aabb.intersects(&shifted) {
                        return true;
                    }
                }
            }
        }
    }

    false
}

pub fn block_collision(block: Block) -> ([AABB; 1], usize) {
    let length = if block == Block::Air { 0 } else { 1 };
    ([AABB::new(dvec3(0.0, 0.0, 0.0), dvec3(1.0, 1.0, 1.0))], length)
}