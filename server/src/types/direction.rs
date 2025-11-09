use crate::block::metadata::BlockMetadata;
use crate::block::rotatable::Rotatable;
use macros::BlockMetadata;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, BlockMetadata)]
pub enum Direction {
    Down = 0,
    Up = 1,
    North = 2, // -z
    South = 3, // +z
    West = 4,  // -x
    East = 5,  // +z
}

impl Rotatable for Direction {
    fn rotate(&self, other: Direction) -> Self {
        match other {
            Direction::North => {
                match self {
                    Direction::North => Direction::North,
                    Direction::East => Direction::East,
                    Direction::South => Direction::South,
                    Direction::West => Direction::West,
                    Direction::Up => Direction::Up,
                    Direction::Down => Direction::Down,
                }
            },

            Direction::East => {
                match self {
                    Direction::North => Direction::East,
                    Direction::East => Direction::South,
                    Direction::South => Direction::West,
                    Direction::West => Direction::North,
                    Direction::Up => Direction::Up,
                    Direction::Down => Direction::Down,
                }
            }
            Direction::South => {
                match self {
                    Direction::North => Direction::South,
                    Direction::East => Direction::West,
                    Direction::South => Direction::North,
                    Direction::West => Direction::East,
                    Direction::Up => Direction::Up,
                    Direction::Down => Direction::Down,
                }
            }
            Direction::West => {
                match self {
                    Direction::North => Direction::West,
                    Direction::East => Direction::North,
                    Direction::South => Direction::East,
                    Direction::West => Direction::South,
                    Direction::Up => Direction::Up,
                    Direction::Down => Direction::Down,
                }
            }
            _ => unreachable!()

        }
    }
}

impl Direction {

    pub fn from_index(index: usize) -> Direction {
        match index {
            0 => Direction::North,
            1 => Direction::East,
            2 => Direction::South,
            3 => Direction::West,
            _ => unreachable!()
        }
    }

    pub fn get_offset(&self) -> (i32, i32, i32) {
        match self {
            Direction::North => (0, 0, -1),
            Direction::East => (1, 0, 0),
            Direction::South => (0, 0, 1),
            Direction::West => (-1, 0, 0),
            Direction::Up => (0, 1, 0),
            Direction::Down => (0, -1, 0),
        }
    }
}