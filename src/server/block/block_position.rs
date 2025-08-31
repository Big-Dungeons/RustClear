use crate::net::packets::packet_deserialize::PacketDeserializable;
use crate::net::packets::packet_serialize::PacketSerializable;
use crate::server::utils::direction::Direction;
use crate::server::utils::dvec3::DVec3;
use bytes::{Buf, BytesMut};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl From<DVec3> for BlockPos {
    fn from(vec: DVec3) -> Self {
        Self {
            x: vec.x as i32,
            y: vec.y as i32,
            z: vec.z as i32,
        }
    }
}

impl BlockPos {
    
    pub fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }
    
    pub const fn is_invalid(&self) -> bool {
        self.y.is_negative()
    }

    pub const fn distance_squared(&self, other: &BlockPos) -> i32 {
        let x = self.x - other.x;
        let y = self.y - other.y;
        let z = self.z - other.z;
        x * x + y * y + z * z
    }

    pub fn distance_to(&self, other: &BlockPos) -> f32 {
        (self.distance_squared(other) as f32).sqrt()
    }

    pub fn replace_y(&self, y: i32) -> Self {
        Self {
            x: self.x,
            y,
            z: self.z,
        }
    }

    pub fn add_x(&self, x: i32) -> Self {
        Self {
            x: self.x + x,
            y: self.y,
            z: self.z,
        }
    }

    pub fn add_y(&self, y: i32) -> Self {
        Self {
            x: self.x,
            y: self.y + y,
            z: self.z,
        }
    }

    pub fn add_z(&self, z: i32) -> Self {
        Self {
            x: self.x,
            y: self.y,
            z: self.z + z,
        }
    }

    pub fn rotate(&self, rotation: Direction) -> Self {
        match rotation {
            Direction::North => Self { x: self.x, y: self.y, z: self.z },
            Direction::East => Self { x: -self.z, y: self.y, z: self.x },
            Direction::South => Self { x: -self.x, y: self.y, z: -self.z },
            Direction::West => Self { x: self.z, y: self.y, z: -self.x },
            _ => Self { x: self.x, y: self.y, z: self.z },
        }
    }

    pub fn add(&self, other: BlockPos) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
            z: self.z + other.z
        }
    }

    pub fn subtract(&self, other: &BlockPos) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
            z: self.z - other.z
        }
    }

    pub fn as_dvec3(&self) -> DVec3 {
            DVec3 {
                x: self.x as f64,
                y: self.y as f64,
                z: self.z as f64
            }
    }
}

const XZ_BITS: i32 = 26;
const Y_BITS: i32 = 12;

const X_SHIFT: i32 = 38;
const Y_SHIFT: i32 = 26;

const XZ_MASK: i64 = 0x3FFFFFF;
const Y_MASK: i64 = 0xFFF;


impl PacketSerializable for BlockPos {
    fn write_size(&self) -> usize {
        const { size_of::<i64>() }
    }
    fn write(&self, buf: &mut BytesMut) {
        let long: i64 = (self.x as i64 & XZ_MASK) << X_SHIFT | (self.y as i64 & Y_MASK) << Y_SHIFT | (self.z as i64 & XZ_MASK);
        long.write(buf);
    }
}

impl PacketDeserializable for BlockPos {
    fn read(buffer: &mut BytesMut) -> anyhow::Result<Self> {
        let long = buffer.get_i64();
        Ok(BlockPos {
            x: (long << (64 - X_SHIFT - XZ_BITS) >> (64 - XZ_BITS)) as i32,
            y: (long << (64 - Y_SHIFT - Y_BITS) >> (64 - Y_BITS)) as i32,
            z: (long << (64 - XZ_BITS) >> (64 - XZ_BITS)) as i32,
        })
    }
}
