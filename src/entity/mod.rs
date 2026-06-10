use bevy::prelude::*;
use glam::{DVec3, Vec3};
use std::f32::consts::PI;

#[derive(Debug, Copy, Clone, Default, Component)]
pub struct Transform {
    pub position: DVec3,
    pub yaw: f32,
    pub pitch: f32,
}

impl Transform {
    pub fn rotation_vec(&self) -> Vec3 {
        let (yaw_sin, yaw_cos) = (-self.yaw.to_radians() - PI).sin_cos();
        let (pitch_sin, pitch_cos) = (-self.pitch.to_radians()).sin_cos();
        Vec3::new(yaw_sin * -pitch_cos, pitch_sin, yaw_cos * -pitch_cos)
    }
}

#[derive(Debug, Copy, Clone, Default, Component, Deref, DerefMut)]
pub struct OldTransform(pub Transform);


pub trait EntityExt {
    fn mc_id(&self) -> i32;
}

impl EntityExt for Entity {
    fn mc_id(&self) -> i32 {
        self.index_u32() as i32
    }
}
