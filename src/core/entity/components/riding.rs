use bevy::prelude::{Component, Deref, DetectChangesMut, Entity, Query, With, Without};
use crate::core::entity::components::transform::Transform;

// scuffed impl, but whatever
#[derive(Component, Deref)]
pub struct Riding(pub Entity);

pub fn update_transform(
    mut riding_query: Query<(&mut Transform, &Riding), With<Riding>>,
    query: Query<&Transform, Without<Riding>>
) {
    for (mut transform, riding) in riding_query.iter_mut() {
        // maybe remove riding if parent entity doesn't exist
        let ridden_transform = query
            .get(**riding)
            .expect("ridden entity doesn't exist");

        transform.position = ridden_transform.position;
        transform.set_changed();
    }
}