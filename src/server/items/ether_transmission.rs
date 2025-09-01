use crate::net::protocol::play::clientbound::PositionLook;
use crate::server::player::player::Player;
use glam::DVec3;
use std::f64::consts::PI;

pub fn handle_teleport(player: &mut Player) {
    let mut start_pos = player.position.clone();
    start_pos.y += 1.54; // assume always sneaking

    let end_pos = {
        let yaw = player.yaw as f64;
        let pitch = player.pitch as f64;
        let rad_yaw = -yaw.to_radians() - PI;
        let rad_pitch = -pitch.to_radians();

        let f2 = -rad_pitch.cos();
        let mut pos = DVec3::new(rad_yaw.sin() * f2, rad_pitch.sin(), rad_yaw.cos() * f2).normalize();
        pos.x *= 10.0;
        pos.y *= 10.0;
        pos.z *= 10.0;

        pos + start_pos
    };
    
    player.write_packet(&PositionLook {
        x: end_pos.x + 0.5,
        y: end_pos.y + 1.05,
        z: end_pos.z + 0.5,
        yaw: 0.0,
        pitch: 0.0,
        // flags make x y z absolute, and yaw/pitch relative,
        // since yaw and pitch is 0, it doesn't rotate the player
        flags: 24,
    });
}
