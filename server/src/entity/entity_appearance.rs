use crate::constants::EntityVariant;
use crate::entity::entity::EntityBase;
use crate::entity::entity_metadata::{EntityMetadata, PlayerMetadata};
use crate::network::binary::var_int::VarInt;
use crate::network::protocol::play::clientbound::{DestroyEntites, EntityRotate, EntityTeleport, EntityYawRotate, PlayerData, PlayerListItem, SpawnMob, SpawnPlayer};
use crate::world::chunk::get_chunk_position;
use crate::{GameProfile, GameProfileProperty, Player, WorldExtension};
use fstr::FString;
use std::collections::HashMap;
use uuid::Uuid;

pub trait EntityAppearance<W: WorldExtension> {
    fn initialize(&self, entity: &mut EntityBase<W>);

    fn destroy(&self, entity: &mut EntityBase<W>, packet: &mut DestroyEntites);

    fn enter_player_view(&self, entity: &mut EntityBase<W>, player: &mut Player<W::Player>);

    fn leave_player_view(&self, entity: &mut EntityBase<W>, player: &mut Player<W::Player>);

    fn update_position(&self, entity: &mut EntityBase<W>);

    fn update_rotation(&self, entity: &mut EntityBase<W>);
}

pub struct MobAppearance {
    pub variant: EntityVariant,
    pub metadata: EntityMetadata,
}

impl<W: WorldExtension> EntityAppearance<W> for MobAppearance {
    fn initialize(&self, _: &mut EntityBase<W>) {}

    fn destroy(&self, entity_base: &mut EntityBase<W>, packet: &mut DestroyEntites) {
        packet.entities.push(VarInt(entity_base.id))
    }

    fn enter_player_view(&self, entity: &mut EntityBase<W>, player: &mut Player<W::Player>) {
        player.write_packet(&SpawnMob {
            entity_id: entity.id,
            entity_variant: self.variant,
            x: entity.position.x,
            y: entity.position.y,
            z: entity.position.z,
            yaw: entity.yaw,
            pitch: entity.pitch,
            head_yaw: entity.yaw,
            velocity_x: entity.velocity.z,
            velocity_y: entity.velocity.y,
            velocity_z: entity.velocity.z,
            metadata: self.metadata,
        });
        player.write_packet(&EntityYawRotate {
            entity_id: entity.id,
            yaw: entity.yaw,
        })
    }

    fn leave_player_view(&self, entity: &mut EntityBase<W>, player: &mut Player<W::Player>) {
        player.write_packet(&DestroyEntites {
            entities: vec![VarInt(entity.id)],
        })
    }

    fn update_position(&self, entity: &mut EntityBase<W>) {
        let (chunk_x, chunk_z) = get_chunk_position(entity.position);
        if let Some(chunk) = entity.world_mut().chunk_grid.get_chunk_mut(chunk_x, chunk_z) {
            chunk.packet_buffer.write_packet(&EntityTeleport {
                entity_id: entity.id,
                pos_x: entity.position.x,
                pos_y: entity.position.y,
                pos_z: entity.position.z,
                yaw: entity.yaw,
                pitch: entity.pitch,
                on_ground: false,
            })
        }
    }

    fn update_rotation(&self, entity: &mut EntityBase<W>) {
        let (chunk_x, chunk_z) = get_chunk_position(entity.position);
        if let Some(chunk) = entity.world_mut().chunk_grid.get_chunk_mut(chunk_x, chunk_z) {
            chunk.packet_buffer.write_packet(&EntityRotate {
                entity_id: entity.id,
                yaw: entity.yaw,
                pitch: entity.pitch,
                on_ground: false,
            });
            chunk.packet_buffer.write_packet(&EntityYawRotate {
                entity_id: entity.id,
                yaw: entity.yaw,
            });
        }
    }
}

pub struct PlayerAppearance {
    uuid: Uuid,
    texture: &'static str,
    signature: &'static str,
}

impl PlayerAppearance {
    pub fn new(texture: &'static str, signature: &'static str) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            texture,
            signature,
        }
    }
}

impl<W: WorldExtension> EntityAppearance<W> for PlayerAppearance {
    fn initialize(&self, _: &mut EntityBase<W>) {}
    fn destroy(&self, entity: &mut EntityBase<W>, packet: &mut DestroyEntites) {
        packet.entities.push(VarInt(entity.id))
    }

    fn enter_player_view(&self, entity: &mut EntityBase<W>, player: &mut Player<W::Player>) {
        player.write_packet(&PlayerListItem {
            action: VarInt(0),
            players: &[PlayerData {
                ping: 0,
                game_mode: 0,
                profile: &GameProfile {
                    uuid: self.uuid,
                    username: FString::EMPTY,
                    properties: HashMap::from([("textures".into(), GameProfileProperty {
                        value: self.texture.into(),
                        signature: Some(self.signature.into()),
                    })]),
                },
                display_name: None,
            }],
        });
        player.write_packet(&SpawnPlayer {
            entity_id: entity.id,
            uuid: self.uuid,
            x: entity.position.x,
            y: entity.position.y,
            z: entity.position.z,
            yaw: entity.yaw,
            pitch: entity.pitch,
            current_item: 0,
            metadata: PlayerMetadata {
                layers: u8::MAX
            },
        });
        player.packet_buffer.write_packet(&EntityYawRotate {
            entity_id: entity.id,
            yaw: entity.yaw,
        });
        player.add_delayed_profile_remove(
            self.uuid
        )
    }

    fn leave_player_view(&self, entity: &mut EntityBase<W>, player: &mut Player<W::Player>) {
        player.write_packet(&DestroyEntites {
            entities: vec![VarInt(entity.id)],
        })
    }

    fn update_position(&self, entity: &mut EntityBase<W>) {
        let (chunk_x, chunk_z) = get_chunk_position(entity.position);
        if let Some(chunk) = entity.world_mut().chunk_grid.get_chunk_mut(chunk_x, chunk_z) {
            chunk.packet_buffer.write_packet(&EntityTeleport {
                entity_id: entity.id,
                pos_x: entity.position.x,
                pos_y: entity.position.y,
                pos_z: entity.position.z,
                yaw: entity.yaw,
                pitch: entity.pitch,
                on_ground: false,
            })
        }
    }

    fn update_rotation(&self, entity: &mut EntityBase<W>) {
        let (chunk_x, chunk_z) = get_chunk_position(entity.position);
        if let Some(chunk) = entity.world_mut().chunk_grid.get_chunk_mut(chunk_x, chunk_z) {
            chunk.packet_buffer.write_packet(&EntityRotate {
                entity_id: entity.id,
                yaw: entity.yaw,
                pitch: entity.pitch,
                on_ground: false,
            });
            chunk.packet_buffer.write_packet(&EntityYawRotate {
                entity_id: entity.id,
                yaw: entity.yaw,
            });
        }
    }
}
