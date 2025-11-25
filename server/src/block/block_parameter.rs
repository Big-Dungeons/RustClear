use crate::block::metadata::BlockMetadata;
use crate::block::rotatable::Rotate;
use crate::types::direction::{Direction, Direction3D};
use macros::BlockMetadata;

/// Used for blocks like: Wool, Stained Glass, Stained Hardened Clay
#[derive(Debug, Clone, Copy, PartialEq, Eq, BlockMetadata)]
pub enum BlockColor {
    White,
    Orange,
    Magenta,
    LightBlue,
    Yellow,
    Lime,
    Pink,
    Gray,
    LightGray,
    Cyan,
    Purple,
    Blue,
    Brown,
    Green,
    Red,
    Black,
}

/// This type of rotation is used in blocks like Logs, etc
#[repr(u8)]
#[derive(PartialEq, Debug, Copy, Clone, Eq, BlockMetadata)]
pub enum Axis {
    Y,
    X,
    Z,
    None,
}

impl Axis {
    pub fn get_direction(&self) -> Direction {
        match self {
            Axis::X => Direction::East,
            Axis::Z => Direction::North,
            _ => unreachable!(),
        }
    }
}

impl Rotate for Axis {
    fn rotate(&self, other: Direction) -> Self {
        match other {
            Direction::North | Direction::South => *self,
            Direction::East | Direction::West => match self {
                Axis::Y => Axis::Y,
                Axis::X => Axis::Z,
                Axis::Z => Axis::X,
                Axis::None => Axis::None,
            },
        }
    }
}

// TODO: This needs rotation
/// Used for exclusively lever.
#[repr(u8)]
#[derive(PartialEq, Debug, Copy, Clone, Eq, BlockMetadata)]
pub enum LeverOrientation {
    DownX,
    East,
    West,
    South,
    North,
    UpZ,
    UpX,
    DownZ,
}

// TODO: Rotate, or maybe wrap around a different direction and just have different Blockmetadata impl
#[repr(u8)]
#[derive(PartialEq, Debug, Copy, Clone, Eq, BlockMetadata)]
pub enum TrapdoorDirection {
    North,
    South,
    West,
    East,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, BlockMetadata)]
pub enum TorchDirection {
    East = 1,
    West = 2,
    South = 3,
    North = 4,
    Up = 5,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, BlockMetadata)]
pub enum StairDirection {
    East,
    West,
    South,
    North,
}

impl Rotate for StairDirection {
    fn rotate(&self, other: Direction) -> Self {
        match other {
            Direction::North => match self {
                StairDirection::North => StairDirection::North,
                StairDirection::East => StairDirection::East,
                StairDirection::South => StairDirection::South,
                StairDirection::West => StairDirection::West,
            },
            Direction::East => match self {
                StairDirection::North => StairDirection::East,
                StairDirection::East => StairDirection::South,
                StairDirection::South => StairDirection::West,
                StairDirection::West => StairDirection::North,
            },
            Direction::South => match self {
                StairDirection::North => StairDirection::South,
                StairDirection::East => StairDirection::West,
                StairDirection::South => StairDirection::North,
                StairDirection::West => StairDirection::East,
            },
            Direction::West => match self {
                StairDirection::North => StairDirection::West,
                StairDirection::East => StairDirection::North,
                StairDirection::South => StairDirection::East,
                StairDirection::West => StairDirection::South,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonDirection(Direction3D);

impl Rotate for ButtonDirection {
    fn rotate(&self, direction: Direction) -> Self {
        ButtonDirection(self.0.rotate(direction))
    }
}

impl BlockMetadata for ButtonDirection {
    const META_SIZE: u8 = 3;

    fn get_meta(&self) -> u8 {
        match self.0 {
            Direction3D::Down => 0,
            Direction3D::East => 1,
            Direction3D::West => 2,
            Direction3D::South => 3,
            Direction3D::North => 4,
            Direction3D::Up => 5,
        }
    }
    fn from_meta(meta: u8) -> Self {
        ButtonDirection(match meta & 0b111 {
            0 => Direction3D::Down,
            1 => Direction3D::East,
            2 => Direction3D::West,
            3 => Direction3D::South,
            4 => Direction3D::North,
            _ => Direction3D::Up, // 5–7 fall back to Up
        })
    }
}

// todo: rotatable
#[derive(Debug, Clone, Copy, PartialEq, Eq, BlockMetadata)]
pub enum RailShape {
    NorthSouth,
    EastWest,
    AscendingEast,
    AscendingWest,
    AcsendingNorth,
    AscendingSouth,
    SouthEast,
    SouthWest,
    NorthWest,
    NorthEast,
}


// TODO: This needs rotation
#[repr(transparent)]
#[derive(PartialEq, Debug, Copy, Clone, Eq)]
pub struct VineMetadata(u8);

impl BlockMetadata for VineMetadata {
    const META_SIZE: u8 = 4;

    fn get_meta(&self) -> u8 {
        self.0
    }
    fn from_meta(meta: u8) -> Self {
        VineMetadata(meta)
    }
}