use crate::core::entity::components::transform::Transform;
use crate::core::player::Player;
use bevy::ecs::{component::Component, query::With, system::Query};
use bevy::prelude::Without;

#[derive(Component)]
pub struct NPCBehaviour {
    pub default_yaw: f32,
    pub default_pitch: f32,
}

pub(super) fn update_npcs(
    mut query: Query<(&NPCBehaviour, &mut Transform), Without<Player>>, // without player needed so it doesn't need param set
    player_query: Query<&Transform, With<Player>>
) {
    for (npc, mut transform) in query.iter_mut() {
        // could obviously be way more optimal, but later
        let player = player_query
            .iter()
            .filter(|p| transform.position.distance_squared(p.position) <= 25.0)
            .min_by(|a, b| {
               let dist_a = transform.position.distance_squared(a.position);
               let dist_b = transform.position.distance_squared(b.position);
                dist_a.partial_cmp(&dist_b).unwrap()
            });

        if let Some(player) = player {
            let direction = player.position - transform.position;
            let horizontal_dist = (direction.x.powi(2) + direction.z.powi(2)).sqrt();
            transform.yaw = (direction.z.atan2(direction.x).to_degrees() - 90.0) as f32;
            transform.pitch = (-direction.y.atan2(horizontal_dist).to_degrees()) as f32;
        } else {
            transform.yaw = npc.default_yaw;
            transform.pitch = npc.default_pitch;
        }
    }
}
