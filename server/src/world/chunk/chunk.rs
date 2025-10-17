use crate::block::blocks::Blocks;
use crate::entity::entity::EntityId;
use crate::network::packets::packet_buffer::PacketBuffer;
use crate::network::protocol::play::clientbound::ChunkData;
use crate::player::player::ClientId;
use glam::DVec3;
use std::collections::HashSet;

pub struct ChunkSection {
    solid_block_amount: u16,
    data: Box<[u16; 4096]>,
}
pub struct Chunk {
    pub chunk_sections: [Option<ChunkSection>; 16],
    pub packet_buffer: PacketBuffer,
    
    pub players: HashSet<ClientId>,
    pub entities: HashSet<EntityId>,

    // contains the chunk data packet,
    // and is updated when a player tries to access it and is dirty,
    // ideally we would somehow store blocks in the packet buffer,
    // however it'd be annoying
    cached_chunk_data: PacketBuffer,
    dirty: bool,
}

impl Chunk {
    
    pub fn new() -> Self {
        Self {
            chunk_sections: [const { None }; 16],
            packet_buffer: PacketBuffer::new(),
            players: HashSet::new(),
            entities: HashSet::new(),

            cached_chunk_data: PacketBuffer::new(),
            dirty: true,
        }
    }
    
    pub fn get_block_at(&self, local_x: i32, y: i32, local_z: i32) -> Blocks {
        if let Some(section) = &self.chunk_sections[(y / 16) as usize] {
            let index = ((y & 15) << 8) | (local_z << 4) | local_x;
            return Blocks::from(section.data[index as usize])
        }
        Blocks::Air
    }
    
    pub fn set_block_at(&mut self, block: Blocks, local_x: i32, y: i32, local_z: i32) {
        let section_index = (y / 16) as usize;
        if self.chunk_sections[section_index].is_none() {
            self.chunk_sections[section_index] = Some(ChunkSection {
                solid_block_amount: 0,
                data: Box::new([0; 4096]),
            })
        }
        if let Some(section) = &mut self.chunk_sections[section_index] {
            let block_state_id = block.get_block_state_id();
            let index = ((y & 15) << 8) | (local_z << 4) | local_x;

            if section.data[index as usize] != 0 {
                section.solid_block_amount -= 1;
            }
            if block != Blocks::Air {
                section.solid_block_amount += 1;
            }
            section.data[index as usize] = block_state_id;
            self.dirty = true;
        }
    }

    pub fn write_chunk_data(&mut self, x: i32, z: i32, new: bool, into: &mut PacketBuffer) {
        // this only writes chunks if the x and z are the same,
        // so for an empty chunk this doesn't work
        if self.dirty {
            
            let mut bitmask = 0u16;

            for index in 0..16 {
                if let Some(section) = &self.chunk_sections[index] && section.solid_block_amount != 0 {
                    bitmask |= 1 << index;
                }
            }

            let section_count = bitmask.count_ones() as usize;
            let data_size: usize = section_count * 12288 + if new { 256 } else { 0 };

            let mut data = vec![0u8; data_size];
            let mut offset = 0;

            for section in self.chunk_sections.iter().flatten() {
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
            self.cached_chunk_data.clear();
            self.cached_chunk_data.write_packet(&ChunkData {
                chunk_x: x,
                chunk_z: z,
                is_new_chunk: new,
                bitmask,
                data,
            });
            self.dirty = false;
        }
        into.copy_from(&self.cached_chunk_data);
    }

    pub fn insert_player(&mut self, client_id: ClientId) {
        debug_assert!(!self.players.contains(&client_id), "player already in chunk");
        self.players.insert(client_id);
    }

    pub fn insert_entity(&mut self, entity_id: EntityId) {
        debug_assert!(!self.entities.contains(&entity_id), "entity already in chunk");
        self.entities.insert(entity_id);
    }

    pub fn remove_player(&mut self, client_id: ClientId) {
        debug_assert!(self.players.contains(&client_id), "player was never in this chunk");
        self.players.remove(&client_id);
    }

    pub fn remove_entity(&mut self, entity_id: EntityId) {
        debug_assert!(self.entities.contains(&entity_id), "entity was never in this chunk");
        self.entities.remove(&entity_id);
    }
    
    pub fn has_players(&self) -> bool {
        !self.players.is_empty()
    }

    pub fn has_entities(&self) -> bool {
        !self.entities.is_empty()
    }
}

pub fn get_chunk_position(position: DVec3) -> (i32, i32) {
    let x = (position.x.floor() as i32) >> 4;
    let z = (position.z.floor() as i32) >> 4;
    (x, z)
}