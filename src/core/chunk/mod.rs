pub mod chunk_grid;

use crate::core::block::Block;
use crate::core::entity::{BevyEntityExt, MobSpawnQueries};
use crate::core::network::packets::BytesMutExt;
use crate::core::network::protocol::play::clientbound::{ChunkData, EntityAttach};
use bevy::ecs::entity::EntityHashSet;
use bevy::prelude::*;
use bytes::BytesMut;
use glam::{DVec3, IVec2};

pub struct ChunkSection {
    solid_block_amount: u16,
    data: [u16; 4096]
}

pub struct Chunk {
    pub sections: Box<[Option<ChunkSection>; 16]>,
    pub packet_buffer: BytesMut,

    cached_packet: BytesMut,
    dirty: bool,

    pub players: EntityHashSet,
    pub entities: EntityHashSet,
}

impl Default for Chunk {
    fn default() -> Self {
        Self {
            sections: Box::new([const { None }; 16]),
            packet_buffer: BytesMut::new(),
            cached_packet: BytesMut::new(),
            dirty: true,

            players: EntityHashSet::new(),
            entities: EntityHashSet::new(),
        }
    }
}

impl Chunk {

    pub fn get_block_at(&self, x: i32, y: i32, z: i32) -> Block {
        if let Some(section) = &self.sections[(y / 16) as usize] {
            let index = ((y & 15) << 8) | (z << 4) | x;
            return Block::from(section.data[index as usize])
        }
        Block::Air
    }

    pub fn set_block_at(&mut self, block: Block, local_x: i32, y: i32, local_z: i32) {
        let section_index = (y / 16) as usize;
        if self.sections[section_index].is_none() {
            self.sections[section_index] = Some(ChunkSection {
                solid_block_amount: 0,
                data: [0; 4096],
            })
        }
        if let Some(section) = &mut self.sections[section_index] {
            let block_state_id = block.get_blockstate_id();
            let index = ((y & 15) << 8) | (local_z << 4) | local_x;

            if section.data[index as usize] != 0 {
                section.solid_block_amount -= 1;
            }
            if block != Block::Air {
                section.solid_block_amount += 1;
            }
            section.data[index as usize] = block_state_id;
            self.dirty = true;
        }
    }

    pub fn write_chunk_data(&mut self, x: i32, z: i32, new: bool, into: &mut BytesMut) {
        // this only writes chunks if the x and z are the same,
        // so for an empty chunk this doesn't work
        if self.dirty {

            let mut section_bitmask = 0u16;

            for index in 0..16 {
                if let Some(section) = &self.sections[index] && section.solid_block_amount != 0 {
                    section_bitmask |= 1 << index;
                }
            }

            let section_count = section_bitmask.count_ones() as usize;
            let data_size: usize = section_count * 12288 + if new { 256 } else { 0 };

            let mut data = vec![0u8; data_size];
            let mut offset = 0;

            for section in self.sections.iter().flatten() {
                if section.solid_block_amount == 0 {
                    continue
                }
                for block in section.data.iter() {
                    let block = *block;
                    data[offset] = (block & 0xFF) as u8;
                    data[offset + 1] = ((block >> 8) & 0xFF) as u8;
                    offset += 2;
                }
            };

            // currently all blocks have max skylight and regular light,
            // however ive come across issues, where it seems clients recalculate light (due to it being invalid?)
            // causing massive fps drops

            if section_count != 0 {
                for _ in 0..4096 {
                    data[offset] = 255;
                    offset += 1;
                }
            }
            if new {
                for _ in 0..256 {
                    data[offset] = 1;
                    offset += 1;
                }
            }
            self.cached_packet.clear();
            self.cached_packet.write_packet(&ChunkData {
                chunk_x: x,
                chunk_z: z,
                is_new_chunk: new,
                section_bitmask,
                data,
            });
            self.dirty = false;
        }
        into.extend_from_slice(&self.cached_packet);
    }

    pub fn unload_packet(x: i32, z: i32) -> ChunkData {
        ChunkData {
            chunk_x: x,
            chunk_z: z,
            is_new_chunk: true,
            section_bitmask: 0,
            data: vec![],
        }
    }

    pub fn insert_player(&mut self, entity: Entity) {
        debug_assert!(!self.players.contains(&entity), "player already in chunk");
        self.players.insert(entity);
    }

    pub fn remove_player(&mut self, entity: Entity) {
        debug_assert!(self.players.contains(&entity), "player was never in this chunk");
        self.players.remove(&entity);
    }
    
    pub fn insert_entity(&mut self, entity: Entity) {
        debug_assert!(!self.entities.contains(&entity), "entity already in chunk");
        self.entities.insert(entity);
    }

    pub fn remove_entity(&mut self, entity: Entity) {
        debug_assert!(self.entities.contains(&entity), "entity was never in this chunk");
        self.entities.remove(&entity);
    }

    pub fn write_spawn_entities(&mut self, query: &MobSpawnQueries) {
        for (entity, mob, transform) in query.mob_query.iter_many(&self.entities) {
            mob.write_spawn_packet(entity, transform, &mut self.packet_buffer)
        }
        // order might not be correct due to nature of systems,
        // so 2nd pass it is
        for (entity, riding) in query.riding_query.iter_many(&self.entities) {
            self.packet_buffer.write_packet(&EntityAttach {
                entity_id: entity.mc_id(),
                vehicle_id: riding.mc_id(),
                leash: false,
            })
        }
    }

    pub fn write_despawn_entities(&mut self, query: &MobSpawnQueries) {
        for (entity, mob, _) in query.mob_query.iter_many(&self.entities) {
            mob.write_despawn_packet(entity, &mut self.packet_buffer);
        }
    }
}

pub fn get_chunk_position(position: DVec3) -> IVec2 {
    let x = (position.x.floor() as i32) >> 4;
    let z = (position.z.floor() as i32) >> 4;
    (x, z).into()
}