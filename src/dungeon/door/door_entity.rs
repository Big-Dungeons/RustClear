use crate::dungeon::dungeon::Dungeon;
use glam::ivec3;
use server::block::blocks::Blocks;
use server::entity::entity::{EntityBase, EntityImpl};
use server::entity::entity_metadata::{EntityMetadata, EntityVariant};
use server::network::binary::var_int::VarInt;
use server::network::packets::packet_buffer::PacketBuffer;
use server::network::protocol::play::clientbound::{DestroyEntites, EntityAttach, EntityRelativeMove, SpawnMob, SpawnObject};

pub struct DoorEntityImpl {
    pub block: Blocks,
    pub x: i32,
    pub z: i32,
}

// currently is specifically made for entrance, wither and blood doors only
impl EntityImpl<Dungeon> for DoorEntityImpl {
    fn spawn(&mut self, entity: &mut EntityBase<Dungeon>, buffer: &mut PacketBuffer) {
        let world = entity.world_mut();

        for x in self.x..self.x + 3 {
            for y in 69..=72 {
                for z in self.z..self.z + 3 {

                    let bat_id = world.new_entity_id();
                    let block_id = world.new_entity_id();

                    let metadata = EntityMetadata {
                        variant: EntityVariant::Bat {
                            hanging: false
                        },
                        is_invisible: true,
                    };
                    buffer.write_packet(&SpawnMob {
                        entity_id: VarInt(bat_id),
                        entity_variant: metadata.variant.get_id(),
                        x: x as f64 + 0.5,
                        y: y as f64 - 0.65,
                        z: z as f64 + 0.5,
                        yaw: 0.0,
                        pitch: 0.0,
                        head_yaw: 0.0,
                        velocity_x: 0.0,
                        velocity_y: 0.0,
                        velocity_z: 0.0,
                        metadata,
                    });

                    let object_data = {
                        let block_state_id = self.block.get_block_state_id() as i32;
                        let block_id = block_state_id >> 4;
                        let metadata = block_state_id & 0b1111;
                        block_id | (metadata << 12)
                    };

                    buffer.write_packet(&SpawnObject {
                        entity_id: VarInt(block_id),
                        entity_variant: 70,
                        x: x as f64 + 0.5,
                        y: y as f64,
                        z: z as f64 + 0.5,
                        pitch: 0.0,
                        yaw: 0.0,
                        data: object_data,
                        velocity_x: 0.0,
                        velocity_y: 0.0,
                        velocity_z: 0.0,
                    });

                    buffer.write_packet(&EntityAttach {
                        entity_id: block_id,
                        vehicle_id: bat_id,
                        leash: false,
                    })
                }
            }
        }
    }

    fn despawn(&mut self, entity: &mut EntityBase<Dungeon>, buffer: &mut PacketBuffer) {
        buffer.write_packet(&DestroyEntites {
            entities: (entity.id + 1..=entity.id + 72).map(|index| VarInt(index)).collect(),
        })
    }

    fn tick(&self, entity: &mut EntityBase<Dungeon>, buffer: &mut PacketBuffer) {
        for entity_id in (entity.id + 1..=entity.id + 72).step_by(2) {
             buffer.write_packet(&EntityRelativeMove {
                 entity_id,
                 pos_x: 0.0,
                 pos_y: -0.25,
                 pos_z: 0.0,
                 on_ground: false,
             })
        }
        if entity.ticks_existed == 20 {
            let world = entity.world_mut();
            world.remove_entity(entity.id);
            world.chunk_grid.fill_blocks(
                Blocks::Air,
                ivec3(self.x, 69, self.z),
                ivec3(self.x + 2, 72, self.z + 2),
            );
        }
    }
}