use crate::block::block_metadata::BlockFieldMetadata;
use macros::BlockFieldMetadata;

#[derive(Debug, Clone, Copy, PartialEq, Eq, BlockFieldMetadata)]
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


#[repr(u8)]
#[derive(PartialEq, Debug, Copy, Clone, Eq, BlockFieldMetadata)]
pub enum BlockAxis {
    // ordered weird, but this is the order that matches vanilla
    Y,
    X,
    Z,
    None,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, BlockFieldMetadata)]
pub enum HorizontalDirection {
    South,
    West,
    North,
    East,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, BlockFieldMetadata)]
pub enum Direction {
    Down,
    Up,
    North, // -z
    South, // +z
    West,  // -x
    East,  // +z
}

#[repr(u8)]
#[derive(PartialEq, Debug, Copy, Clone, Eq, BlockFieldMetadata)]
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

#[repr(u8)]
#[derive(PartialEq, Debug, Copy, Clone, Eq, BlockFieldMetadata)]
pub enum TrapdoorDirection {
    North,
    South,
    West,
    East,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, BlockFieldMetadata)]
pub enum TorchDirection {
    East = 1,
    West = 2,
    South = 3,
    North = 4,
    Up = 5,
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, BlockFieldMetadata)]
pub enum StairDirection {
    East,
    West,
    South,
    North,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, BlockFieldMetadata)]
pub enum ButtonDirection {
    Down,
    East,
    West,
    South,
    North,
    Up,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, BlockFieldMetadata)]
pub enum RailShape {
    NorthSouth,
    EastWest,
    AscendingEast,
    AscendingWest,
    AscendingNorth,
    AscendingSouth,
    SouthEast,
    SouthWest,
    NorthWest,
    NorthEast,
}

#[derive(PartialEq, Debug, Copy, Clone, Eq)]
pub struct VineMetadata(u8);

impl VineMetadata {
    pub fn south(self) -> bool {
        self.0 & 0x1 != 0
    }
    pub fn west(self) -> bool {
        self.0 & 0x2 != 0
    }
    pub fn north(self) -> bool {
        self.0 & 0x4 != 0
    }
    pub fn east(self) -> bool {
        self.0 & 0x8 != 0
    }
}

impl BlockFieldMetadata for VineMetadata {
    const META_SIZE: u8 = 4;

    fn get_meta(self) -> u8 {
        self.0
    }

    fn from_meta(meta: u8) -> Self {
        VineMetadata(meta)
    }
}