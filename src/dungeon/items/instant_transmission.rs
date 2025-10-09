use crate::dungeon::dungeon_player::DungeonPlayer;
use crate::dungeon::items::etherwarp::VALID_ETHER_WARP_BLOCK_IDS;
use crate::dungeon::room::room_data::RoomType;
use crate::network::protocol::play::clientbound::{PositionLook, Relative};
use crate::player::player::Player;
use crate::world::chunk::chunk_grid::ChunkGrid;
use glam::{dvec3, DVec3, IVec3};

// it appears hypixel uses a generic stepping algorithm for ray-casting
pub fn instant_transmission(
    player: &mut Player<DungeonPlayer>,
    distance: f64,
) {
    if let Some(room_rc) = player.get_current_room() {
        match room_rc.borrow().data.room_type {
            RoomType::Trap | RoomType::Puzzle => return,
            _ => {}
        }
    };

    let chunk_grid = &player.world().chunk_grid;

    let mut start = player.position;
    start.y += 1.62;

    const STEP_LEN: f64 = 0.5;
    let step_amount = (distance / STEP_LEN).ceil() as usize;
    let direction_vec = player.rotation_vec().normalize().as_dvec3() * STEP_LEN;

    let mut current_position = start;
    let mut last_checked: IVec3;
    let mut current_block: Option<IVec3> = None;
    let mut block_in_way: bool = false;

    // this small section acts as the first iteration.
    // because even if it is invalid teleport position,
    // it still sets last checked, and won't stop the ray-cast.
    last_checked = mc_floor_vec3(current_position);
    current_position += direction_vec;

    if is_valid(chunk_grid, last_checked) {
        current_block = Some(last_checked);
    }

    for _ in 1..step_amount {
        let block_pos = mc_floor_vec3(current_position);
        current_position += direction_vec;

        if block_pos == last_checked {
            continue;
        }
        if !is_valid(chunk_grid, block_pos) {
            block_in_way = true;
            break;
        }

        last_checked = block_pos;
        current_block = Some(block_pos);
    }

    // todo: sounds
    if let Some(position) = current_block {
        let position = dvec3(position.x as f64 + 0.5, position.y as f64, position.z as f64 + 0.5);
        let dungeon = &player.world().extension;

        if let Some((room_rc, _)) = dungeon.get_room(&position, player.collision_aabb_at(&position)) {
            if room_rc.borrow().data.room_type == RoomType::Puzzle {
                return;
            }
        }
        player.write_packet(&PositionLook {
            x: position.x,
            y: position.y,
            z: position.z,
            yaw: 0.0,
            pitch: 0.0,
            flags: Relative::Yaw | Relative::Pitch,
        })
    }
    if block_in_way {
        // play block in the way sound
    }
}

fn is_valid(chunk_grid: &ChunkGrid, position: IVec3) -> bool {
    let block1 = chunk_grid.get_block_at(position.x, position.y, position.z).get_block_state_id() >> 4;
    let block2 = chunk_grid.get_block_at(position.x, position.y + 1, position.z).get_block_state_id() >> 4;
    VALID_ETHER_WARP_BLOCK_IDS.contains(block1 as usize) && VALID_ETHER_WARP_BLOCK_IDS.contains(block2 as usize)
}

fn mc_floor(value: f64) -> i32 {
    let i = value as i32;
    if value < i as f64 { i - 1 } else { i }
}

fn mc_floor_vec3(v: DVec3) -> IVec3 {
    IVec3::new(mc_floor(v.x), mc_floor(v.y), mc_floor(v.z))
}
