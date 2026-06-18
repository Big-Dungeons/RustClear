use glam::{DVec3, I16Vec3, IVec3};

// used in entity packets and such

pub fn packed_position(position: DVec3) -> IVec3 {
    IVec3 {
        x: (position.x * 32.0).floor() as i32,
        y: (position.y * 32.0).floor() as i32,
        z: (position.z * 32.0).floor() as i32,
    }
}

pub fn packed_rotation(rot: f32) -> i8 {
    (rot * 256.0 / 360.0) as i32 as i8
}

pub fn packed_velocity(velocity: DVec3) -> I16Vec3 {
    I16Vec3 {
        x: 0,
        y: 0,
        z: 0,
    }
}