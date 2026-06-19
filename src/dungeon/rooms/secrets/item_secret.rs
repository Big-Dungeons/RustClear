use crate::core::entity::components::transform::Transform;
use crate::core::entity::components::velocity::Velocity;
use crate::core::entity::object_metadata::ObjectMetadata;
use crate::core::entity::{EntitySize, Mob};
use crate::core::player::inventory::item_stack::ItemStack;
use crate::dungeon::rooms::secrets::SecretSpawned;
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
        ));
    }
}