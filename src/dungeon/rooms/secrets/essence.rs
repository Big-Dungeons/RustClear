use crate::core::block::block_entity::{BlockEntity, BlockEntityType, SkullRotation, SkullType};
use crate::core::block::block_parameters::Direction;
use crate::core::block::Block;
use crate::core::chunk::chunk_grid::ChunkGrid;
use crate::core::entity::components::despawn_after::DespawnAfter;
use crate::core::entity::components::equipment::Equipment;
use crate::core::entity::components::transform::Transform;
use crate::core::entity::entity_metadata::ArmorStandMetadata;
use crate::core::entity::{Mob, SpawnedOnTick};
use crate::core::player::inventory::item_stack::ItemStack;
use crate::core::player::particles::ParticleEvent;
use crate::core::player::sound::LocalSound;
use crate::core::player::PlayerSkin;
use crate::core::types::particles::Particle;
use crate::core::types::sound::Sound;
use crate::core::ServerTick;
use crate::dungeon::player::block_interaction::{BlockInteractable, BlockInteractionEvent};
use crate::dungeon::rooms::secrets::{CollectSecretEvent, SecretSpawned};
use bevy::prelude::*;
use glam::{dvec3, IVec3, Vec3};
use uuid::Uuid;

const ESSENCE_UUID: Uuid = Uuid::from_u128(1);
const ESSENCE_TEXTURE: &str = "ewogICJ0aW1lc3RhbXAiIDogMTYwMzYxMDQ0MzU4MywKICAicHJvZmlsZUlkIiA6ICIzM2ViZDMyYmIzMzk0YWQ5YWM2NzBjOTZjNTQ5YmE3ZSIsCiAgInByb2ZpbGVOYW1lIiA6ICJEYW5ub0JhbmFubm9YRCIsCiAgInNpZ25hdHVyZVJlcXVpcmVkIiA6IHRydWUsCiAgInRleHR1cmVzIiA6IHsKICAgICJTS0lOIiA6IHsKICAgICAgInVybCIgOiAiaHR0cDovL3RleHR1cmVzLm1pbmVjcmFmdC5uZXQvdGV4dHVyZS9lNDllYzdkODJiMTQxNWFjYWUyMDU5Zjc4Y2QxZDE3NTRiOWRlOWIxOGNhNTlmNjA5MDI0YzRhZjg0M2Q0ZDI0IgogICAgfQogIH0KfQ==";

#[derive(Component)]
pub struct EssenceSecret {
    pub spawn_position: IVec3,
}

pub(super) fn on_secret_spawn(
    event: On<Insert, SecretSpawned>,
    query: Query<&EssenceSecret>,

    mut chunks: ResMut<ChunkGrid>,
    mut commands: Commands,
) {
    if let Ok(secret) = query.get(event.entity) {
        let block = Block::Skull { direction: Direction::Up, no_drop: false };
        chunks.set_block_at(block, secret.spawn_position);

        commands.spawn((
            ChildOf(event.entity),
            EssenceBlock,
            BlockInteractable {
                position: secret.spawn_position
            },
            BlockEntity::new(
                secret.spawn_position,
                BlockEntityType::Skull {
                    rotation: SkullRotation::default(),
                    skull_type: SkullType::PlayerHead {
                        uuid: ESSENCE_UUID,
                        skin: PlayerSkin::new(ESSENCE_TEXTURE.to_string()),
                    },
                }
            ),
        ));
    }
}

#[derive(Component)]
pub(super) struct EssenceBlock;

pub(super) fn on_interact(
    event: On<BlockInteractionEvent>,
    block_query: Query<(&BlockInteractable, &ChildOf), With<EssenceBlock>>,
    mut chunks: ResMut<ChunkGrid>,
    mut commands: Commands,
) {
    if let Ok((block, child_of)) = block_query.get(event.entity) {
        commands.trigger(CollectSecretEvent {
            entity: child_of.parent(),
        });

        commands
            .entity(event.entity)
            .despawn();

        let mut transform = Transform::new_centered(block.position);
        transform.position.y -= 1.4;

        let skull = ItemStack::new()
            .item_id(397)
            .metadata(3)
            .skull_owner(
                ESSENCE_UUID,
                PlayerSkin {
                    texture: ESSENCE_TEXTURE.to_string(),
                    _signature: None,
                }
            );

        commands.spawn((
            DespawnAfter {
                ticks: 20
            },
            Mob::new(ArmorStandMetadata::new().invisible(true)),
            transform,
            Equipment::new().helmet(skull),
            EssenceSpinningThing,
        ));

        chunks.set_block_at(Block::Air, block.position);
    }
}

#[derive(Component)]
pub(super) struct EssenceSpinningThing;

pub(super) fn update_essence_mob(
    mut query: Query<(&mut Transform, &SpawnedOnTick), With<EssenceSpinningThing>>,
    mut sounds: MessageWriter<LocalSound>,
    mut particles: MessageWriter<ParticleEvent>,
    server_tick: Res<ServerTick>,
) {
    for (mut transform, spawned_on_tick) in query.iter_mut() {
        transform.position.y += 0.04;
        transform.yaw += 15.0;

        let ticks = server_tick.elapsed_since(**spawned_on_tick);

        if ticks % 5 == 0 {
            let position = transform.position + dvec3(0.0, 1.5, 0.0);
            sounds.write(LocalSound {
                sound: Sound::NoteHarp,
                volume: 1.0,
                pitch: 0.8 + ((ticks / 5) as f32 * 0.1),
                position,
            });
            particles.write(ParticleEvent {
                particle: Particle::Cloud,
                position: position.as_vec3(),
                offset: Vec3::ZERO,
                speed: 0.06,
                count: 5,
            });
        }
        if ticks == 20 {
            let sound = LocalSound {
                sound: Sound::RandomOrb,
                volume: 1.0,
                pitch: 1.5,
                position: transform.position,
            };
            // hypixel writes it twice
            sounds.write(sound);
            sounds.write(sound);
        }
    }
}
