use crate::block::metadata::BlockMetadata;
use crate::block::rotatable::Rotatable;
use macros::BlockMetadata;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, BlockMetadata)]
pub enum Direction {
    South,
    West,
    North,
    East,
}

impl Rotatable for Direction {
    fn rotate(&self, other: Direction3D) -> Self {
        match other {
            Direction3D::North => {
                match self {
                    Direction::North => Direction::North,
                    Direction::East => Direction::East,
                    Direction::South => Direction::South,
                    Direction::West => Direction::West,
                }
            },
            Direction3D::East => {
                match self {
                    Direction::North => Direction::East,
                    Direction::East => Direction::South,
                    Direction::South => Direction::West,
                    Direction::West => Direction::North,
                }
            }
            Direction3D::South => {
                match self {
                    Direction::North => Direction::South,
                    Direction::East => Direction::West,
                    Direction::South => Direction::North,
                    Direction::West => Direction::East,
                }
            }
            Direction3D::West => {
                match self {
                    Direction::North => Direction::West,
                    Direction::East => Direction::North,
                    Direction::South => Direction::East,
                    Direction::West => Direction::South,
                }
            }
            _ => unreachable!()

        }
    }
}


#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, BlockMetadata)]
pub enum Direction3D {
    Down = 0,
    Up = 1,
    North = 2, // -z
    South = 3, // +z
    West = 4,  // -x
    East = 5,  // +z
}

impl Rotatable for Direction3D {
    fn rotate(&self, other: Direction3D) -> Self {
        match other {
            Direction3D::North => {
                match self {
                    Direction3D::North => Direction3D::North,
                    Direction3D::East => Direction3D::East,
                    Direction3D::South => Direction3D::South,
                    Direction3D::West => Direction3D::West,
                    Direction3D::Up => Direction3D::Up,
                    Direction3D::Down => Direction3D::Down,
                }
            },

            Direction3D::East => {
                match self {
                    Direction3D::North => Direction3D::East,
                    Direction3D::East => Direction3D::South,
                    Direction3D::South => Direction3D::West,
                    Direction3D::West => Direction3D::North,
                    Direction3D::Up => Direction3D::Up,
                    Direction3D::Down => Direction3D::Down,
                }
            }
            Direction3D::South => {
                match self {
                    Direction3D::North => Direction3D::South,
                    Direction3D::East => Direction3D::West,
                    Direction3D::South => Direction3D::North,
                    Direction3D::West => Direction3D::East,
                    Direction3D::Up => Direction3D::Up,
                    Direction3D::Down => Direction3D::Down,
                }
            }
            Direction3D::West => {
                match self {
                    Direction3D::North => Direction3D::West,
                    Direction3D::East => Direction3D::North,
                    Direction3D::South => Direction3D::East,
                    Direction3D::West => Direction3D::South,
                    Direction3D::Up => Direction3D::Up,
                    Direction3D::Down => Direction3D::Down,
                }
            }
            _ => unreachable!()

        }
    }
}

impl Direction3D {

    pub fn from_index(index: usize) -> Direction3D {
        match index {
            0 => Direction3D::North,
            1 => Direction3D::East,
            2 => Direction3D::South,
            3 => Direction3D::West,
            _ => unreachable!()
        }
    }

    pub fn get_offset(&self) -> (i32, i32, i32) {
        match self {
            Direction3D::North => (0, 0, -1),
            Direction3D::East => (1, 0, 0),
            Direction3D::South => (0, 0, 1),
            Direction3D::West => (-1, 0, 0),
            Direction3D::Up => (0, 1, 0),
            Direction3D::Down => (0, -1, 0),
        }
    }
}