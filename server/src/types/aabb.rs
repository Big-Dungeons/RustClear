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

    pub const fn new(min: DVec3, max: DVec3) -> Self {
        Self {
            min,
            max,
        }
    }
    
    pub const fn intersects(&self, other: &AABB) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y &&
        self.min.z <= other.max.z && self.max.z >= other.min.z
    }

    pub const fn volume(&self) -> f64 {
        let dx = (self.max.x - self.min.x).max(0.0);
        let dy = (self.max.y - self.min.y).max(0.0);
        let dz = (self.max.z - self.min.z).max(0.0);
        dx * dy * dz
    }

    pub const fn intersection_volume(&self, other: &AABB) -> f64 {
        let dx = (self.max.x.min(other.max.x) - self.min.x.max(other.min.x)).max(0.0);
        let dy = (self.max.y.min(other.max.y) - self.min.y.max(other.min.y)).max(0.0);
        let dz = (self.max.z.min(other.max.z) - self.min.z.max(other.min.z)).max(0.0);
        dx * dy * dz
    }

    pub const fn from_height_width(height: f64, width: f64) -> Self {
        Self { 
            min: DVec3 { x: -width / 2.0, y: 0.0, z: -width / 2.0 },
            max: DVec3 { x: width / 2.0, y: height, z: width / 2.0 }
        }
    }
}