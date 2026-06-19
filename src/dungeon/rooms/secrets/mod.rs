pub mod item_secret;
mod chest_secret;

use crate::core::block::block_rotation::Rotate;
use crate::core::entity::components::transform::Transform;
use crate::core::types::aabb::AABB;
use crate::dungeon::rooms::room_data::{RoomData, SecretSpawnCondition, SecretType};
use crate::dungeon::rooms::secrets::chest_secret::ChestSecret;
use crate::dungeon::rooms::secrets::item_secret::{ItemSecret, ItemSecretType};
use crate::dungeon::rooms::Room;
use bevy::prelude::*;
use glam::dvec3;

// marker
#[derive(Component)]
pub struct Secret;

// marker
#[derive(Component)]
pub struct SpawnOnRoomEnter;

#[derive(Component)]
pub struct SecretSpawnArea {
    pub aabb: AABB,
}

#[derive(Component)]
struct SecretSpawned;

fn on_player_enter_area(
    player_query: Query<&Transform, Changed<Transform>>,
    secret_query: Query<(Entity, &SecretSpawnArea), Without<SecretSpawned>>,
    mut commands: Commands,
) {
    // it would obviously be more optimal to query spatial, but for now it's fine
    for transform in player_query.iter() {
        for (entity, SecretSpawnArea { aabb }) in secret_query.iter() {
            if aabb.contains(transform.position) {
                commands
                    .entity(entity)
                    .insert(SecretSpawned);
            }
        }
    }
}

pub struct DungeonSecretsPlugin;

impl Plugin for DungeonSecretsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(item_secret::on_secret_spawn)
            .add_observer(chest_secret::on_secret_spawn)
            .add_systems(PostStartup, load_secrets)
            .add_systems(Update, on_player_enter_area);
    }
}

pub fn load_secrets(mut room_query: Query<(&mut Room, &RoomData)>, mut commands: Commands) {
    for (mut room, data) in room_query.iter_mut() {
        for secret in data.secrets.iter() {
            let mut secret_entity = commands.spawn(
                Secret,
            );

            let world_position = room.relative_to_world(secret.position);

            match secret.secret_type {
                SecretType::Chest { rotation } => {
                    let rotation = rotation.rotate(room.rotation);
                    secret_entity.insert(ChestSecret {
                        spawn_location: world_position,
                        rotation,
                    });
                }
                SecretType::Item => {
                    secret_entity.insert(ItemSecret {
                        spawn_location: world_position,
                        item_type: ItemSecretType::SpiritLeap,
                    });
                },
            }

            match secret.spawn_condition {
                SecretSpawnCondition::EnterArea { width, height } => {
                    let width = width as f64 / 2.0;
                    let height = height as f64 / 2.0;
                    let position = world_position.as_dvec3() + dvec3(0.5, 0.0, 0.5);

                    let aabb = AABB::new(
                        position - dvec3(width, height, width),
                        position + dvec3(width, height, width),
                    );

                    secret_entity.insert(SecretSpawnArea {
                        aabb,
                    });
                }
                SecretSpawnCondition::EnterRoom => {}
            }

            room.secrets.insert(secret_entity.id());
        }
    }
}