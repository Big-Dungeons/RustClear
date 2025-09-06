use crate::entity::entity_metadata::EntityMetadata;
use crate::get_chunk_position;
use crate::network::binary::var_int::VarInt;
use crate::network::packets::packet_buffer::PacketBuffer;
use crate::network::protocol::play::clientbound::{DestroyEntites, EntityTeleport, SpawnMob, SpawnObject};
use crate::player::player::{Player, PlayerExtension};
use crate::world::chunk::chunk_grid::ChunkDiff;
use crate::world::world::{World, WorldExtension, VIEW_DISTANCE};
use glam::DVec3;

pub type EntityId = i32;

#[allow(unused)]
pub trait EntityImpl<W : WorldExtension, P : PlayerExtension = <W as WorldExtension>::Player> {
    
    fn spawn(&mut self, entity: &mut EntityBase<W>, _: &mut PacketBuffer);
    
    fn despawn(&mut self, entity: &mut EntityBase<W>, _: &mut PacketBuffer);
    
    fn tick(&self, entity: &mut EntityBase<W>, _: &mut PacketBuffer);
    
    fn interact(&self, entity: &mut EntityBase<W>, player: &mut Player<P>) {}
}

pub struct EntityBase<W : WorldExtension> {
    world: *mut World<W>,
    pub id: EntityId,
    
    pub position: DVec3,
    pub velocity: DVec3,
    pub yaw: f32,
    pub pitch: f32,
    
    pub last_position: DVec3,
    pub last_yaw: f32,
    pub last_pitch: f32,
    
    pub metadata: EntityMetadata,
}

impl<W : WorldExtension> EntityBase<W> {

    pub fn world<'a>(&self) -> &'a World<W> {
        unsafe { self.world.as_ref().expect("world is null") }
    }

    pub fn world_mut<'a>(&self) -> &'a mut World<W> {
        unsafe { self.world.as_mut().expect("world is null") }
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

    // idk how to feel about handling despawning
    pub fn write_despawn_packet(&self, buffer: &mut PacketBuffer) {
        buffer.write_packet(&DestroyEntites {
            entities: vec![VarInt(self.id)],
        })
    }
} 

pub struct Entity<W : WorldExtension> {
    pub base: EntityBase<W>,
    pub entity_impl: Box<dyn  EntityImpl<W>>,
}

impl<W : WorldExtension> Entity<W> {
    
    pub fn new<E : EntityImpl<W> + 'static>(
        world: *mut World<W>,
        entity_metadata: EntityMetadata,
        entity_id: EntityId,
        position: DVec3,
        yaw: f32,
        pitch: f32,
        entity_impl: E
    ) -> Self {
        let entity_base = EntityBase {
            world,
            id: entity_id,
            position,
            velocity: DVec3::ZERO,
            yaw,
            pitch,
            last_position: position,
            last_yaw: yaw,
            last_pitch: pitch,
            metadata: entity_metadata,
        };
        Self {
            base: entity_base,
            entity_impl: Box::new(entity_impl),
        }
    }
    
    pub fn tick(&mut self, packet_buffer: &mut PacketBuffer) {
        let entity = &mut self.base;
        self.entity_impl.tick(entity, packet_buffer);
        
        if entity.position != entity.last_position || entity.yaw != entity.last_yaw || entity.pitch != entity.last_pitch {
            packet_buffer.write_packet(&EntityTeleport {
                entity_id: entity.id,
                pos_x: entity.position.x,
                pos_y: entity.position.y,
                pos_z: entity.position.z,
                yaw: entity.yaw,
                pitch: entity.pitch,
                on_ground: true,
            });
            
            let chunk_position = get_chunk_position(entity.position);
            let last_chunk_position = get_chunk_position(entity.last_position);
            
            if chunk_position != last_chunk_position { 
                let world = entity.world_mut();
                let chunk_grid = &mut world.chunk_grid;
                entity.world().chunk_grid.for_each_diff(
                    chunk_position,
                    last_chunk_position,
                    VIEW_DISTANCE,
                    |x, z, diff| {
                        // shouldn't be none?
                        let Some(chunk) = chunk_grid.get_chunk_mut(x, z) else {
                            return;
                        };
                        if diff == ChunkDiff::New {
                            entity.write_spawn_packet(&mut chunk.packet_buffer);
                            self.entity_impl.spawn(entity, &mut chunk.packet_buffer);
                        }
                    }
                )
            }
            
        } /*else if entity.yaw != entity.last_yaw || entity.pitch != entity.last_pitch {
            packet_buffer.write_packet(&EntityRotate {
                entity_id: entity.id,
                yaw: wrap_yaw(entity.yaw),
                pitch: entity.pitch,
                on_ground: false,
            })
        }*/
        entity.last_position = entity.position;
        entity.last_yaw = entity.yaw;
        entity.last_pitch = entity.pitch;
    }
    
    pub fn spawn(&mut self) {
        let entity = &mut self.base;
        let (chunk_x, chunk_z) = get_chunk_position(entity.position);

        let Some(chunk) = entity.world_mut().chunk_grid.get_chunk_mut(chunk_x, chunk_z) else {
            // early return since entity isn't in a valid chunk,
            // there is no buffer to write spawn packets.
            return;
        };

        let packet_buffer = &mut chunk.packet_buffer;
        entity.write_spawn_packet(packet_buffer);
        self.entity_impl.spawn(entity, packet_buffer);
    }
}