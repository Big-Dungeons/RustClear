use crate::dungeon::dungeon::Dungeon;
use crate::dungeon::dungeon_player::DungeonPlayer;
use crate::dungeon::items::dungeon_items::DungeonItem;
use glam::{dvec3, IVec3};
use server::block::blocks::Blocks;
use server::network::protocol::play::serverbound::PlayerDiggingAction;
use server::types::aabb::AABB;
use server::{Player, World};

// if returns false should
pub fn dungeon_breaker_dig(
    player: &mut Player<DungeonPlayer>,
    world: &mut World<Dungeon>,
    position: IVec3,
    action: &PlayerDiggingAction,
) -> bool {

    if !matches!(action, PlayerDiggingAction::StartDestroyBlock) {
        return false
    }

    if !world.has_started() || player.pickaxe_charges == 0 {
        return false
    }

    if matches!(player.get_held_item(), Some(DungeonItem::Pickaxe)) {
        let Some((room_rc, _)) = &player.extension.current_room else {
            // not in room
            return false
        };

        let chunk_grid = &mut world.chunk_grid;
        let room = room_rc.borrow();

        let block_aabb = AABB::new(
            position.as_dvec3() + dvec3(-0.75, -0.75, -0.75),
            position.as_dvec3() + dvec3(1.5, 1.5, 1.5),
        );

        // check if room doesn't allow, check if overlaps with secrets

        let mut volume_inside = 0.0;

        for bounds in room.room_bounds.iter() {
            volume_inside += block_aabb.intersection_volume(&bounds.aabb);
        }

        if block_aabb.volume() == volume_inside {
            let previous = chunk_grid.get_block_at(position.x, position.y, position.z);
            if previous == Blocks::Bedrock {
                return false;
            }
            player.extension.broken_blocks.push((position, previous, 200));

            chunk_grid.set_block_at(Blocks::Air, position.x, position.y, position.z);
            player.extension.pickaxe_charges -= 1;
            return true;
        }
    }

    false
}