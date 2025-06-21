use crate::net::internal_packets::NetworkThreadMessage;
use crate::net::packets::client_bound::particles::Particles;
use crate::net::packets::client_bound::position_look::PositionLook;
use crate::net::packets::client_bound::sound_effect::SoundEffect;
use crate::net::packets::packet::SendPacket;
use crate::server::block::blocks::Blocks::Air;
use crate::server::entity::entity::Entity;
use crate::server::player::Player;
use crate::server::utils::particles::ParticleTypes::SpellWitch;
use crate::server::utils::sounds::Sounds;
use crate::server::utils::vec3f::Vec3f;
use crate::server::world::World;
use crate::utils::bitset::BitSet;
use std::f64::consts::PI;
use tokio::sync::mpsc::UnboundedSender;

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

pub fn handle_ether_warp(
    player: &Player,
    network_tx: &UnboundedSender<NetworkThreadMessage>,
    world: &World,
    entity: &Entity
) -> anyhow::Result<()> {
    let mut start_pos = entity.pos.clone();
    start_pos.y += 1.54; // assume always sneaking

    let end_pos = {
        let yaw = entity.yaw as f64;
        let pitch = entity.pitch as f64;
        let rad_yaw = -yaw.to_radians() - PI;
        let rad_pitch = -pitch.to_radians();

        let f2 = -rad_pitch.cos();

        let mut pos = Vec3f {
            x: rad_yaw.sin() * f2,
            y: rad_pitch.sin(),
            z: rad_yaw.cos() * f2,
        }.normalize();

        pos.x *= 61.0;
        pos.y *= 61.0;
        pos.z *= 61.0;

        pos + start_pos
    };

    if let EtherResult::Valid(x, y, z) = traverse_voxels(world, start_pos, end_pos) {

        if let Ok(packet) = Particles::new(
            SpellWitch,
            entity.pos,
            Vec3f::new(0.25, 1.0, 0.25),
            0.0,
            25,
            true,
            None,
        ) {
            packet.send_packet(player.client_id, network_tx)?;
        }

        PositionLook {
            x: x as f64 + 0.5,
            y: y as f64 + 1.05,
            z: z as f64 + 0.5,
            yaw: 0.0,
            pitch: 0.0,
            // these flags make xyz absolute meaning they set directly
            // while keeping yaw and pitch relative (meaning it is added to players yaw)
            // since yaw and pitch provided is 0, it doesn't rotate the player causing head snapping
            flags: 24,
        }.send_packet(player.client_id, network_tx)?;

        SoundEffect {
            sounds: Sounds::EnderDragonHit,
            volume: 1.0,
            pitch: 0.53968257,
            x: x as f64 + 0.5,
            y: y as f64 + 1.05,
            z: z as f64 + 0.5,
        }.send_packet(player.client_id, network_tx)?;
    }
    Ok(())
}

fn traverse_voxels(world: &World, start: Vec3f, end: Vec3f) -> EtherResult {
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
        let current_block = world.get_block_at(x, y, z);

        if current_block != Air {
            if VALID_ETHER_WARP_BLOCK_IDS.contains((current_block.get_block_state_id() >> 4) as usize) {
                return EtherResult::Failed;
            }
            let block_up1 = world.get_block_at(x, y + 1, z).get_block_state_id() >> 4;
            let block_up2 = world.get_block_at(x, y + 1, z).get_block_state_id() >> 4;

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