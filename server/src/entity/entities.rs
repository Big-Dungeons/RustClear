use crate::entity::components::{EntityAppearance, EntityBehaviour};
use crate::entity::entity::MinecraftEntity;
use crate::WorldExtension;
use bevy_ecs::bundle::Bundle;
use bevy_ecs::entity::Entity;
use bevy_ecs::world::{EntityRef, EntityWorldMut, World};
use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;

pub type MCEntityId = i32;

#[derive(Default)]
struct Systems {
    pre: HashSet<fn(&mut World)>,
    post: HashSet<fn(&mut World)>,
}

pub struct Entities<W: WorldExtension> {
    ecs: World,
    tick_systems: Systems,

    current_entity_id: MCEntityId,
    id_to_entities: HashMap<MCEntityId, Entity>,

    phantom: PhantomData<W>
}

impl<W: WorldExtension + 'static> Entities<W> {

    pub fn new() -> Self {
        Self {
            ecs: World::new(),
            tick_systems: Default::default(),

            current_entity_id: 0,
            id_to_entities: Default::default(),

            phantom: Default::default(),
        }
    }

    pub fn tick(&mut self) {
        for system in self.tick_systems.pre.iter() {
            system(&mut self.ecs)
        }
        for system in self.tick_systems.post.iter() {
            system(&mut self.ecs)
        }
    }

    pub fn register_tick_system(&mut self, callback: fn(&mut World)) {
        self.tick_systems.pre.insert(callback);
    }

    pub fn register_behaviour<T: EntityBehaviour<W>>(&mut self) {
        self.tick_systems.pre.insert(T::query);
    }

    // maybe rename, since this is what also ticks mc entity
    pub(crate) fn register_appearance_update<T: EntityAppearance<W>>(&mut self) {
        self.tick_systems.post.insert(|world| {
            let mut query = world.query::<(&mut MinecraftEntity<W>, &T)>();
            for (mut entity, appearance) in query.iter_mut(world) {
                entity.update(appearance);
            }
        });
    }

    pub fn mc_id_to_entity(&self, id: MCEntityId) -> Option<&Entity> {
        self.id_to_entities.get(&id)
    }

    pub fn get_entity(&'_ self, entity: Entity) -> EntityRef<'_> {
        self.ecs.entity(entity)
    }

    pub fn get_entity_mut(&'_ mut self, entity: Entity) -> EntityWorldMut<'_> {
        self.ecs.entity_mut(entity)
    }

    pub const fn next_entity_id(&mut self) -> MCEntityId {
        self.current_entity_id += 1;
        self.current_entity_id
    }

    pub fn spawn<B: Bundle>(&mut self, entity: B) -> Entity {
        let entity = self.ecs.spawn(entity);
        if let Some(base) = entity.get::<MinecraftEntity<W>>() {
            let mc_id = base.id;
            self.id_to_entities.insert(mc_id, entity.id());
        };
        entity.id()
    }

    pub fn despawn(&mut self, entity: Entity) {
        let entity = self.ecs.entity_mut(entity);
        if let Some(base) = entity.get::<MinecraftEntity<W>>() {
            self.id_to_entities.remove(&base.id);
        }
        entity.despawn();
    }
}