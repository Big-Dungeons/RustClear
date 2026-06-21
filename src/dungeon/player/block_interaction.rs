use crate::core::player::interact::PlayerRightClick;
use crate::dungeon::rng::DHashMap;
use bevy::prelude::*;
use glam::IVec3;

#[derive(Component)]
pub struct BlockInteractable {
    pub position: IVec3,
}

#[derive(EntityEvent)]
pub struct BlockInteractionEvent {
    pub entity: Entity,
    pub player: Entity,
}

pub(super) fn on_add_block_interactable(
    event: On<Add, BlockInteractable>,
    query: Query<&BlockInteractable>,
    mut lookup: ResMut<BlockInteractableLookup>
) {
    let interactable = query
        .get(event.entity)
        .unwrap();

    debug_assert!(
        !lookup.contains_key(&interactable.position),
        "block interactable position already used"
    );

    lookup.insert(interactable.position, event.entity);
}

pub(super) fn on_remove_block_interactable(
    event: On<Remove, BlockInteractable>,
    query: Query<&BlockInteractable>,
    mut lookup: ResMut<BlockInteractableLookup>
) {
    let interactable = query
        .get(event.entity)
        .unwrap();

    debug_assert!(
        lookup.contains_key(&interactable.position),
        "removed block interactable, but it's position wasn't in the lookup"
    );

    lookup.insert(interactable.position, event.entity);
}

#[derive(Default, Resource, Deref, DerefMut)]
pub(super) struct BlockInteractableLookup(DHashMap<IVec3, Entity>);


pub(super) fn on_player_interact(
    mut events: MessageReader<PlayerRightClick>,
    lookup: Res<BlockInteractableLookup>,
    mut commands: Commands,
) {
    for PlayerRightClick { client, block_interact_result: result } in events.read() {
        if let Some(result) = result && let Some(entity) = lookup.get(&result.position) {
            commands.trigger(BlockInteractionEvent {
                entity: *entity,
                player: *client,
            })
        }
    }
}