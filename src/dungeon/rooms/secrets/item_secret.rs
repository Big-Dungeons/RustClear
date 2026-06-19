use crate::core::entity::components::transform::Transform;
use crate::core::entity::components::velocity::Velocity;
use crate::core::entity::object_metadata::ObjectMetadata;
use crate::core::entity::{BevyEntityExt, EntitySize, Mob};
use crate::core::network::packets::BytesMutExt;
use crate::core::network::protocol::play::clientbound::CollectItem;
use crate::core::network::protocol::var_int::VarInt;
use crate::core::player::inventory::item_stack::ItemStack;
use crate::core::player::{Player, PlayerPacketBuffer};
use crate::core::types::aabb::AABB;
use crate::dungeon::rooms::secrets::{CollectSecretEvent, SecretSpawned};
use bevy::prelude::*;
use glam::{dvec3, IVec3};

// todo: add rest
pub enum ItemSecretType {
    SpiritLeap,
}

impl ItemSecretType {
    pub fn get_item(&self) -> ItemStack {
        match self {
            Self::SpiritLeap => ItemStack::new().item_id(368).enchant(1, 1),
        }
    }
}

#[derive(Component)]
pub struct ItemSecret {
    pub spawn_location: IVec3,
    pub item_type: ItemSecretType,
}

// this is the one inside the world
#[derive(Component)]
pub(super) struct ItemPickupRadius {
    width: f64,
    height: f64,
}

pub(super) fn on_secret_spawn(
    event: On<Insert, SecretSpawned>,
    query: Query<&ItemSecret>,
    mut commands: Commands
) {
    if let Ok(secret) = query.get(event.entity) {
        commands.spawn((
            Mob::new_object(ObjectMetadata::DroppedItem {
                item: secret.item_type.get_item()
            }),
            EntitySize {
                half_width: 0.125,
                height: 2.5,
            },
            Transform::new_centered(secret.spawn_location),
            Velocity(dvec3(0.0, 0.1, 0.0)),
            ItemPickupRadius {
                // might not be correct
                width: 3.0,
                height: 3.0
            },
            ChildOf(event.entity),
        ));
    }
}

pub(super) fn pickup_item_secret(
    secret_query: Query<(Entity, &ItemPickupRadius, &Transform, &ChildOf)>,
    mut player_query: Query<
        (
            Entity,
            &Transform,
            &EntitySize,
            &mut PlayerPacketBuffer
        ),
        With<Player>
    >,
    mut commands: Commands,
) {
    for (entity, secret, transform, child_of) in secret_query.iter() {
        let secret_aabb = AABB::new(
            transform.position - dvec3(secret.width, secret.height, secret.width),
            transform.position + dvec3(secret.width, secret.height, secret.width),
        );

        for (player_entity, player_transform, size, mut packet_buffer) in player_query.iter_mut() {
            let player_aabb = size.aabb(player_transform.position);

            if player_aabb.intersects(&secret_aabb) {
                packet_buffer.write_packet(&CollectItem {
                    item_entity_id: VarInt(entity.mc_id()),
                    player_entity_id: VarInt(player_entity.mc_id()),
                });

                commands
                    .entity(entity)
                    .despawn();

                commands.trigger(CollectSecretEvent { entity: child_of.parent() });
                break;
            }
        }
    }
}