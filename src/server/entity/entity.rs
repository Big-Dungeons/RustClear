use crate::net::packets::packet_buffer::PacketBuffer;
use crate::net::protocol::play::clientbound::{EntityTeleport, SpawnMob, SpawnObject};
use crate::net::protocol::play::serverbound::EntityInteractionType;
use crate::net::var_int::VarInt;
use crate::server::chunk::chunk::Chunk;
use crate::server::entity::entity_metadata::EntityMetadata;
use crate::server::player::player::Player;
use crate::server::world::World;
use glam::DVec3;

pub type EntityId = i32;

/// provides functionality to an entity
pub trait EntityImpl {
    
    fn spawn(&mut self, _: &mut Entity, _: &mut PacketBuffer) {}
    
    fn despawn(&mut self, _: &mut Entity, _: &mut PacketBuffer) {}

    /// runs when an entity is ticked
    /// used to add custom functionality to an entity
    fn tick(&mut self, entity: &mut Entity, packet_buffer: &mut PacketBuffer);
    
    fn interact(&mut self, _: &mut Entity, _: &mut Player, _: &EntityInteractionType) {}
}

/// represents an entity, its position, rotation, and its variant
pub struct Entity {
    world: *mut World,
    pub id: EntityId,

    pub position: DVec3,
    pub velocity: DVec3,
    pub yaw: f32,
    pub pitch: f32,
    pub on_ground: bool,

    pub last_position: DVec3,
    pub last_yaw: f32,
    pub last_pitch: f32,
    
    pub ticks_existed: u32,
    
    pub metadata: EntityMetadata,
}

impl Entity {

    pub fn new(
        world: *mut World,
        id: EntityId,
        position: DVec3,
        metadata: EntityMetadata,
    ) -> Self {
        Self {
            world,
            id,
            position,
            velocity: DVec3::ZERO,
            yaw: 0.0,
            pitch: 0.0,
            on_ground: false,
            last_position: DVec3::ZERO,
            last_yaw: 0.0,
            last_pitch: 0.0,
            ticks_existed: 0,
            metadata,
        }
    }

    pub fn world_mut<'a>(&self) -> &'a mut World {
        unsafe { self.world.as_mut().unwrap() }
    }
    
    pub fn enter_view() {
        
    }
    
    pub fn write_spawn_packet(&self, buffer: &mut PacketBuffer) {
        let variant = &self.metadata.variant;
        if variant.is_player() {
            // needs player list item
            // buffer.write_packet(&SpawnPlayer {
            //     entity_id: VarInt(self.id),
            //     uuid,
            //     x: self.position.x,
            //     y: self.position.y,
            //     z: self.position.z,
            //     yaw: self.yaw,
            //     pitch: self.pitch,
            //     current_item: 0,
            //     metadata: self.metadata.clone(),
            // });
        } else if variant.is_object() { 
            buffer.write_packet(&SpawnObject {
                entity_id: VarInt(self.id),
                entity_variant: variant.get_id(),
                x: self.position.x,
                y: self.position.y,
                z: self.position.z,
                yaw: self.yaw,
                pitch: self.pitch,
                data: 0,
                velocity_x: self.velocity.x,
                velocity_y: self.velocity.y,
                velocity_z: self.velocity.z,
            })
        } else {
            buffer.write_packet(&SpawnMob {
                entity_id: VarInt(self.id),
                entity_variant: variant.get_id(),
                x: self.position.x,
                y: self.position.y,
                z: self.position.z,
                yaw: self.yaw,
                pitch: self.pitch,
                head_yaw: self.yaw,
                velocity_x: self.velocity.x,
                velocity_y: self.velocity.y,
                velocity_z: self.velocity.z,
                metadata: self.metadata.clone(),
            });
        }
    } 
    
    pub fn tick(
        &mut self,
        entity_impl: &mut Box<dyn EntityImpl>,
        packet_buffer: &mut PacketBuffer
    ) {
        entity_impl.tick(self, packet_buffer);
        
        if self.position != self.last_position {
            packet_buffer.write_packet(&EntityTeleport {
                entity_id: self.id,
                pos_x: self.position.x,
                pos_y: self.position.y,
                pos_z: self.position.z,
                yaw: self.yaw,
                pitch: self.pitch,
                on_ground: self.on_ground,
            });
            self.last_position = self.position;
        }
        self.ticks_existed += 1;
    }
    
    pub fn chunk_position(&self) -> (i32, i32) {
        ((self.position.x.floor() as i32) >> 4, (self.position.z.floor() as i32) >> 4)
    }
    
    pub fn chunk_mut<'a>(&self) -> Option<&'a mut Chunk> {
        let (x, z) = self.chunk_position();
        self.world_mut().chunk_grid.get_chunk_mut(x, z)
    }
}


/// used for entities with no implementation
pub struct NoEntityImpl;

impl EntityImpl for NoEntityImpl {
    fn tick(&mut self, _: &mut Entity, _: &mut PacketBuffer) {}
}
