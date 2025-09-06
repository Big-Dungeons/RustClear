use crate::constants::particle::Particle;
use crate::dungeon::dungeon_player::DungeonPlayer;
use crate::dungeon::room::room_data::RoomType;
use crate::network::protocol::play::clientbound::{PositionLook, SoundEffect};
use crate::player::player::Player;
use crate::utils::bitset::BitSet;
use crate::world::chunk::chunk_grid::ChunkGrid;
use glam::{vec3, DVec3};
use std::f64::consts::PI;

const VALID_ETHER_WARP_BLOCK_IDS: BitSet<3> = BitSet::new(
    &[
        0, 6, 9, 11, 30, 31, 32, 36, 37, 38, 39, 40, 50, 51, 55, 59, 65, 66, 69, 76, 77, 78,
        93, 94, 104, 105, 106, 111, 115, 131, 132, 140, 141, 142, 143, 144, 149, 150, 157, 171, 175
    ]
);

enum EtherResult {
    Valid(i32, i32, i32),
    Failed,
}

pub fn etherwarp(player: &mut Player<DungeonPlayer>) {
    // temporary, but just to test,
    // since some puzzles let you teleport in them, and others don't
    if let Some(current_room) = player.current_room() {
        match current_room.data.room_type {
            RoomType::Trap | RoomType::Puzzle => return,
            _ => {}
        }
    };
    
    let mut start_pos = player.position.clone();
    start_pos.y += 1.54; // assume always sneaking

    let end_pos = {
        let yaw = player.yaw as f64;
        let pitch = player.pitch as f64;
        let rad_yaw = -yaw.to_radians() - PI;
        let rad_pitch = -pitch.to_radians();

        let f2 = -rad_pitch.cos();

        let mut pos = DVec3::new(rad_yaw.sin() * f2, rad_pitch.sin(), rad_yaw.cos() * f2).normalize();
        pos *= 61.0;
        pos + start_pos
    };

    if let EtherResult::Valid(x, y, z) = traverse_voxels(&player.world().chunk_grid, start_pos, end_pos) {
        let dungeon = &player.world().extension;
        if let Some(index) = dungeon.get_room_at(x, z) {
            let room = &dungeon.rooms[index];
            if room.data.room_type == RoomType::Puzzle { 
                return;
            }
        }
        
        
        player.world_mut().spawn_particle(
            Particle::SpellWitch,
            player.position.as_vec3(), 
            vec3(0.25, 1.0, 0.25),
            25
        );
        player.write_packet(&PositionLook {
            x: x as f64 + 0.5,
            y: y as f64 + 1.05,
            z: z as f64 + 0.5,
            yaw: 0.0,
            pitch: 0.0,
            // these flags make xyz absolute meaning they set directly
            // while keeping yaw and pitch relative (meaning it is added to players yaw)
            // since yaw and pitch provided is 0, it doesn't rotate the player causing head snapping
            flags: 24,
        });
        player.write_packet(&SoundEffect {
            sound: "mob.enderdragon.hit",
            volume: 1.0,
            pitch: 0.53968257,
            pos_x: x as f64 + 0.5,
            pos_y: y as f64 + 1.05,
            pos_z: z as f64 + 0.5,
        });
    }
}

fn traverse_voxels(chunk_grid: &ChunkGrid, start: DVec3, end: DVec3) -> EtherResult {
    let (x0, y0, z0) = (start.x, start.y, start.z);
    let (x1, y1, z1) = (end.x, end.y, end.z);

    let (mut x, mut y, mut z) = (start.x.floor() as i32, start.y.floor() as i32, start.z.floor() as i32);
    let (end_x, end_y, end_z) = (end.x.floor() as i32, end.y.floor() as i32, end.z.floor() as i32);

    let dir_x = x1 - x0;
    let dir_y = y1 - y0;
    let dir_z = z1 - z0;

    let step_x = dir_x.signum() as i32;
    let step_y = dir_y.signum() as i32;
    let step_z = dir_z.signum() as i32;

    let inv_dir_x = if dir_x != 0.0 { 1.0 / dir_x } else { f64::MAX };
    let inv_dir_y = if dir_y != 0.0 { 1.0 / dir_y } else { f64::MAX };
    let inv_dir_z = if dir_z != 0.0 { 1.0 / dir_z } else { f64::MAX };

    let t_delta_x = (inv_dir_x * step_x as f64).abs();
    let t_delta_y = (inv_dir_y * step_y as f64).abs();
    let t_delta_z = (inv_dir_z * step_z as f64).abs();

    // t_max initialization follows the "next voxel boundary" logic
    let mut t_max_x = ((x as f64 + if step_x > 0 { 1.0 } else { 0.0 } - x0) * inv_dir_x).abs();
    let mut t_max_y = ((y as f64 + if step_y > 0 { 1.0 } else { 0.0 } - y0) * inv_dir_y).abs();
    let mut t_max_z = ((z as f64 + if step_z > 0 { 1.0 } else { 0.0 } - z0) * inv_dir_z).abs();

    for _ in 0..1000 {
        // Check block at current voxel coordinates
        let current_block = chunk_grid.get_block_at(x, y, z);

        if !VALID_ETHER_WARP_BLOCK_IDS.contains((current_block.get_block_state_id() >> 4) as usize) {
            let block_up1 = chunk_grid.get_block_at(x, y + 1, z).get_block_state_id() >> 4;
            let block_up2 = chunk_grid.get_block_at(x, y + 2, z).get_block_state_id() >> 4;

            return if VALID_ETHER_WARP_BLOCK_IDS.contains(block_up1 as usize) && VALID_ETHER_WARP_BLOCK_IDS.contains(block_up2 as usize) {
                EtherResult::Valid(x, y, z)
            } else {
                EtherResult::Failed
            }
        }

        if x == end_x && y == end_y && z == end_z {
            return EtherResult::Failed;
        }

        if t_max_x <= t_max_y && t_max_x <= t_max_z {
            t_max_x += t_delta_x;
            x += step_x;
        } else if t_max_y <= t_max_z {
            t_max_y += t_delta_y;
            y += step_y;
        } else {
            t_max_z += t_delta_z;
            z += step_z;
        }
    }

    EtherResult::Failed
}