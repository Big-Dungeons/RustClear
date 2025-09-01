use crate::net::packets::packet_buffer::PacketBuffer;
use crate::net::protocol::play::clientbound::DestroyEntites;
use crate::net::var_int::VarInt;
use crate::server::block::block_interact_action::BlockInteractAction;
use crate::server::block::block_position::BlockPos;
use crate::server::block::blocks::Blocks;
use crate::server::chunk::chunk_grid::ChunkGrid;
use crate::server::entity::entity::{Entity, EntityId, EntityImpl};
use crate::server::entity::entity_metadata::EntityMetadata;
use crate::server::player::player::{ClientId, Player};
use crate::server::server::Server;
use crate::server::utils::player_list::PlayerList;
use glam::DVec3;
use std::collections::HashMap;
use std::mem::take;

pub const VIEW_DISTANCE: u8 = 6;

pub struct World {
    /// Don't use directly!!, use .server_mut() instead
    /// This is unsafe,
    /// but since server should be alive for the entire program this is fine (I hope)
    pub server: *mut Server,

    pub chunk_grid: ChunkGrid,
    pub interactable_blocks: HashMap<BlockPos, BlockInteractAction>,

    pub player_info: PlayerList, // might need to be per player, not sure.

    // entity ids are always positive so they could theoretically be unsigned but minecraft uses signed ints in vanilla and casting might cause weird behavior, also assumes we ever reach the end of i32 though so it might be fine
    pub next_entity_id: i32,
    pub players: HashMap<ClientId, Player>,
    pub entities: HashMap<EntityId, (Entity, Box<dyn EntityImpl>)>,

    pub entities_for_removal: Vec<EntityId>,

    // pub commands: Vec<Command>
    
    // pub player_info: PlayerList,
    pub spawn_point: DVec3,
    pub spawn_yaw: f32,
    pub spawn_pitch: f32,
}

impl World {

    pub fn new() -> World {
        World {
            server: std::ptr::null_mut(),

            chunk_grid: ChunkGrid::new(16, 13, 13),
            interactable_blocks: HashMap::new(),

            player_info: PlayerList::new(),

            next_entity_id: 1, // might have to start at 1
            players: HashMap::new(),
            entities: HashMap::new(),
            entities_for_removal: Vec::new(),

            spawn_point: DVec3::ZERO,
            spawn_yaw: 0.0,
            spawn_pitch: 0.0,
        }
    }

    pub fn server_mut<'a>(&self) -> &'a mut Server {
        unsafe { self.server.as_mut().expect("server is null") }
    }

    pub fn new_entity_id(&mut self) -> EntityId {
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        id
    }

    pub fn spawn_entity<E : EntityImpl + 'static>(&mut self,position: DVec3, metadata: EntityMetadata, mut entity_impl: E,) -> anyhow::Result<EntityId> {
        let world_ptr: *mut World = self;
        let mut entity = Entity::new(
            world_ptr,
            self.new_entity_id(),
            position,
            metadata,
        );

        let chunk_x = (entity.position.x.floor() as i32) >> 4;
        let chunk_z = (entity.position.z.floor() as i32) >> 4;
        
        if let Some(chunk) = self.chunk_grid.get_chunk_mut(chunk_x, chunk_z) {
            chunk.insert_entity(entity.id);
            entity.write_spawn_packet(&mut chunk.packet_buffer);
            entity_impl.spawn(&mut entity, &mut chunk.packet_buffer);
        }

        let id = entity.id;
        self.entities.insert(id, (entity, Box::new(entity_impl)));
        Ok(id)
    }

    /// adds the entity id to 
    pub fn despawn_entity(&mut self, entity_id: EntityId) {
        self.entities_for_removal.push(entity_id)
    }

    pub fn tick(&mut self) -> anyhow::Result<()> {
        if !self.entities_for_removal.is_empty() {
            for entity_id in take(&mut self.entities_for_removal) {
                if let Some((mut entity, mut entity_impl)) = self.entities.remove(&entity_id) {
                    if let Some(chunk) = entity.chunk_mut() {
                        chunk.packet_buffer.write_packet(&DestroyEntites {
                            entities: vec![VarInt(entity.id)],
                        });
                        entity_impl.despawn(&mut entity, &mut chunk.packet_buffer);
                        chunk.remove_entity(&entity_id);
                    }
                }
            }
        }

        for (entity, entity_impl) in self.entities.values_mut() {
            let packet_buffer = if let Some(chunk) = entity.chunk_mut() {
                &mut chunk.packet_buffer
            } else {
                // throwaway packet buffer, doesn't feel like a good idea
                // however there is a chance that an entity might end up outside chunk grid 
                // and end up stuck if we tick only inside a valid chunk
                &mut PacketBuffer::new()
            };
            entity.tick(entity_impl, packet_buffer);
        }
        Ok(())
    }

    pub fn set_block_at(&mut self, block: Blocks, x: i32, y: i32, z: i32) {
        self.chunk_grid.set_block_at(block, x, y, z);
    }

    pub fn get_block_at(&self, x: i32, y: i32, z: i32) -> Blocks {
        self.chunk_grid.get_block_at(x, y, z)
    }

    pub fn set_spawn_point(&mut self, position: DVec3, yaw: f32, pitch: f32) {
        self.spawn_point = position;
        self.spawn_yaw = yaw;
        self.spawn_pitch = pitch;
    }

    pub fn fill_blocks(&mut self, block: Blocks, start: BlockPos, end: BlockPos) {
        iterate_blocks(start, end, |x, y, z| {
            self.set_block_at(block, x, y, z)
        })
    }
}

/// iterates over the blocks in area between start and end
/// and runs a function
#[inline(always)]
pub fn iterate_blocks<F>(
    start: BlockPos,
    end: BlockPos,
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