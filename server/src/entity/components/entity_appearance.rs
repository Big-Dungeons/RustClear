use crate::constants::EntityVariant;
use crate::entity::entity::MinecraftEntity;
use crate::entity::entity_metadata::{EntityMetadata, PlayerMetadata};
use crate::network::binary::var_int::VarInt;
use crate::network::packets::packet_buffer::PacketBuffer;
use crate::network::protocol::play::clientbound::{DestroyEntites, EntityRotate, EntityTeleport, EntityYawRotate, PlayerData, PlayerListItem, SpawnMob, SpawnPlayer};
use crate::{GameProfile, GameProfileProperty, Player, WorldExtension};
use bevy_ecs::component::Mutable;
use bevy_ecs::prelude::Component;
use fstr::FString;
use std::collections::HashMap;
use uuid::Uuid;

pub trait EntityAppearance<W: WorldExtension + 'static>: Component<Mutability = Mutable> + Sized {

    fn enter_player_view(&self, entity: &MinecraftEntity<W>, player: &mut Player<W::Player>);

    fn leave_player_view(&self, entity: &MinecraftEntity<W>, player: &mut Player<W::Player>);

    fn update_position(&self, entity: &MinecraftEntity<W>, packet_buffer: &mut PacketBuffer);

    fn destroy(&self, entity: &MinecraftEntity<W>, packet: &mut DestroyEntites);
}

#[derive(Component)]
pub struct MobAppearance {
    pub variant: EntityVariant,
    pub metadata: EntityMetadata,
}

impl<W: WorldExtension + 'static> EntityAppearance<W> for MobAppearance {
    fn enter_player_view(&self, entity: &MinecraftEntity<W>, player: &mut Player<W::Player>) {
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
        });
    }

    fn leave_player_view(&self, entity: &MinecraftEntity<W>, player: &mut Player<W::Player>) {
        player.write_packet(&DestroyEntites {
            entities: vec![VarInt(entity.id)],
        })
    }

    fn update_position(&self, entity: &MinecraftEntity<W>, packet_buffer: &mut PacketBuffer) {
        update_position(entity, packet_buffer)
    }

    fn destroy(&self, entity: &MinecraftEntity<W>, packet: &mut DestroyEntites) {
        packet.entities.push(VarInt(entity.id))
    }
}

#[derive(Component)]
pub struct PlayerAppearance {
    name: &'static str,
    metadata: PlayerMetadata,
    uuid: Uuid,
    texture: &'static str,
    signature: &'static str,
}

impl PlayerAppearance {
    pub fn new(
        name: &'static str,
        metadata: PlayerMetadata,
        texture: &'static str,
        signature: &'static str
    ) -> Self {
        Self {
            name,
            metadata,
            uuid: Uuid::new_v4(),
            texture,
            signature,
        }
    }
}

impl<W: WorldExtension + 'static> EntityAppearance<W> for PlayerAppearance {
    fn enter_player_view(&self, entity: &MinecraftEntity<W>, player: &mut Player<W::Player>) {
        player.write_packet(&PlayerListItem {
            action: VarInt(0),
            players: &[PlayerData {
                ping: 0,
                game_mode: 0,
                profile: &GameProfile {
                    uuid: self.uuid,
                    username: FString::new(self.name),
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
            metadata: self.metadata,
        });
        player.write_packet(&EntityRotate {
            entity_id: entity.id,
            yaw: entity.yaw,
            pitch: entity.pitch,
            on_ground: false,
        });
        player.write_packet(&EntityYawRotate {
            entity_id: entity.id,
            yaw: entity.yaw
        });
        player.add_delayed_profile_remove(self.uuid);
    }

    fn leave_player_view(&self, entity: &MinecraftEntity<W>, player: &mut Player<W::Player>) {
        player.write_packet(&DestroyEntites {
            entities: vec![VarInt(entity.id)],
        })
    }

    fn update_position(&self, entity: &MinecraftEntity<W>, packet_buffer: &mut PacketBuffer) {
        update_position(entity, packet_buffer)
    }

    fn destroy(&self, entity: &MinecraftEntity<W>, packet: &mut DestroyEntites) {
        packet.entities.push(VarInt(entity.id))
    }
}

fn update_position<W : WorldExtension>(entity: &MinecraftEntity<W>, buffer: &mut PacketBuffer) {
    buffer.write_packet(&EntityTeleport {
        entity_id: entity.id,
        pos_x: entity.position.x,
        pos_y: entity.position.y,
        pos_z: entity.position.z,
        yaw: entity.yaw,
        pitch: entity.pitch,
        on_ground: false,
    });
    buffer.write_packet(&EntityYawRotate {
        entity_id: entity.id,
        yaw: entity.yaw,
    });
}