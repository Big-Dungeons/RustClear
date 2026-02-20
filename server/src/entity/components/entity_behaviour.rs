use crate::entity::entity::MinecraftEntity;
use crate::WorldExtension;
use bevy_ecs::component::{Component, Mutable};
use bevy_ecs::world::World;

// could maybe use a macro to provide custom query options somehow?
pub trait EntityBehaviour<W : WorldExtension + 'static>: Component<Mutability = Mutable> + Sized {
    fn tick(entity: &mut MinecraftEntity<W>, component: &mut Self);

    fn query(world: &mut World) {
        let mut query = world.query::<(&mut MinecraftEntity<W>, &mut Self)>();
        for (mut entity, mut component) in query.iter_mut(world) {
            Self::tick(&mut *entity, &mut *component);
        }
    }
}