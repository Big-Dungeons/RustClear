use crate::types::direction::Direction3D;
use glam::IVec3;

pub trait Rotatable {
    fn rotate(&self, direction: Direction3D) -> Self;
}

impl Rotatable for f32 {
    fn rotate(&self, dir: Direction3D) -> f32 {
        let offset = match dir {
            Direction3D::North => 0.0,
            Direction3D::East  => 90.0,
            Direction3D::South => 180.0,
            Direction3D::West  => 270.0,
            Direction3D::Up | Direction3D::Down => 0.0,
        };
        (self + offset) % 360.0
    }
}

impl Rotatable for IVec3 {
    fn rotate(&self, direction: Direction3D) -> Self {
        match direction {
            Direction3D::North => Self { x: self.x, y: self.y, z: self.z },
            Direction3D::East => Self { x: -self.z, y: self.y, z: self.x },
            Direction3D::South => Self { x: -self.x, y: self.y, z: -self.z },
            Direction3D::West => Self { x: self.z, y: self.y, z: -self.x },
            _ => Self { x: self.x, y: self.y, z: self.z },
        }
    }
}