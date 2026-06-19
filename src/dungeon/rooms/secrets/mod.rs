pub mod item_secret;

use crate::core::entity::components::transform::Transform;
use crate::core::types::aabb::AABB;
use bevy::prelude::*;

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
            .add_systems(Update, on_player_enter_area);
    }
}