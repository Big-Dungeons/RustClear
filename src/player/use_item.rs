use crate::block::block_parameters::Direction;
use crate::chunk::chunk_grid::ChunkGrid;
use crate::dungeon::items::get_item_stack;
use crate::network::packets::{BytesMutExt, PacketEvent};
use crate::network::protocol::block_position::BlockPosition;
use crate::network::protocol::play::clientbound::BlockChange;
use crate::network::protocol::play::serverbound::PlayerBlockPlacement;
use crate::network::protocol::var_int::VarInt;
use crate::player::inventory::{Inventory, SyncInventory};
use crate::player::{PacketReader, PlayerPacketBuffer};
use bevy::prelude::{Commands, Component, Entity, Message, MessageWriter, Query, Res};
use glam::IVec3;

#[derive(Message)]
pub struct PlayerRightClick {
    pub client: Entity,
    pub block_interact_result: Option<BlockInteractResult>,
}

#[derive(Copy, Clone)]
pub struct BlockInteractResult {
    pub position: IVec3,
    pub direction: Direction,
}

// merge with other "sent" tracking stuff
#[derive(Component)]
pub struct SentInteract(pub bool);

// add anticheat stuff
pub(super) fn handle_block_interact(
    mut packets: PacketReader<PlayerBlockPlacement>,
    mut output: MessageWriter<PlayerRightClick>,
    mut query: Query<(&mut SentInteract, &Inventory, &mut PlayerPacketBuffer)>,
    mut commands: Commands,
    chunks: Res<ChunkGrid>,
) {
    for PacketEvent { client, packet } in packets.read() {
        let (mut has_sent_already, inventory, mut packet_buffer) = query.get_mut(*client).unwrap();
        if !has_sent_already.0 {
            *has_sent_already = SentInteract(true);

            if get_item_stack(inventory.held_item()) != packet.item_stack {
                commands.trigger(SyncInventory { entity: *client })
            }

            let block_hit_result = if packet.position.y.is_negative() {
                None
            } else {
                Some(BlockInteractResult {
                    position: *packet.position,
                    direction: match packet.placed_direction {
                        0 => Direction::Down,
                        1 => Direction::Up,
                        2 => Direction::North,
                        3 => Direction::South,
                        4 => Direction::West,
                        5 => Direction::East,
                        _ => unreachable!(),
                    },
                })
            };

            // restore block
            if let Some(BlockInteractResult { direction, .. }) = block_hit_result {
                let mut position = *packet.position;

                match direction {
                    Direction::Down => position.y -= 1,
                    Direction::Up => position.y += 1,
                    Direction::North => position.z -= 1,
                    Direction::South => position.z += 1,
                    Direction::West => position.x -= 1,
                    Direction::East => position.x += 1,
                }

                let block = chunks.get_block_at(position);
                packet_buffer.write_packet(&BlockChange {
                    position: BlockPosition(position),
                    block_state: VarInt(block.get_blockstate_id() as i32),
                });
            }

            output.write(PlayerRightClick {
                client: *client,
                block_interact_result: block_hit_result,
            });
        }
    }
}

pub(super) fn clear_sent_interacts(mut query: Query<&mut SentInteract>) {
    for mut sent_interact in query.iter_mut() {
        sent_interact.0 = false
    }
}
