use crate::dungeon::dungeon::Dungeon;
use crate::dungeon::dungeon_player::DungeonPlayer;
use glam::ivec3;
use server::block::Block;
use server::constants::{EntityVariant, ObjectVariant};
use server::entity::entity::{EntityBase, EntityExtension};
use server::entity::entity_appearance::EntityAppearance;
use server::entity::entity_metadata::EntityMetadata;
use server::network::binary::var_int::VarInt;
use server::network::packets::packet_buffer::PacketBuffer;
use server::network::protocol::play::clientbound::{DestroyEntites, EntityAttach, EntityRelativeMove, SpawnMob, SpawnObject};
use server::Player;

pub(super) struct DoorEntityAppearance {
    pub block: Block,
}

impl EntityAppearance<Dungeon> for DoorEntityAppearance {

    fn initialize(&self, entity: &mut EntityBase<Dungeon>) {
        // reserve 72 entity ids
        let world = entity.world_mut();
        for _ in 0..72 {
            world.new_entity_id();
        }
    }

    fn destroy(&self, entity: &mut EntityBase<Dungeon>, packet: &mut DestroyEntites) {
        packet.entities.extend((entity.id..entity.id + 72).map(VarInt))
    }
    fn enter_player_view(&self, entity: &mut EntityBase<Dungeon>, player: &mut Player<DungeonPlayer>) {
        // println!("player {}, entity pos {:?}", player.profile.username, entity.position);
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

    fn leave_player_view(&self, entity: &mut EntityBase<Dungeon>, player: &mut Player<DungeonPlayer>) {
        player.write_packet(&DestroyEntites {
            entities: (entity.id..entity.id + 72).map(VarInt).collect(),
        })
    }

    fn update_position(&self, entity: &mut EntityBase<Dungeon>, packet_buffer: &mut PacketBuffer) {
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

    fn update_rotation(&self, _: &mut EntityBase<Dungeon>, _: &mut PacketBuffer) {}
}

pub(super) struct DoorEntityExtension;

impl EntityExtension<Dungeon> for DoorEntityExtension {

    fn tick(&mut self, entity: &mut EntityBase<Dungeon>, _: &mut PacketBuffer) {
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
            world.remove_entity(entity.id);
        }
    }
}