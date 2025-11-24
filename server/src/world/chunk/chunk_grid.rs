use crate::block::blocks::Block;
use crate::network::protocol::play::clientbound::{BlockChange, ChunkData};
use crate::world::chunk::chunk::Chunk;
use glam::{ivec3, IVec3};
use std::cmp::{max, min};

/// Chunk grid
///
/// Stores a square grid of chunks, based on the provided size.
pub struct ChunkGrid {
    pub chunks: Vec<Chunk>,
    pub size: usize,

    index_offset_x: usize,
    index_offset_z: usize,
}

#[derive(PartialEq)]
pub enum ChunkDiff {
    New,
    Old,
}

impl ChunkGrid {

    pub fn new(size: usize, offset_x: usize, offset_z: usize) -> ChunkGrid {
        let mut chunks = Vec::with_capacity(size * size);
        for _ in 0..size * size {
            chunks.push(Chunk::new());
        }
        ChunkGrid {
            chunks,
            size,
            index_offset_x: offset_x,
            index_offset_z: offset_z,
        }
    }

    pub fn get_block_at(&self, x: i32, y: i32, z: i32) -> Block {
        if !self.is_block_valid(x, y, z) {
            return Block::Air;
        }
        let chunk_x = x >> 4;
        let chunk_z = z >> 4;

        if let Some(chunk) = self.get_chunk(chunk_x, chunk_z) {
            let local_x = x & 15;
            let local_z = z & 15;
            return chunk.get_block_at(local_x, y, local_z)
        }
        Block::Air
    }

    pub fn set_block_at(&mut self, block: Block, x: i32, y: i32, z: i32) {
        if !self.is_block_valid(x, y, z) {
            return;
        }
        let chunk_x = x >> 4;
        let chunk_z = z >> 4;

        if let Some(chunk) = self.get_chunk_mut(chunk_x, chunk_z) {
            let local_x = x & 15;
            let local_z = z & 15;
            chunk.set_block_at(block, local_x, y, local_z);
            chunk.packet_buffer.write_packet(&BlockChange {
                block_pos: ivec3(x, y, z),
                block_state: block.get_blockstate_id(),
            })
        }
    }

    /// checks is block is a valid block within the chunk grid.
    fn is_block_valid(&self, x: i32, y: i32, z: i32) -> bool {
        let size = self.size as i32;
        let chunk_x = (x >> 4) + self.index_offset_x as i32;
        let chunk_z = (z >> 4) + self.index_offset_z as i32;
        y >= 0 && y < 256 && chunk_x >= 0 && chunk_x < size && chunk_z >= 0 && chunk_z < size
    }

    /// returns the chunk at the x and z coordinates provided, none if no chunk is present
    pub fn get_chunk(&self, chunk_x: i32, chunk_z: i32) -> Option<&Chunk> {
        let size = self.size as i32;
        let x = chunk_x + self.index_offset_x as i32;
        let z = chunk_z + self.index_offset_z as i32;
        if x < 0 || z < 0 || x >= size || z >= size {
            return None
        }
        self.chunks.get(z as usize * self.size + x as usize)
    }

    pub fn get_chunk_mut(&mut self, chunk_x: i32, chunk_z: i32) -> Option<&mut Chunk> {
        let size = self.size as i32;
        let x = chunk_x + self.index_offset_x as i32;
        let z = chunk_z + self.index_offset_z as i32;
        if x < 0 || z < 0 || x >= size || z >= size {
            return None
        }
        self.chunks.get_mut(z as usize * self.size + x as usize)
    }

    pub fn for_each_in_view<F>(
        &mut self,
        chunk_x: i32,
        chunk_z: i32,
        view_distance: i32,
        mut callback: F,
    ) where
        F: FnMut(&mut Chunk, i32, i32)
    {
        let min_x = max(chunk_x - view_distance + self.index_offset_x as i32, 0);
        let min_z = max(chunk_z - view_distance + self.index_offset_z as i32, 0);
        let max_x = min(chunk_x + view_distance + self.index_offset_x as i32, self.size as i32);
        let max_z = min(chunk_z + view_distance + self.index_offset_z as i32, self.size as i32);

        for x in min_x..max_x {
            for z in min_z..max_z {
                if let Some(chunk) = self.chunks.get_mut(z as usize * self.size + x as usize) {
                    callback(chunk, x - self.index_offset_x as i32, z - self.index_offset_z as i32)
                }
            }
        }
    }

    pub fn for_each_diff<F>(
        &self,
        new: (i32, i32),
        old: (i32, i32),
        view_distance: i32,
        mut callback: F,
    ) where
        F: FnMut(i32, i32, ChunkDiff),
    {
        let (nx, nz) = (new.0 + self.index_offset_x as i32, new.1 + self.index_offset_z as i32);
        let min_x = max(nx - view_distance, 0);
        let min_z = max(nz - view_distance, 0);
        let max_x = min(nx + view_distance, self.size as i32);
        let max_z = min(nz + view_distance, self.size as i32);

        let (ox, oz) = (old.0 + self.index_offset_x as i32, old.1 + self.index_offset_z as i32);
        let old_min_x = max(ox - view_distance, 0);
        let old_min_z = max(oz - view_distance, 0);
        let old_max_x = min(ox + view_distance, self.size as i32);
        let old_max_z = min(oz + view_distance, self.size as i32);

        // it would be more optimal to loop over chunks that are only different
        for x in min_x..=max_x {
            for z in min_z..=max_z {
                let in_old_range =
                    x >= old_min_x && x <= old_max_x &&
                        z >= old_min_z && z <= old_max_z;

                if !in_old_range {
                    callback(x - self.index_offset_x as i32, z - self.index_offset_z as i32, ChunkDiff::New);
                }
            }
        }

        for x in old_min_x..=old_max_x {
            for z in old_min_z..=old_max_z {
                let in_new_range =
                    x >= min_x && x <= max_x &&
                        z >= min_z && z <= max_z;

                if !in_new_range {
                    callback(x - self.index_offset_x as i32, z - self.index_offset_z as i32, ChunkDiff::Old);
                }
            }
        }
    }

    pub fn fill_blocks(&mut self, block: Block, start: IVec3, end: IVec3) {
        iterate_blocks(start, end, |x, y, z| {
            self.set_block_at(block, x, y, z)
        })
    }

    pub fn get_unload_chunk_packet(chunk_x: i32, chunk_z: i32) -> ChunkData {
        ChunkData {
            chunk_x,
            chunk_z,
            is_new_chunk: true,
            bitmask: 0,
            data: vec![],
        }
    }
}

/// iterates over the blocks in area between start and end
/// and runs a function
#[inline(always)]
pub fn iterate_blocks<F>(
    start: IVec3,
    end: IVec3,
    mut callback: F,
) where
    F : FnMut(i32, i32, i32)
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
                callback(x, y, z);
            }
        }
    }
}