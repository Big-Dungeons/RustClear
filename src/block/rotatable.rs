use crate::types::direction::Direction;
use glam::IVec3;

pub trait Rotatable {
    fn rotate(&self, direction: Direction) -> Self;
}

impl Rotatable for f32 {
    fn rotate(&self, dir: Direction) -> f32 {
        let offset = match dir {
            Direction::North => 0.0,
            Direction::East  => 90.0,
            Direction::South => 180.0,
            Direction::West  => 270.0,
            Direction::Up | Direction::Down => 0.0,
        };
        (self + offset) % 360.0
    }
}

impl Rotatable for IVec3 {
    fn rotate(&self, direction: Direction) -> Self {
        match direction {
            Direction::North => Self { x: self.x, y: self.y, z: self.z },
            Direction::East => Self { x: -self.z, y: self.y, z: self.x },
            Direction::South => Self { x: -self.x, y: self.y, z: -self.z },
            Direction::West => Self { x: self.z, y: self.y, z: -self.x },
            _ => Self { x: self.x, y: self.y, z: self.z },
        }
    }
}