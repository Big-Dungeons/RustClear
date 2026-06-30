use crate::core::block::block_metadata::BlockFieldMetadata;
use crate::core::block::block_parameters::{BlockAxis, ButtonDirection, Direction, HorizontalDirection, LeverOrientation, RailShape, StairDirection, TorchDirection, TrapdoorDirection, VineMetadata};
use glam::{DVec3, IVec3};
use serde::Deserialize;

#[derive(Deserialize, Debug, Copy, Clone)]
pub enum Rotation {
    None,
    Clockwise90,
    Clockwise180,
    CounterClockwise90,
}

impl Rotation {
    pub fn inverse(&self) -> Rotation {
        match self {
            Rotation::None => Rotation::None,
            Rotation::Clockwise90 => Rotation::CounterClockwise90,
            Rotation::Clockwise180 => Rotation::Clockwise180,
            Rotation::CounterClockwise90 => Rotation::Clockwise90,
        }
    }
}

pub trait Rotate {
    fn rotate(&self, rotation: Rotation) -> Self;
}

impl Rotate for Rotation {
    fn rotate(&self, rotation: Rotation) -> Self {
        match rotation {
            Rotation::None => *self,
            Rotation::Clockwise90 => match self {
                Rotation::None => Rotation::Clockwise90,
                Rotation::Clockwise90 => Rotation::Clockwise180,
                Rotation::Clockwise180 => Rotation::CounterClockwise90,
                Rotation::CounterClockwise90 => Rotation::None,
            },
            Rotation::Clockwise180 => match self {
                Rotation::None => Rotation::Clockwise180,
                Rotation::Clockwise90 => Rotation::CounterClockwise90,
                Rotation::Clockwise180 => Rotation::None,
                Rotation::CounterClockwise90 => Rotation::Clockwise90,
            },
            Rotation::CounterClockwise90 => match self {
                Rotation::None => Rotation::CounterClockwise90,
                Rotation::Clockwise90 => Rotation::None,
                Rotation::Clockwise180 => Rotation::Clockwise90,
                Rotation::CounterClockwise90 => Rotation::Clockwise180,
            },
        }
    }
}

impl Rotate for f32 {
    fn rotate(&self, rotation: Rotation) -> f32 {
        let offset = match rotation {
            Rotation::None => 0.0,
            Rotation::Clockwise90 => 90.0,
            Rotation::Clockwise180 => 180.0,
            Rotation::CounterClockwise90 => 270.0,
        };
        (self + offset).rem_euclid(360.0)
    }
}

impl Rotate for IVec3 {
    fn rotate(&self, rotation: Rotation) -> Self {
        match rotation {
            Rotation::None => *self,
            Rotation::Clockwise90 => Self {
                x: self.z,
                y: self.y,
                z: -self.x,
            },
            Rotation::Clockwise180 => Self {
                x: -self.x,
                y: self.y,
                z: -self.z,
            },
            Rotation::CounterClockwise90 => Self {
                x: -self.z,
                y: self.y,
                z: self.x,
            },
        }
    }
}

impl Rotate for DVec3 {
    fn rotate(&self, rotation: Rotation) -> Self {
        match rotation {
            Rotation::None => *self,
            Rotation::Clockwise90 => Self {
                x: self.z,
                y: self.y,
                z: -self.x + 1.0,
            },
            Rotation::Clockwise180 => Self {
                x: -self.x + 1.0,
                y: self.y,
                z: -self.z + 1.0,
            },
            Rotation::CounterClockwise90 => Self {
                x: -self.z + 1.0,
                y: self.y,
                z: self.x,
            },
        }
    }
}

impl Rotate for BlockAxis {
    fn rotate(&self, rotation: Rotation) -> Self {
        match rotation {
            Rotation::None | Rotation::Clockwise180 => *self,
            Rotation::CounterClockwise90 | Rotation::Clockwise90 => match self {
                BlockAxis::Y => BlockAxis::Y,
                BlockAxis::X => BlockAxis::Z,
                BlockAxis::Z => BlockAxis::X,
                BlockAxis::None => BlockAxis::None,
            },
        }
    }
}

impl Rotate for HorizontalDirection {
    fn rotate(&self, rotation: Rotation) -> Self {
        match rotation {
            Rotation::None => *self,
            Rotation::Clockwise90 => match self {
                HorizontalDirection::North => HorizontalDirection::East,
                HorizontalDirection::East => HorizontalDirection::South,
                HorizontalDirection::South => HorizontalDirection::West,
                HorizontalDirection::West => HorizontalDirection::North,
            },
            Rotation::Clockwise180 => match self {
                HorizontalDirection::North => HorizontalDirection::South,
                HorizontalDirection::East => HorizontalDirection::West,
                HorizontalDirection::South => HorizontalDirection::North,
                HorizontalDirection::West => HorizontalDirection::East,
            },
            Rotation::CounterClockwise90 => match self {
                HorizontalDirection::North => HorizontalDirection::West,
                HorizontalDirection::East => HorizontalDirection::North,
                HorizontalDirection::South => HorizontalDirection::East,
                HorizontalDirection::West => HorizontalDirection::South,
            },
        }
    }
}

impl Rotate for Direction {
    fn rotate(&self, rotation: Rotation) -> Self {
        match rotation {
            Rotation::None => *self,
            Rotation::Clockwise90 => match self {
                Direction::North => Direction::East,
                Direction::East => Direction::South,
                Direction::South => Direction::West,
                Direction::West => Direction::North,
                Direction::Up => Direction::Up,
                Direction::Down => Direction::Down,
            },
            Rotation::Clockwise180 => match self {
                Direction::North => Direction::South,
                Direction::East => Direction::West,
                Direction::South => Direction::North,
                Direction::West => Direction::East,
                Direction::Up => Direction::Up,
                Direction::Down => Direction::Down,
            },
            Rotation::CounterClockwise90 => match self {
                Direction::North => Direction::West,
                Direction::West => Direction::South,
                Direction::South => Direction::East,
                Direction::East => Direction::North,
                Direction::Up => Direction::Up,
                Direction::Down => Direction::Down,
            },
        }
    }
}

impl Rotate for LeverOrientation {
    fn rotate(&self, rotation: Rotation) -> Self {
        match rotation {
            Rotation::None => *self,
            Rotation::Clockwise90 => match self {
                LeverOrientation::DownX => LeverOrientation::DownZ,
                LeverOrientation::DownZ => LeverOrientation::DownX,
                LeverOrientation::UpX => LeverOrientation::UpZ,
                LeverOrientation::UpZ => LeverOrientation::UpX,
                LeverOrientation::North => LeverOrientation::East,
                LeverOrientation::East => LeverOrientation::South,
                LeverOrientation::South => LeverOrientation::West,
                LeverOrientation::West => LeverOrientation::North,
            },
            Rotation::Clockwise180 => match self {
                LeverOrientation::North => LeverOrientation::South,
                LeverOrientation::East => LeverOrientation::West,
                LeverOrientation::South => LeverOrientation::North,
                LeverOrientation::West => LeverOrientation::East,
                _ => *self
            },
            Rotation::CounterClockwise90 => match self {
                LeverOrientation::DownX => LeverOrientation::DownZ,
                LeverOrientation::DownZ => LeverOrientation::DownX,
                LeverOrientation::UpX => LeverOrientation::UpZ,
                LeverOrientation::UpZ => LeverOrientation::UpX,
                LeverOrientation::North => LeverOrientation::West,
                LeverOrientation::East => LeverOrientation::North,
                LeverOrientation::South => LeverOrientation::East,
                LeverOrientation::West => LeverOrientation::South,
            },
        }
    }
}

impl Rotate for TrapdoorDirection {
    fn rotate(&self, rotation: Rotation) -> Self {
        match rotation {
            Rotation::None => *self,
            Rotation::Clockwise90 => match self {
                TrapdoorDirection::North => TrapdoorDirection::East,
                TrapdoorDirection::East => TrapdoorDirection::South,
                TrapdoorDirection::South => TrapdoorDirection::West,
                TrapdoorDirection::West => TrapdoorDirection::North,
            },
            Rotation::Clockwise180 => match self {
                TrapdoorDirection::North => TrapdoorDirection::South,
                TrapdoorDirection::East => TrapdoorDirection::West,
                TrapdoorDirection::South => TrapdoorDirection::North,
                TrapdoorDirection::West => TrapdoorDirection::East,
            },
            Rotation::CounterClockwise90 => match self {
                TrapdoorDirection::North => TrapdoorDirection::West,
                TrapdoorDirection::East => TrapdoorDirection::North,
                TrapdoorDirection::South => TrapdoorDirection::East,
                TrapdoorDirection::West => TrapdoorDirection::South,
            },
        }
    }
}

impl Rotate for TorchDirection {
    fn rotate(&self, rotation: Rotation) -> Self {
        match rotation {
            Rotation::None => *self,
            Rotation::Clockwise90 => match self {
                TorchDirection::North => TorchDirection::East,
                TorchDirection::East => TorchDirection::South,
                TorchDirection::South => TorchDirection::West,
                TorchDirection::West => TorchDirection::North,
                TorchDirection::Up => TorchDirection::Up,
            },
            Rotation::Clockwise180 => match self {
                TorchDirection::North => TorchDirection::South,
                TorchDirection::East => TorchDirection::West,
                TorchDirection::South => TorchDirection::North,
                TorchDirection::West => TorchDirection::East,
                TorchDirection::Up => TorchDirection::Up,
            },
            Rotation::CounterClockwise90 => match self {
                TorchDirection::North => TorchDirection::West,
                TorchDirection::East => TorchDirection::North,
                TorchDirection::South => TorchDirection::East,
                TorchDirection::West => TorchDirection::South,
                TorchDirection::Up => TorchDirection::Up,
            },
        }
    }
}

impl Rotate for StairDirection {
    fn rotate(&self, rotation: Rotation) -> Self {
        match rotation {
            Rotation::None => *self,
            Rotation::Clockwise90 => match self {
                StairDirection::North => StairDirection::East,
                StairDirection::East => StairDirection::South,
                StairDirection::South => StairDirection::West,
                StairDirection::West => StairDirection::North,
            },
            Rotation::Clockwise180 => match self {
                StairDirection::North => StairDirection::South,
                StairDirection::East => StairDirection::West,
                StairDirection::South => StairDirection::North,
                StairDirection::West => StairDirection::East,
            },
            Rotation::CounterClockwise90 => match self {
                StairDirection::North => StairDirection::West,
                StairDirection::East => StairDirection::North,
                StairDirection::South => StairDirection::East,
                StairDirection::West => StairDirection::South,
            },
        }
    }
}

impl Rotate for ButtonDirection {
    fn rotate(&self, rotation: Rotation) -> Self {
        match rotation {
            Rotation::None => *self,
            Rotation::Clockwise90 => match self {
                ButtonDirection::North => ButtonDirection::East,
                ButtonDirection::East => ButtonDirection::South,
                ButtonDirection::South => ButtonDirection::West,
                ButtonDirection::West => ButtonDirection::North,
                ButtonDirection::Up => ButtonDirection::Up,
                ButtonDirection::Down => ButtonDirection::Down,
            },
            Rotation::Clockwise180 => match self {
                ButtonDirection::North => ButtonDirection::South,
                ButtonDirection::East => ButtonDirection::West,
                ButtonDirection::South => ButtonDirection::North,
                ButtonDirection::West => ButtonDirection::East,
                ButtonDirection::Up => ButtonDirection::Up,
                ButtonDirection::Down => ButtonDirection::Down,
            },
            Rotation::CounterClockwise90 => match self {
                ButtonDirection::North => ButtonDirection::West,
                ButtonDirection::East => ButtonDirection::North,
                ButtonDirection::South => ButtonDirection::East,
                ButtonDirection::West => ButtonDirection::South,
                ButtonDirection::Up => ButtonDirection::Up,
                ButtonDirection::Down => ButtonDirection::Down,
            },
        }
    }
}

impl Rotate for RailShape {
    fn rotate(&self, rotation: Rotation) -> Self {
        match rotation {
            Rotation::None => *self,
            Rotation::Clockwise90 => match self {
                RailShape::NorthSouth => RailShape::EastWest,
                RailShape::EastWest => RailShape::NorthSouth,
                RailShape::AscendingNorth => RailShape::AscendingEast,
                RailShape::AscendingEast => RailShape::AscendingSouth,
                RailShape::AscendingSouth => RailShape::AscendingWest,
                RailShape::AscendingWest => RailShape::AscendingNorth,
                RailShape::NorthEast => RailShape::SouthEast,
                RailShape::SouthEast => RailShape::SouthWest,
                RailShape::SouthWest => RailShape::NorthWest,
                RailShape::NorthWest => RailShape::NorthEast,
            },
            Rotation::Clockwise180 => match self {
                RailShape::AscendingNorth => RailShape::AscendingSouth,
                RailShape::AscendingEast => RailShape::AscendingWest,
                RailShape::AscendingSouth => RailShape::AscendingNorth,
                RailShape::AscendingWest => RailShape::AscendingEast,
                RailShape::NorthEast => RailShape::SouthWest,
                RailShape::SouthEast => RailShape::NorthWest,
                RailShape::SouthWest => RailShape::NorthEast,
                RailShape::NorthWest => RailShape::SouthEast,
                _ => *self,
            },
            Rotation::CounterClockwise90 => match self {
                RailShape::NorthSouth => RailShape::EastWest,
                RailShape::EastWest => RailShape::NorthSouth,
                RailShape::AscendingNorth => RailShape::AscendingWest,
                RailShape::AscendingEast => RailShape::AscendingNorth,
                RailShape::AscendingSouth => RailShape::AscendingEast,
                RailShape::AscendingWest => RailShape::AscendingSouth,
                RailShape::NorthEast => RailShape::NorthWest,
                RailShape::SouthEast => RailShape::NorthEast,
                RailShape::SouthWest => RailShape::SouthEast,
                RailShape::NorthWest => RailShape::SouthWest,
            },
        }
    }
}

impl Rotate for VineMetadata {
    fn rotate(&self, rotation: Rotation) -> Self {
        match rotation {
            Rotation::None => *self,
            Rotation::CounterClockwise90 => VineMetadata::from_meta(
                (self.west() as u8)
                    | (self.north() as u8) << 1
                    | (self.east() as u8) << 2
                    | (self.south() as u8) << 3,
            ),
            Rotation::Clockwise180 => VineMetadata::from_meta(
                (self.north() as u8)
                    | (self.east() as u8) << 1
                    | (self.south() as u8) << 2
                    | (self.west() as u8) << 3,
            ),
            Rotation::Clockwise90 => VineMetadata::from_meta(
                (self.east() as u8)
                    | (self.south() as u8) << 1
                    | (self.west() as u8) << 2
                    | (self.north() as u8) << 3,
            ),
        }
    }
}
