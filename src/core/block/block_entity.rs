use bevy::prelude::*;
use glam::IVec3;
use bytes::BytesMut;
use crate::core::chunk::chunk_grid::ChunkGrid;
use crate::core::network::packets::BytesMutExt;
use crate::core::network::protocol::block_position::BlockPosition;
use crate::core::network::protocol::nbt::{byte, compound, compound_node, int, list, string, NBTNode, NBT, TAG_COMPOUND_ID};
use crate::core::network::protocol::play::clientbound::UpdateBlockEntity;
use crate::core::player::PlayerSkin;
use crate::dungeon::rng::DHashMap;

pub enum BlockEntityType {
    MobSpawner,
    CommandBlock,
    Beacon,
    Skull {
        rotation: i8,
        skull_type: i8,
        uuid: uuid::Uuid,
        skin: PlayerSkin,
    },
    FlowerPot,
    Banner,
}

impl BlockEntityType {
    pub fn id(&self) -> u8 {
        match self {
            Self::MobSpawner => 1,
            Self::CommandBlock => 2,
            Self::Beacon => 3,
            Self::Skull { .. } => 4,
            Self::FlowerPot => 5,
            Self::Banner => 6,
        }
    }
}

#[derive(Component)]
pub struct BlockEntity {
    pub position: IVec3,
    pub block_entity_type: BlockEntityType,
    cached_nbt: NBT
}

impl BlockEntity {
    pub fn new(position: IVec3, block_entity: BlockEntityType) -> Self {
        Self {
            position,
            block_entity_type: block_entity,
            cached_nbt: NBT::new(),
        }
    }

    pub fn write_update_packet(&self, packet_buffer: &mut BytesMut) {
        packet_buffer.write_packet(&UpdateBlockEntity {
            position: BlockPosition(self.position),
            block_entity_id: self.block_entity_type.id(),
            nbt: Some(self.cached_nbt.clone())
        })
    }

    fn update_nbt(&mut self) {
        self.cached_nbt.nodes.clear();
        let mut nbt = self.cached_nbt.builder();

        nbt.insert([
            int("x", self.position.x),
            int("y", self.position.y),
            int("z", self.position.z),
        ]);

        #[allow(clippy::single_match)]
        match &self.block_entity_type {
            BlockEntityType::Skull { rotation, skull_type, uuid, skin } => {
                nbt.insert([
                    byte("SkullType", *skull_type),
                    byte("Rot", *rotation),
                    compound("Owner", [
                        string("Id", uuid.hyphenated().to_string()),
                        compound("Properties", [
                            list::<TAG_COMPOUND_ID>("textures", [
                                compound_node([
                                    string("Value", skin.texture.clone()),
                                ]),
                            ])
                        ])
                    ])
                ]);
            }
            _ => {}
        }
    }
}

pub fn on_add_block_entity(
    event: On<Add, BlockEntity>,
    mut query: Query<&mut BlockEntity>,
    mut chunks: ResMut<ChunkGrid>,
) {
    let mut block_entity = query
        .get_mut(event.entity)
        .unwrap();

    let chunk = chunks
        .get_mut_from_world(block_entity.position)
        .expect("spawned block entity into an invalid chunk");

    block_entity.bypass_change_detection().update_nbt();
    chunk.insert_block_entity(event.entity, block_entity.position);
    block_entity.write_update_packet(&mut chunk.packet_buffer);
}

pub fn on_remove_block_entity(
    event: On<Remove, BlockEntity>,
    query: Query<&BlockEntity>,
    mut chunks: ResMut<ChunkGrid>,
) {
    let block_entity = query
        .get(event.entity)
        .unwrap();

    let chunk = chunks
        .get_mut_from_world(block_entity.position)
        .expect("block entity, that was despawned, was in an invalid chunk");

    chunk.remove_block_entity(block_entity.position);
}

pub fn on_block_entity_change(
    mut query: Query<&mut BlockEntity, Changed<BlockEntity>>,
    mut chunks: ResMut<ChunkGrid>,
) {
    for mut block_entity in query.iter_mut() {
        let chunk = chunks
            .get_mut_from_world(block_entity.position)
            .unwrap();

        block_entity.bypass_change_detection().update_nbt();
        block_entity.write_update_packet(&mut chunk.packet_buffer);
    }
}