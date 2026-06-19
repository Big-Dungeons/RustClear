use glam::DVec3;

#[allow(clippy::upper_case_acronyms)] // reason: Aabb is ugly
#[derive(Debug, Copy, Clone)]
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

    pub fn contains(&self, point: DVec3) -> bool {
        point.x >= self.min.x && point.x <= self.max.x &&
        point.y >= self.min.y && point.y <= self.max.y &&
        point.z >= self.min.z && point.z <= self.max.z
    }

    pub fn offset(self, dvec3: DVec3) -> AABB {
        AABB::new(self.min + dvec3, self.max + dvec3)
    }
}