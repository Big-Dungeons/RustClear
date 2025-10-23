use crate::block::metadata::BlockMetadata;
use crate::block::rotatable::Rotatable;
use crate::types::direction::Direction;
use blocks::BlockMetadata;

/// This type of rotation is used in blocks like Logs, etc
#[repr(u8)]
#[derive(PartialEq, Debug, Copy, Clone, Eq, BlockMetadata)]
pub enum Axis {
    Y,
    X,
    Z,
    None
}

impl Rotatable for Axis {
    fn rotate(&self, other: Direction) -> Self {
        match other {
            Direction::North | Direction::South => {
                *self
            }
            Direction::East | Direction::West => {
                match self {
                    Axis::Y => Axis::Y,
                    Axis::X => Axis::Z,
                    Axis::Z => Axis::X,
                    Axis::None => Axis::None,
                }
            }
            _ => unreachable!()
        }
    }
}

impl Axis {
    pub fn get_direction(&self) -> Direction {
        match self {
            Axis::X => Direction::East,
            Axis::Z => Direction::North,
            _ => Direction::Up
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
    DownZ
}

// TODO: Rotate, or maybe wrap around a different direction and just have different Blockmetadata impl
#[repr(u8)]
#[derive(PartialEq, Debug, Copy, Clone, Eq, BlockMetadata)]
pub enum TrapdoorDirection {
    North,
    South,
    West,
    East
}

// TODO: This needs rotation
#[repr(transparent)]
#[derive(PartialEq, Debug, Copy, Clone, Eq)]
pub struct VineMetadata(u8);

impl VineMetadata {
    pub fn meta_size() -> u8 {
        4
    }
    pub fn get_meta(&self) -> u8 {
        self.0
    }
    pub fn from_meta(meta: u8) -> Self {
        VineMetadata(meta)
    }
}

/// custom direction for Torch blocks, since they used a slightly modified version of direction.
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
pub enum HorizontalDirection {
    South,
    West,
    North,
    East,
}

impl Rotatable for HorizontalDirection {
    fn rotate(&self, other: Direction) -> Self {
        match other {
            Direction::North => {
                match self {
                    HorizontalDirection::North => HorizontalDirection::North,
                    HorizontalDirection::East => HorizontalDirection::East,
                    HorizontalDirection::South => HorizontalDirection::South,
                    HorizontalDirection::West => HorizontalDirection::West,
                }
            },
            Direction::East => {
                match self {
                    HorizontalDirection::North => HorizontalDirection::East,
                    HorizontalDirection::East => HorizontalDirection::South,
                    HorizontalDirection::South => HorizontalDirection::West,
                    HorizontalDirection::West => HorizontalDirection::North,
                }
            }
            Direction::South => {
                match self {
                    HorizontalDirection::North => HorizontalDirection::South,
                    HorizontalDirection::East => HorizontalDirection::West,
                    HorizontalDirection::South => HorizontalDirection::North,
                    HorizontalDirection::West => HorizontalDirection::East,
                }
            }
            Direction::West => {
                match self {
                    HorizontalDirection::North => HorizontalDirection::West,
                    HorizontalDirection::East => HorizontalDirection::North,
                    HorizontalDirection::South => HorizontalDirection::East,
                    HorizontalDirection::West => HorizontalDirection::South,
                }
            }
            _ => unreachable!()

        }
    }
}


#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, BlockMetadata)]
pub enum StairDirection {
    East,
    West,
    South,
    North,
}

impl Rotatable for StairDirection {
    fn rotate(&self, other: Direction) -> Self {
        match other {
            Direction::North => {
                match self {
                    StairDirection::North => StairDirection::North,
                    StairDirection::East => StairDirection::East,
                    StairDirection::South => StairDirection::South,
                    StairDirection::West => StairDirection::West,
                }
            },
            Direction::East => {
                match self {
                    StairDirection::North => StairDirection::East,
                    StairDirection::East => StairDirection::South,
                    StairDirection::South => StairDirection::West,
                    StairDirection::West => StairDirection::North,
                }
            }
            Direction::South => {
                match self {
                    StairDirection::North => StairDirection::South,
                    StairDirection::East => StairDirection::West,
                    StairDirection::South => StairDirection::North,
                    StairDirection::West => StairDirection::East,
                }
            }
            Direction::West => {
                match self {
                    StairDirection::North => StairDirection::West,
                    StairDirection::East => StairDirection::North,
                    StairDirection::South => StairDirection::East,
                    StairDirection::West => StairDirection::South,
                }
            }
            _ => unreachable!()

        }
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ButtonDirection(Direction);

impl Rotatable for ButtonDirection {
    fn rotate(&self, direction: Direction) -> Self {
        ButtonDirection(self.0.rotate(direction))
    }
}

impl BlockMetadata for ButtonDirection {
    fn meta_size() -> u8 {
        3
    }
    fn get_meta(&self) -> u8 {
        match self.0 {
            Direction::Down => 0,
            Direction::East => 1,
            Direction::West => 2,
            Direction::South => 3,
            Direction::North => 4,
            Direction::Up => 5,
        }
    }
    fn from_meta(meta: u8) -> Self {
        ButtonDirection(match meta & 0b111 {
            0 => Direction::Down,
            1 => Direction::East,
            2 => Direction::West,
            3 => Direction::South,
            4 => Direction::North,
            _ => Direction::Up, // 5–7 fall back to Up
        })
    }
}