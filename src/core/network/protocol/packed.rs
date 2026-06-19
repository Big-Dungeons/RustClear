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

const MOTION_CLAMP: f64 = 3.9;

pub fn packed_velocity(velocity: DVec3) -> I16Vec3 {
    I16Vec3 {
        x: (velocity.x.clamp(-MOTION_CLAMP, MOTION_CLAMP) * 8000.0) as i16,
        y: (velocity.y.clamp(-MOTION_CLAMP, MOTION_CLAMP) * 8000.0) as i16,
        z: (velocity.z.clamp(-MOTION_CLAMP, MOTION_CLAMP) * 8000.0) as i16,
    }
}