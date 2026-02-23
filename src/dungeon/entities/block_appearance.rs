use crate::dungeon::dungeon::Dungeon;
use crate::dungeon::dungeon_player::DungeonPlayer;
use bevy_ecs::prelude::Component;
use glam::DVec3;
use server::block::Block;
use server::constants::{EntityVariant, ObjectVariant};
use server::entity::components::EntityAppearance;
use server::entity::entity::MinecraftEntity;
use server::entity::entity_metadata::{BatMetadata, EntityMetadata};
use server::network::binary::var_int::VarInt;
use server::network::packets::packet_buffer::PacketBuffer;
use server::network::protocol::play::clientbound::{DestroyEntites, EntityAttach, EntityTeleport, SpawnMob, SpawnObject};
use server::{Player, World};

#[derive(Component)]
pub struct BlockAppearance {
    pub block: Block
}

impl EntityAppearance<Dungeon> for BlockAppearance {

    fn init(&self, world: &mut World<Dungeon>) {
        // allocate one more entity id for the falling block
        world.entities.next_entity_id();
    }

    fn enter_player_view(&self, entity: &MinecraftEntity<Dungeon>, player: &mut Player<DungeonPlayer>) {
        let DVec3 { x, y, z } = entity.position;

        player.write_packet(&SpawnMob {
            entity_id: entity.id,
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
            metadata: EntityMetadata::Bat(BatMetadata {
                flags: 0x20,
                hanging: false,
            }),
        });

        let object_data = {
            let block_state_id = self.block.get_blockstate_id() as i32;
            let block_id = block_state_id >> 4;
            let metadata = block_state_id & 0b1111;
            block_id | (metadata << 12)
        };

        player.write_packet(&SpawnObject {
            entity_id: entity.id + 1,
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
            entity_id: entity.id+ 1,
            vehicle_id: entity.id,
            leash: false,
        });
    }

    fn leave_player_view(&self, entity: &MinecraftEntity<Dungeon>, player: &mut Player<DungeonPlayer>) {
        player.write_packet(&DestroyEntites {
            entities: vec![VarInt(entity.id), VarInt(entity.id + 1)],
        })
    }

    fn update_position(&self, entity: &MinecraftEntity<Dungeon>, packet_buffer: &mut PacketBuffer) {
        packet_buffer.write_packet(&EntityTeleport {
            entity_id: entity.id,
            pos_x: entity.position.x,
            pos_y: entity.position.y - 0.65,
            pos_z: entity.position.z,
            yaw: 0.0,
            pitch: 0.0,
            on_ground: false,
        });
    }

    fn destroy(&self, entity: &MinecraftEntity<Dungeon>, packet: &mut DestroyEntites) {
        packet.entities.push(VarInt(entity.id));
        packet.entities.push(VarInt(entity.id + 1));
    }
}

// doesn't prevent clientside gravity whatsoever, used for falling blocks and in ice fill
#[derive(Component)]
pub struct FallingBlockAppearance {
    pub block: Block
}

impl EntityAppearance<Dungeon> for FallingBlockAppearance {

    fn enter_player_view(&self, entity: &MinecraftEntity<Dungeon>, player: &mut Player<DungeonPlayer>) {
        let DVec3 { x, y, z } = entity.position;

        let object_data = {
            let block_state_id = self.block.get_blockstate_id() as i32;
            let block_id = block_state_id >> 4;
            let metadata = block_state_id & 0b1111;
            block_id | (metadata << 12)
        };

        player.write_packet(&SpawnObject {
            entity_id: entity.id,
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
    }

    fn leave_player_view(&self, entity: &MinecraftEntity<Dungeon>, player: &mut Player<DungeonPlayer>) {
        player.write_packet(&DestroyEntites {
            entities: vec![VarInt(entity.id)],
        })
    }

    fn update_position(&self, entity: &MinecraftEntity<Dungeon>, packet_buffer: &mut PacketBuffer) {
        packet_buffer.write_packet(&EntityTeleport {
            entity_id: entity.id,
            pos_x: entity.position.x,
            pos_y: entity.position.y,
            pos_z: entity.position.z,
            yaw: 0.0,
            pitch: 0.0,
            on_ground: false,
        });
    }

    fn destroy(&self, entity: &MinecraftEntity<Dungeon>, packet: &mut DestroyEntites) {
        packet.entities.push(VarInt(entity.id));
    }
}