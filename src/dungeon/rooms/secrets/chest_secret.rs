use crate::core::block::block_parameters::Direction;
use crate::core::block::block_rotation::{Rotate, Rotation};
use crate::core::block::Block;
use crate::core::chunk::chunk_grid::ChunkGrid;
use crate::core::entity::components::despawn_after::DespawnAfter;
use crate::core::entity::components::equipment::Equipment;
use crate::core::entity::components::transform::Transform;
use crate::core::entity::entity_metadata::ArmorStandMetadata;
use crate::core::entity::Mob;
use crate::core::network::packets::BytesMutExt;
use crate::core::network::protocol::play::clientbound::{BlockAction, Chat, SoundEffect};
use crate::core::player::inventory::item_stack::ItemStack;
use crate::core::player::{PlayerPacketBuffer, PlayerSkin};
use crate::core::types::sound::Sound;
use crate::dungeon::player::block_interaction::{BlockInteractable, BlockInteractionEvent};
use crate::dungeon::rooms::secrets::{essence, CollectSecretEvent, Secret, SecretSpawned};
use bevy::prelude::*;
use glam::{DVec3, IVec3};
use uuid::Uuid;

const BLESSING_UUID: Uuid = Uuid::from_u128(2);
const BLESSING_TEXTURE: &str = "eyJ0ZXh0dXJlcyI6eyJTS0lOIjp7InVybCI6Imh0dHA6Ly90ZXh0dXJlcy5taW5lY3JhZnQubmV0L3RleHR1cmUvZTkzZTIwNjg2MTc4NzJjNTQyZWNkYTFkMjdkZjRlY2U5MWM2OTk5MDdiZjMyN2M0ZGRiODUzMDk0MTJkMzkzOSJ9fX0=";

pub enum ChestSecretType {
    Blessing { locked: bool },
    Item,
}

#[derive(Component)]
pub struct ChestSecret {
    pub spawn_position: IVec3,
    pub chest_type: ChestSecretType,
    pub rotation: Rotation,
}

pub(super) fn on_secret_spawn(
    event: On<Insert, SecretSpawned>,
    query: Query<&ChestSecret>,
    mut chunks: ResMut<ChunkGrid>,
    mut commands: Commands,
) {
    if let Ok(secret) = query.get(event.entity) {
        let direction = Direction::North.rotate(secret.rotation);
        chunks.set_block_at(Block::Chest { direction }, secret.spawn_position);

        commands.spawn((
            ChildOf(event.entity),
            ChestSecretBlock,
            BlockInteractable {
                position: secret.spawn_position,
            },
        ));
    }
}

#[derive(Component)]
pub(super) struct ChestSecretBlock;

pub(super) fn on_interact(
    event: On<BlockInteractionEvent>,
    block_query: Query<(&BlockInteractable, &ChildOf), With<ChestSecretBlock>>,
    secret_query: Query<(&Secret, &ChestSecret)>,

    mut player_query: Query<&mut PlayerPacketBuffer>,
    mut chunks: ResMut<ChunkGrid>,
    mut commands: Commands,
) {
    if let Ok((block, child_of)) = block_query.get(event.entity) {
        let (secret, chest) = secret_query
            .get(child_of.parent())
            .unwrap();

        let mut packet_buffer = player_query
            .get_mut(event.player)
            .unwrap();

        if let ChestSecretType::Blessing { locked } = chest.chest_type {
            if locked {
                packet_buffer.write_packet(&Chat::new("§cThat chest is locked!"));
                return;
            }

            let mut transform = Transform::new_centered(block.position);
            transform.position.y -= 1.0;

            if !secret.collected {
                let player_head = ItemStack::new()
                    .item_id(397)
                    .metadata(3)
                    .skull_owner(
                        BLESSING_UUID,
                        PlayerSkin {
                            texture: BLESSING_TEXTURE.to_string(),
                            _signature: None,
                        },
                    );
                commands.spawn((
                    DespawnAfter { ticks: 20 },
                    Mob::new(ArmorStandMetadata { flags: 0x20 }),
                    transform,
                    Equipment::new().helmet(player_head),
                    essence::EssenceSpinningThing,
                ));
            }
        } else if secret.collected {
            // this message only appears on non blessing chest
            packet_buffer.write_packet(&Chat::new("§cThis chest has already been searched!"));
        }

        if !secret.collected {
            packet_buffer.write_packet(&SoundEffect::new(
                Sound::RandomChestOpen,
                block.position.as_dvec3() + DVec3::splat(0.5),
                1.0,
                0.975,
            ));
        }

        if let Some(chunk) = chunks.get_mut_from_world(block.position) {
            chunk.packet_buffer.write_packet(&BlockAction::new(
                block.position,
                1,
                1,
                Block::Chest {
                    direction: Direction::North,
                }, // only uses block_id
            ))
        }

        commands.trigger(CollectSecretEvent {
            entity: child_of.parent(),
        })
    }
}