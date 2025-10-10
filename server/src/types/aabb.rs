use glam::DVec3;

#[derive(Debug, Clone)]
pub struct AABB {
    pub min: DVec3,
    pub max: DVec3,
}

impl AABB {

    pub const ZERO: AABB = AABB {
        min: DVec3::ZERO,
        max: DVec3::ZERO,
    };

    pub fn new(min: DVec3, max: DVec3) -> Self {
        Self {
            min,
            max,
        }
    }
    
    pub fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y &&
        self.min.z <= other.max.z && self.max.z >= other.min.z
    }

    pub const fn from_height_width(height: f64, width: f64) -> Self {
        Self { 
            min: DVec3 { x: -width / 2.0, y: 0.0, z: -width / 2.0 },
            max: DVec3 { x: width / 2.0, y: height, z: width / 2.0 }
        }
    }
}