use bevy::prelude::{Component, Deref, DerefMut, Query};
use glam::{DVec3, Vec3};
use std::f32::consts::PI;

#[derive(Debug, Default, Copy, Clone, PartialEq, Component)]
pub struct Transform {
    pub position: DVec3,
    pub yaw: f32,
    pub pitch: f32,
}

impl Transform {
    pub fn new(position: impl Into<DVec3>) -> Self {
        Self {
            position: position.into(),
            yaw: 0.0,
            pitch: 0.0,
        }
    }
    
    pub fn rotation_vec(&self) -> Vec3 {
        let (yaw_sin, yaw_cos) = (-self.yaw.to_radians() - PI).sin_cos();
        let (pitch_sin, pitch_cos) = (-self.pitch.to_radians()).sin_cos();
        Vec3::new(yaw_sin * -pitch_cos, pitch_sin, yaw_cos * -pitch_cos)
    }
}

#[derive(Debug, Copy, Clone, Default, Component, Deref, DerefMut)]
pub struct OldTransform(pub Transform);

pub fn set_old_transform(mut query: Query<(&Transform, &mut OldTransform)>) {
    for (new, mut old) in query.iter_mut() {
        *old = OldTransform(*new)
    }
}