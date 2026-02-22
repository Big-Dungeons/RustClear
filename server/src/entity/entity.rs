use crate::entity::components::EntityAppearance;
use crate::entity::entities::MCEntityId;
use crate::network::packets::packet_buffer::PacketBuffer;
use crate::network::protocol::play::clientbound::DestroyEntites;
use crate::world::chunk::get_chunk_position;
use crate::{Player, World, WorldExtension};
use bevy_ecs::component::Component;
use bevy_ecs::entity::Entity;
use bevy_ecs::world::{EntityMut, EntityRef};
use glam::DVec3;
use std::ptr::NonNull;

#[derive(Component)]
#[derive(Clone)]
pub struct MinecraftEntity<W: WorldExtension> {
    world: NonNull<World<W>>,
    pub id: MCEntityId,

    pub position: DVec3,
    pub velocity: DVec3,
    pub yaw: f32,
    pub pitch: f32,

    pub last_position: DVec3,
    pub last_yaw: f32,
    pub last_pitch: f32,

    pub ticks_existed: u32,

    pub enter_view: fn(entity: &MinecraftEntity<W>, &EntityRef, &mut Player<W::Player>),
    pub leave_view: fn(entity: &MinecraftEntity<W>, &EntityRef, &mut Player<W::Player>),
    pub update_position: fn(entity: &MinecraftEntity<W>, EntityMut, packet_buffer: &mut PacketBuffer),
    pub destroy: fn(entity: &MinecraftEntity<W>, &EntityRef, packet: &mut DestroyEntites)
}

impl<W: WorldExtension + 'static> MinecraftEntity<W> {

    pub fn new<T : EntityAppearance<W>>(
        world: &mut World<W>,
        position: DVec3,
        yaw: f32,
        pitch: f32,
    ) -> MinecraftEntity<W> {
        let entity_id = world.entities.next_entity_id();
        MinecraftEntity {
            world: NonNull::from_mut(world),
            id: entity_id,
            position,
            velocity: DVec3::ZERO,
            yaw,
            pitch,
            last_position: position,
            last_yaw: yaw,
            last_pitch: pitch,
            ticks_existed: 0,

            enter_view: |entity, entity_ref, player| {
                let appearance = entity_ref.get::<T>().unwrap();
                appearance.enter_player_view(entity, player)
            },
            leave_view: |entity, entity_ref, player| {
                let appearance = entity_ref.get::<T>().unwrap();
                appearance.leave_player_view(entity, player)
            },
            update_position: |entity, entity_ref, buffer| {
                let appearance = entity_ref.get::<T>().unwrap();
                appearance.update_position(entity, buffer)
            },
            destroy: |entity, entity_ref, packet| {
                let appearance = entity_ref.get::<T>().unwrap();
                appearance.destroy(entity, packet)
            }
        }
    }

    pub fn world<'a>(&self) -> &'a World<W> {
        unsafe { self.world.as_ref() }
    }

    pub fn world_mut<'a>(&mut self) -> &'a mut World<W> {
        unsafe { self.world.as_mut() }
    }

    pub(crate) fn update<T : EntityAppearance<W>>(&mut self, appearance: &T, entity_id: Entity) {
        if self.position != self.last_position || self.yaw != self.last_yaw || self.pitch != self.last_pitch {
            let (chunk_x, chunk_z) = get_chunk_position(self.position);
            let Some(chunk) = self.world_mut().chunk_grid.get_chunk_mut(chunk_x, chunk_z) else {
                return;
            };

            let (old_cx, old_cz) = get_chunk_position(self.last_position);
            if old_cx != chunk_x || old_cz != chunk_z {
                if let Some(chunk) = self.world_mut().chunk_grid.get_chunk_mut(old_cx, old_cz) {
                    chunk.remove_entity(entity_id)
                }
                chunk.insert_entity(entity_id)
            }

            appearance.update_position(self, &mut chunk.packet_buffer);
            self.last_position = self.position;
            self.last_yaw = self.yaw;
            self.last_pitch = self.pitch;
        }
        self.ticks_existed += 1;
    }

    pub fn destroy(&mut self) {
        let world = self.world_mut();
        let entity = world.entities.mc_id_to_entity(self.id).unwrap();
        world.remove_entity(*entity)
    }
}


unsafe impl<W: WorldExtension> Send for MinecraftEntity<W> {}
unsafe impl<W: WorldExtension> Sync for MinecraftEntity<W> {}