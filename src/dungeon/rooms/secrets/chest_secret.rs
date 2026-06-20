use crate::core::block::block_parameters::Direction;
use crate::core::block::block_rotation::{Rotate, Rotation};
use crate::core::block::Block;
use crate::core::chunk::chunk_grid::ChunkGrid;
use crate::dungeon::player::block_interaction::{BlockInteractable, BlockInteractionEvent};
use crate::dungeon::rooms::secrets::{CollectSecretEvent, Secret, SecretSpawned};
use bevy::prelude::*;
use glam::IVec3;
use crate::core::network::packets::BytesMutExt;
use crate::core::network::protocol::play::clientbound::{BlockAction, Chat};
use crate::core::player::PlayerPacketBuffer;

pub enum ChestSecretType {
    Blessing {
        locked: bool,
    },
    Item
}

#[derive(Component)]
pub struct ChestSecret {
    pub chest_type: ChestSecretType,
    pub spawn_location: IVec3,
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
        chunks.set_block_at(Block::Chest { direction }, secret.spawn_location);

        commands.spawn((
            ChestSecretBlock,
            BlockInteractable {
                position: secret.spawn_location
            },
            ChildOf(event.entity),
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
                // That chest is locked!
                return;
            }
            
            // spawn the blessing for like some amount of ticks
        } else if secret.collected {
            packet_buffer.write_packet(&Chat::new("§cThis chest has already been searched!"));
        }

        if let Some(chunk) = chunks.get_mut_from_world(block.position) {
            chunk.packet_buffer.write_packet(&BlockAction::new(
                block.position,
                1,
                1,
                Block::Chest { direction: Direction::North }) // only uses block_id
            )
        }

        commands.trigger(CollectSecretEvent {
            entity: child_of.parent(),
        })
    }
}