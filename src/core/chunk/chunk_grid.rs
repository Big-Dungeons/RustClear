use crate::core::block::Block;
use crate::core::chunk::{get_chunk_position, Chunk};
use crate::core::network::packets::BytesMutExt;
use crate::core::network::protocol::play::clientbound::BlockChange;
use bevy::prelude::Resource;
use glam::{ivec3, DVec3, IVec2, IVec3, Vec3Swizzles};
use std::cmp::{max, min};

#[derive(PartialEq)]
pub enum ChunkDiff {
    New,
    Old,
}

#[derive(Copy, Clone)]
pub struct ChunkGridBounds {
    pub size: i32,
    pub offset: IVec2,
}

#[derive(Resource)]
pub struct ChunkGrid {
    pub chunks: Vec<Chunk>,
    pub bounds: ChunkGridBounds,
}

impl ChunkGrid {
    pub fn new(bounds: ChunkGridBounds) -> Self {
        let size = bounds.size;
        let mut chunks = Vec::with_capacity((size * size) as usize);
        for _ in 0..size * size {
            chunks.push(Chunk::default());
        }
        Self {
            chunks,
            bounds,
        }
    }

    pub fn get_block_at(&self, position: impl Into<IVec3>) -> Block {
        let IVec3 { x, y, z } = position.into();

        if !self.is_block_valid(x, y, z) {
            return Block::Air;
        }
        if let Some(chunk) = self.get((x >> 4, z >> 4)) {
            let local_x = x & 15;
            let local_z = z & 15;
            return chunk.get_block_at(local_x, y, local_z);
        }
        Block::Air
    }

    pub fn set_block_at(&mut self, block: Block, position: impl Into<IVec3>) {
        let IVec3 { x, y, z } = position.into();
        
        if !self.is_block_valid(x, y, z) {
            return;
        }

        if let Some(chunk) = self.get_mut((x >> 4, z >> 4)) {
            let local_x = x & 15;
            let local_z = z & 15;
            chunk.set_block_at(block, local_x, y, local_z);
            chunk.packet_buffer.write_packet(&BlockChange::new(ivec3(x, y, z), block))
        }
    }

    /// returns the chunk at the x and z coordinates provided, none if no chunk is present
    pub fn get(&self, chunk_position: impl Into<IVec2>) -> Option<&Chunk> {
        let ChunkGridBounds { size, offset } = self.bounds;
        let IVec2 { x, y } = chunk_position.into() + offset;
        if x < 0 || y < 0 || x >= size || y >= size {
            return None;
        }
        self.chunks.get(y as usize * size as usize + x as usize)
    }

    pub fn get_mut(&mut self, chunk_position: impl Into<IVec2>) -> Option<&mut Chunk> {
        let ChunkGridBounds { size, offset } = self.bounds;
        let IVec2 { x, y } = chunk_position.into() + offset;
        if x < 0 || y < 0 || x >= size || y >= size {
            return None;
        }
        self.chunks.get_mut(y as usize * size as usize + x as usize)
    }
    
    pub fn get_from_world(&self, position: IVec3) -> Option<&Chunk> {
        self.get(position.xz() >> 4)
    }

    pub fn get_mut_from_world(&mut self, position: IVec3) -> Option<&mut Chunk> {
        self.get_mut(position.xz() >> 4)
    }

    pub fn get_from_position(&self, position: DVec3) -> Option<&Chunk> {
        self.get(get_chunk_position(position))
    }

    pub fn get_mut_from_position(&mut self, position: DVec3) -> Option<&mut Chunk> {
        self.get_mut(get_chunk_position(position))
    }

    pub fn for_each_in_view<F>(&mut self, position: IVec2, view_distance: i32, mut callback: F)
    where
        F: FnMut(&mut Chunk, i32, i32),
    {
        let (chunk_x, chunk_z) = position.into();
        let ChunkGridBounds { size, offset } = self.bounds;

        let min_x = max(chunk_x - view_distance + offset.x, 0);
        let min_z = max(chunk_z - view_distance + offset.y, 0);
        let max_x = min(chunk_x + view_distance + offset.x, size);
        let max_z = min(chunk_z + view_distance + offset.y, size);

        for x in min_x..max_x {
            for z in min_z..max_z {
                if let Some(chunk) = self.chunks.get_mut(z as usize * size as usize + x as usize) {
                    callback(chunk, x - offset.x, z - offset.y)
                }
            }
        }
    }

    pub fn for_each_diff<F>(
        bounds: ChunkGridBounds,
        new: IVec2,
        old: IVec2,
        view_distance: i32,
        mut callback: F,
    ) where
        F: FnMut(i32, i32, ChunkDiff),
    {
        let ChunkGridBounds { size, offset } = bounds;

        let (nx, nz) = (new.x + offset.x, new.y + offset.y);
        let min_x = max(nx - view_distance, 0);
        let min_z = max(nz - view_distance, 0);
        let max_x = min(nx + view_distance, size);
        let max_z = min(nz + view_distance, size);

        let (ox, oz) = (old.x + offset.x, old.y + offset.y);
        let old_min_x = max(ox - view_distance, 0);
        let old_min_z = max(oz - view_distance, 0);
        let old_max_x = min(ox + view_distance, size);
        let old_max_z = min(oz + view_distance, size);

        // it would be more optimal to loop over chunks that are only different
        for x in min_x..=max_x {
            for z in min_z..=max_z {
                let in_old_range =
                    x >= old_min_x && x <= old_max_x && z >= old_min_z && z <= old_max_z;

                if !in_old_range {
                    callback(x - offset.x, z - offset.y, ChunkDiff::New);
                }
            }
        }
        for x in old_min_x..=old_max_x {
            for z in old_min_z..=old_max_z {
                let in_new_range = x >= min_x && x <= max_x && z >= min_z && z <= max_z;

                if !in_new_range {
                    callback(x - offset.x, z - offset.x, ChunkDiff::Old);
                }
            }
        }
    }

    fn is_block_valid(&self, x: i32, y: i32, z: i32) -> bool {
        let ChunkGridBounds { size, offset } = self.bounds;
        let chunk_x = (x >> 4) + offset.x;
        let chunk_z = (z >> 4) + offset.y;
        (0..256).contains(&y) && chunk_x >= 0 && chunk_x < size && chunk_z >= 0 && chunk_z < size
    }

    pub fn fill_blocks(&mut self, block: Block, start: IVec3, end: IVec3) {
        iterate_blocks(start, end, |position| {
            self.set_block_at(block, position)
        })
    }
}

#[inline(always)]
pub fn iterate_blocks<F>(
    start: IVec3,
    end: IVec3,
    mut callback: F,
) where
    F : FnMut(IVec3)
{
    let x0 = start.x.min(end.x);
    let y0 = start.y.min(end.y);
    let z0 = start.z.min(end.z);

    let x1 = start.x.max(end.x);
    let y1 = start.y.max(end.y);
    let z1 = start.z.max(end.z);

    for x in x0..=x1 {
        for z in z0..=z1 {
            for y in y0..=y1 {
                callback((x, y, z).into());
            }
        }
    }
}