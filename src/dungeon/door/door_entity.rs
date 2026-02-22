use crate::dungeon::dungeon::Dungeon;
use crate::dungeon::dungeon_player::DungeonPlayer;
use bevy_ecs::prelude::Component;
use glam::{ivec3, DVec3};
use server::block::Block;
use server::constants::{EntityVariant, ObjectVariant};
use server::entity::components::{EntityAppearance, EntityBehaviour};
use server::entity::entity::MinecraftEntity;
use server::entity::entity_metadata::EntityMetadata;
use server::network::binary::var_int::VarInt;
use server::network::packets::packet_buffer::PacketBuffer;
use server::network::protocol::play::clientbound::{DestroyEntites, EntityAttach, EntityRelativeMove, SpawnMob, SpawnObject};
use server::{Player, World};

#[derive(Component)]
pub struct DoorBehaviour;

impl EntityBehaviour<Dungeon> for DoorBehaviour {
    fn tick(entity: &mut MinecraftEntity<Dungeon>, _: &mut Self) {
        entity.position.y -= 0.25;

        if entity.ticks_existed == 20 {
            let world = entity.world_mut();
            let x = entity.position.x as i32;
            let z = entity.position.z as i32;

            world.chunk_grid.fill_blocks(
                Block::Air,
                ivec3(x, 69, z),
                ivec3(x + 2, 72, z + 2),
            );
            entity.destroy()
        }
    }
}

#[derive(Component)]
pub(super) struct DoorEntity {
    block: Block
}

impl DoorEntity {
    pub fn spawn_into_world(
        world: &mut World<Dungeon>,
        position: DVec3,
        block: Block
    ) {
        world.spawn_entity(position, 0.0, 0.0, DoorEntity { block }, DoorBehaviour);
        for _ in 0..72 {
            world.entities.next_entity_id();
        }
    }
}

impl EntityAppearance<Dungeon> for DoorEntity {
    fn enter_player_view(&self, entity: &MinecraftEntity<Dungeon>, player: &mut Player<DungeonPlayer>) {
        let mut iter = 0;
        for x in 0..3 {
            for y in 0..4 {
                for z in 0..3 {
                    let x = entity.position.x + (x as f64) + 0.5;
                    let y = entity.position.y + (y as f64);
                    let z = entity.position.z + (z as f64) + 0.5;

                    player.write_packet(&SpawnMob {
                        entity_id: entity.id + iter,
                        entity_variant: EntityVariant::Bat,
                        x,
                        y: y - 0.65,
                        z,
                        yaw: 0.0,
                        pitch: 0.0,
                        head_yaw: 0.0,
                        velocity_x: 0.0,
                        velocity_y: 0.0,
                        velocity_z: 0.0,
                        metadata: EntityMetadata::Bat(Default::default()),
                    });

                    let object_data = {
                        let block_state_id = self.block.get_blockstate_id() as i32;
                        let block_id = block_state_id >> 4;
                        let metadata = block_state_id & 0b1111;
                        block_id | (metadata << 12)
                    };

                    player.write_packet(&SpawnObject {
                        entity_id: entity.id + iter + 1,
                        variant: ObjectVariant::FallingBlock,
                        x,
                        y,
                        z,
                        pitch: 0.0,
                        yaw: 0.0,
                        data: object_data,
                        velocity_x: 0.0,
                        velocity_y: 0.0,
                        velocity_z: 0.0,
                    });

                    player.write_packet(&EntityAttach {
                        entity_id: entity.id + iter + 1,
                        vehicle_id: entity.id + iter,
                        leash: false,
                    });

                    iter += 2;
                }
            }
        }
    }
    fn leave_player_view(&self, entity: &MinecraftEntity<Dungeon>, player: &mut Player<DungeonPlayer>) {
        player.write_packet(&DestroyEntites {
            entities: (entity.id..entity.id + 72).map(VarInt).collect(),
        })
    }
    fn update_position(&self, entity: &MinecraftEntity<Dungeon>, packet_buffer: &mut PacketBuffer) {
        // only y can be updated
        let difference = entity.position.y - entity.last_position.y;

        for entity_id in (entity.id..entity.id + 72).step_by(2) {
            packet_buffer.write_packet(&EntityRelativeMove {
                entity_id,
                pos_x: 0.0,
                pos_y: difference,
                pos_z: 0.0,
                on_ground: false,
            });
        }
    }
    fn destroy(&self, entity: &MinecraftEntity<Dungeon>, packet: &mut DestroyEntites) {
        packet.entities.extend((entity.id..entity.id + 72).map(VarInt))
    }
}