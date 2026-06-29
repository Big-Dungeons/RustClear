pub mod item_secret;
pub mod chest_secret;
pub mod essence;

use crate::core::block::block_rotation::Rotate;
use crate::core::entity::components::transform::Transform;
use crate::core::run_every_ticks;
use crate::core::types::aabb::AABB;
use crate::dungeon::rooms::room_data::{RoomData, SecretSpawnCondition, SecretType};
use crate::dungeon::rooms::room_enter::RoomEntered;
use crate::dungeon::rooms::secrets::chest_secret::{ChestSecret, ChestSecretType};
use crate::dungeon::rooms::secrets::essence::EssenceSecret;
use crate::dungeon::rooms::secrets::item_secret::{ItemSecret, ItemSecretType};
use crate::dungeon::rooms::Room;
use bevy::prelude::*;
use glam::dvec3;

#[derive(Component)]
pub struct Secret {
    pub collected: bool,
}

// marker
#[derive(Component)]
pub struct SpawnOnRoomEnter;

#[derive(Component)]
pub struct SecretSpawnArea {
    pub aabb: AABB,
}

#[derive(Component)]
struct SecretSpawned;

fn on_player_enter_room(
    event: On<Insert, RoomEntered>,
    room_query: Query<&Room>,
    secret_query: Query<Entity, (With<SpawnOnRoomEnter>, Without<SecretSpawned>)>,
    mut commands: Commands,
) {
    let room = room_query
        .get(event.entity)
        .unwrap();

    for entity in secret_query.iter_many(&room.secrets) {
        commands
            .entity(entity)
            .insert(SecretSpawned);
    }
}

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

#[derive(EntityEvent)]
pub struct CollectSecretEvent {
    pub entity: Entity
}

pub fn update_room_secrets(
    event: On<CollectSecretEvent>,
    mut secret_query: Query<(&mut Secret, Option<&ChildOf>)>,
    mut room_query: Query<&mut Room>,
) {
    let (mut secret, child_of) = secret_query
        .get_mut(event.entity)
        .expect("Used collect secret event on non-secret entity");

    if secret.collected {
        return;
    }

    secret.collected = true;
    if let Some(child_of) = child_of {
        let mut room = room_query
            .get_mut(child_of.parent())
            .expect("secret's parent wasn't a room");

        room.found_secrets += 1;
    }
}

pub fn load_secrets(mut room_query: Query<(Entity, &mut Room, &RoomData)>, mut commands: Commands) {
    for (entity, mut room, data) in room_query.iter_mut() {
        for secret in data.secrets.iter() {
            let mut secret_entity = commands.spawn((
                Secret {
                    collected: false
                },
                ChildOf(entity)
            ));

            let world_position = room.relative_to_world(secret.position);

            match secret.secret {
                SecretType::Chest { rotation } => {
                    let rotation = rotation.rotate(room.rotation);
                    secret_entity.insert(ChestSecret {
                        spawn_position: world_position,
                        // todo: rng choose, and if lever related set locked to true
                        chest_type: ChestSecretType::Blessing {
                            locked: false
                        },
                        rotation,
                    });
                }
                SecretType::Item => {
                    secret_entity.insert(ItemSecret {
                        spawn_position: world_position,
                        // todo: rng choose
                        item_type: ItemSecretType::SpiritLeap,
                    });
                },
                SecretType::Essence { rotation } => {
                    // todo: rotation
                    secret_entity.insert(EssenceSecret {
                        spawn_position: world_position,
                    });
                }
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
                SecretSpawnCondition::EnterRoom => {
                    secret_entity.insert(SpawnOnRoomEnter);
                }
            }

            room.secrets.insert(secret_entity.id());
        }
    }
}

pub struct DungeonSecretsPlugin;

impl Plugin for DungeonSecretsPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_observer(item_secret::on_secret_spawn)
            .add_observer(chest_secret::on_secret_spawn)
            .add_observer(chest_secret::on_interact)
            .add_observer(essence::on_secret_spawn)
            .add_observer(essence::on_interact)
            .add_observer(on_player_enter_room)
            .add_observer(update_room_secrets)
            .add_systems(PostStartup, load_secrets)
            .add_systems(Update, (
                on_player_enter_area
                    // .run_if(in_state(DungeonState::Started))
                    .run_if(run_every_ticks::<20>),

                item_secret::pickup_item_secret,
                essence::update_essence_mob,
            ));
    }
}