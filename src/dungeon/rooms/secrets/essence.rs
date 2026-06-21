use crate::core::block::block_entity::{BlockEntity, BlockEntityType};
use crate::core::block::block_parameters::Direction;
use crate::core::block::Block;
use crate::core::chunk::chunk_grid::ChunkGrid;
use crate::core::entity::components::despawn_after::DespawnAfter;
use crate::core::entity::components::equipment::Equipment;
use crate::core::entity::components::transform::Transform;
use crate::core::entity::entity_metadata::ArmorStandMetadata;
use crate::core::entity::Mob;
use crate::core::player::inventory::item_stack::ItemStack;
use crate::core::player::PlayerSkin;
use crate::dungeon::player::block_interaction::{BlockInteractable, BlockInteractionEvent};
use crate::dungeon::rooms::secrets::{CollectSecretEvent, SecretSpawned};
use bevy::prelude::*;
use glam::IVec3;
use uuid::Uuid;

// 0..=15
// pub struct EssenceRotation {}

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
                    rotation: 0,
                    skull_type: 3,
                    uuid: Default::default(),
                    skin: PlayerSkin {
                        texture: ESSENCE_TEXTURE.to_string(),
                        _signature: None,
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
                Uuid::max(),
                PlayerSkin {
                    texture: ESSENCE_TEXTURE.to_string(),
                    _signature: None,
                }
            );

        commands.spawn((
            DespawnAfter {
                ticks: 20
            },
            Mob::new(ArmorStandMetadata {
                flags: 0x20
            }),
            transform,
            Equipment::new().helmet(skull),
            EssenceSpinningThing,
        ));

        chunks.set_block_at(Block::Air, block.position);
    }
}

#[derive(Component)]
pub(super) struct EssenceSpinningThing;

pub(super) fn update_essence_mob(mut query: Query<&mut Transform, With<EssenceSpinningThing>>) {
    for mut transform in query.iter_mut() {
        transform.position.y += 0.04;
        transform.yaw += 15.0;
    }
}
