use crate::dungeon::dungeon::Dungeon;
use crate::dungeon::dungeon_player::DungeonPlayer;
use bevy_ecs::component::Component;
use glam::dvec3;
use server::block::block_collision::check_block_collisions;
use server::constants::ObjectVariant;
use server::entity::components::{EntityAppearance, EntityBehaviour};
use server::entity::entity::MinecraftEntity;
use server::network::binary::var_int::VarInt;
use server::network::packets::packet_buffer::PacketBuffer;
use server::network::protocol::play::clientbound::{DestroyEntites, EntityTeleport, EntityVelocity, PositionLook, Relative, SpawnObject};
use server::types::aabb::AABB;
use server::{ClientId, Player};

pub fn throw_pearl(player: &mut Player<DungeonPlayer>) {
    let world = player.world_mut();

    let entity = world.spawn_entity(
        player.player_eye_position(),
        player.yaw,
        player.pitch,
        EnderPearlAppearance,
        EnderPearlBehaviour { player_id: player.client_id },
    );

    let mut entity = world.entities.get_entity_mut(entity);
    let entity = entity.get_mut::<MinecraftEntity<Dungeon>>();
    entity.unwrap().velocity = player.rotation_vec().as_dvec3() * 1.5;
}


#[derive(Component)]
pub struct EnderPearlBehaviour {
    player_id: ClientId
}

impl EntityBehaviour<Dungeon> for EnderPearlBehaviour {
    fn tick(entity: &mut MinecraftEntity<Dungeon>, component: &mut Self) {
        // todo: sounds
        const GRAVITY: f64 = 0.03;
        const DRAG: f64 = 0.99;

        entity.velocity.y -= GRAVITY;
        entity.velocity *= DRAG;
        entity.position += entity.velocity;

        let world = entity.world_mut();
        let aabb = AABB::from_width_height(0.25, 0.25).offset(entity.position);
        if check_block_collisions(world, &aabb) {
            if let Some(index) = world.player_map.get(component.player_id) {
                let player_rc = &mut world.players[*index];
                let player = unsafe { &mut *player_rc.get() };

                let land_pos = dvec3(
                    entity.last_position.x.floor() + 0.5,
                    entity.last_position.y.floor() + 1.0,
                    entity.last_position.z.floor() + 0.5,
                );

                player.write_packet(&PositionLook {
                    x: land_pos.x,
                    y: land_pos.y,
                    z: land_pos.z,
                    yaw: 0.0,
                    pitch: 0.0,
                    flags: Relative::Yaw | Relative::Pitch,
                });
            }
            entity.destroy()
        }

        // prevent falling into void infinitely or something
        if entity.ticks_existed == 100 {
            entity.destroy()
        }
    }
}

#[derive(Component)]
pub struct EnderPearlAppearance;

impl EntityAppearance<Dungeon> for EnderPearlAppearance {
    fn enter_player_view(&self, entity: &MinecraftEntity<Dungeon>, player: &mut Player<DungeonPlayer>) {
        player.write_packet(&SpawnObject {
            entity_id: entity.id,
            variant: ObjectVariant::EnderPearl,
            x: entity.position.x,
            y: entity.position.y,
            z: entity.position.z,
            pitch: entity.yaw,
            yaw: entity.pitch,
            data: player.entity_id,
            velocity_x: entity.velocity.x,
            velocity_y: entity.velocity.y,
            velocity_z: entity.velocity.z,
        });
    }
    fn leave_player_view(&self, entity: &MinecraftEntity<Dungeon>, player: &mut Player<DungeonPlayer>) {
        player.write_packet(&DestroyEntites {
            entities: vec![VarInt(entity.id)],
        });
    }
    fn update_position(&self, entity: &MinecraftEntity<Dungeon>, packet_buffer: &mut PacketBuffer) {
        if entity.ticks_existed.is_multiple_of(10) {
            packet_buffer.write_packet(&EntityTeleport {
                entity_id: entity.id,
                pos_x: entity.position.x,
                pos_y: entity.position.y,
                pos_z: entity.position.z,
                yaw: 0.0,
                pitch: 0.0,
                on_ground: false,
            });
            packet_buffer.write_packet(&EntityVelocity {
                entity_id: entity.id,
                velocity_x: entity.velocity.x,
                velocity_y: entity.velocity.y,
                velocity_z: entity.velocity.z,
            })
        }
    }
    fn destroy(&self, entity: &MinecraftEntity<Dungeon>, packet: &mut DestroyEntites) {
        packet.entities.push(VarInt(entity.id))
    }
}