use crate::dungeon::dungeon::Dungeon;
use crate::dungeon::dungeon_player::DungeonPlayer;
use crate::dungeon::items::dungeon_items::DungeonItem;
use bevy_ecs::prelude::Component;
use glam::dvec3;
use indoc::indoc;
use server::block::block_collision::check_block_collisions;
use server::constants::{ObjectVariant, Sound};
use server::entity::components::{EntityAppearance, EntityBehaviour};
use server::entity::entity::MinecraftEntity;
use server::inventory::item_stack::ItemStack;
use server::network::binary::nbt::NBT;
use server::network::binary::var_int::VarInt;
use server::network::packets::packet_buffer::PacketBuffer;
use server::network::protocol::play::clientbound::{DestroyEntites, EntityTeleport, EntityVelocity, PositionLook, Relative, SpawnObject};
use server::player::packet_processing::BlockInteractResult;
use server::types::aabb::AABB;
use server::{ClientId, Player};

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct EnderPearl;

impl DungeonItem for EnderPearl {
    
    fn on_interact(&self, player: &mut Player<DungeonPlayer>, block: Option<BlockInteractResult>) {
        player.sync_inventory();
        if block.is_some() { 
            return;
        }

        let world = player.world_mut();

        let entity = world.spawn_entity(
            player.player_eye_position(),
            player.yaw,
            player.pitch,
            EnderPearlAppearance,
            EnderPearlBehaviour { player_id: player.client_id },
        );

        player.play_sound(Sound::GhastFireball, 0.2, 1.0);

        let mut entity = world.entities.get_entity_mut(entity);
        let mut entity = entity.get_mut::<MinecraftEntity<Dungeon>>().unwrap();
        entity.velocity = player.rotation_vec().as_dvec3() * 1.5;
        
    }
    
    fn item_stack(&self) -> ItemStack {
        ItemStack {
            item: 368,
            stack_size: 16,
            metadata: 0,
            tag_compound: Some(NBT::with_nodes(vec![
                NBT::compound("display", vec![
                    NBT::list_from_string("Lore", indoc! {r#"
                            §8Collection Item

                            §f§lCOMMON
                        "#}),
                    NBT::string("Name", "§fEnder Pearl"),
                ]),
            ])),
        }
    }
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
        // last position may have been inside a block if player was in a block
        let last_aabb = AABB::from_width_height(0.25, 0.25).offset(entity.last_position);

        if check_block_collisions(world, &aabb) || check_block_collisions(world, &last_aabb) {
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