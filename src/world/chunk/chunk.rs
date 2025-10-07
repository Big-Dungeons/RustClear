use crate::block::blocks::Blocks;
use crate::entity::entity::EntityId;
use crate::network::packets::packet_buffer::PacketBuffer;
use crate::network::protocol::play::clientbound::ChunkData;
use crate::player::player::ClientId;

pub struct ChunkSection {
    solid_block_amount: u16,
    data: [u16; 4096],
}

// could make the data a packet buffer? like store the chunk just as the packet
// so instead of serializing the chunk for packet you just copy chunk packet buffer

pub struct Chunk {
    pub chunk_sections: [Option<ChunkSection>; 16],
    pub packet_buffer: PacketBuffer,
    pub players: Vec<ClientId>,
    pub entities: Vec<EntityId>
}

impl Chunk {
    
    pub fn new() -> Self {
        Self {
            chunk_sections: [const { None }; 16],
            packet_buffer: PacketBuffer::new(),
            players: Vec::new(),
            entities: Vec::new(),
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
                data: [0; 4096],
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
        }
    }

    pub fn get_chunk_data(&self, x: i32, z: i32, new: bool) -> ChunkData {
        let mut bitmask = 0u16;
        
        for index in 0..16 {
            if let Some(section) = &self.chunk_sections[index] {
                if section.solid_block_amount != 0 {
                    bitmask |= 1 << index;
                }
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
            for block in section.data {
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
        ChunkData {
            chunk_x: x,
            chunk_z: z,
            is_new_chunk: new,
            bitmask,
            data,
        }
    }

    // maybe use hashset instead for these methods?

    pub fn insert_player(&mut self, client_id: ClientId) {
        self.players.push(client_id)
    }
    
    pub fn insert_entity(&mut self, entity_id: EntityId) {
        self.entities.push(entity_id)
    }
    
    pub fn remove_player(&mut self, client_id: ClientId) {
        if let Some(index) = self.players.iter().position(|id| *id == client_id) {
            self.players.remove(index);
        }
    }

    pub fn remove_entity(&mut self, entity_id: EntityId) {
        if let Some(index) = self.entities.iter().position(|id| *id == entity_id) {
            self.entities.remove(index);
        }
    }
    
    pub fn has_players(&self) -> bool {
        !self.players.is_empty()
    }
}