use bevy::prelude::{ChildOf, Children, Commands, Component, DetectChangesMut, Entity, EntityEvent, On, Query, RelationshipTarget, ResMut};
use glam::{dvec3, ivec3, IVec2};
use crate::core::block::Block;
use crate::core::chunk::chunk_grid::{iterate_blocks, ChunkGrid};
use crate::core::entity::components::riding::Riding;
use crate::core::entity::components::transform::Transform;
use crate::core::entity::entity_metadata::BatMetadata;
use crate::core::entity::Mob;
use crate::core::entity::object_metadata::ObjectMetadata;
use crate::dungeon::door::Door;

#[derive(EntityEvent)]
pub struct OpenDoorEvent {
    pub entity: Entity
}

pub fn open_door(
    event: On<OpenDoorEvent>,
    door_query: Query<&Door>,
    mut chunks: ResMut<ChunkGrid>,
    mut commands: Commands,
) {
    let door = door_query
        .get(event.entity)
        .unwrap();

    let opening_door = commands.spawn(OpeningDoor {
        position: door.position,
        ticks_left: 20,
    }).id();

    iterate_blocks(
        ivec3(door.position.x - 1, 69, door.position.y - 1),
        ivec3(door.position.x + 1, 72, door.position.y + 1),
        |position| {
            let bat = commands.spawn((
                Mob::new(BatMetadata {
                    flags: 0,
                    hanging: false,
                }),
                Transform::new(position.as_dvec3() + dvec3(0.5, -0.65, 0.5)),
                ChildOf(opening_door),
            )).id();

            commands.spawn((
                Mob::new_object(ObjectMetadata::FallingBlock {
                    block: door.get_block(),
                }),
                Transform::default(),
                ChildOf(bat),
                Riding(bat),
            ));

            chunks.set_block_at(Block::Barrier, position)
        }
    );
}

#[derive(Component)]
pub struct OpeningDoor {
    position: IVec2,
    ticks_left: usize,
}

pub fn handle_opening_door(
    mut query: Query<(Entity, &mut OpeningDoor, &Children)>,
    mut block_query: Query<&mut Transform>,
    mut chunks: ResMut<ChunkGrid>,
    mut commands: Commands,
) {
    for (entity, mut door, children) in query.iter_mut() {
        door.ticks_left -= 1;

        if door.ticks_left == 0 {
            iterate_blocks(
                ivec3(door.position.x - 1, 69, door.position.y - 1),
                ivec3(door.position.x + 1, 72, door.position.y + 1),
                |position| {
                    chunks.set_block_at(Block::Air, position);
                }
            );
            commands.entity(entity).despawn();
        }

        for entity in children.iter() {
            let mut transform = block_query.get_mut(entity).unwrap();
            transform.position.y -= 0.25;
            transform.set_changed();
        }
    }
}