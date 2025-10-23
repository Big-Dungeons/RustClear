use crate::entity::entity_appearance::EntityAppearance;
use crate::network::protocol::play::clientbound::DestroyEntites;
use crate::network::protocol::play::serverbound::EntityInteractionType;
use crate::{Player, World, WorldExtension};
use glam::DVec3;
use std::ptr::NonNull;

pub type EntityId = i32;

#[allow(unused_variables)]
pub trait EntityExtension<W : WorldExtension> {
    fn tick(&mut self, entity: &mut EntityBase<W>);

    fn interact(
        &mut self,
        entity: &mut EntityBase<W>,
        player: &mut Player<W::Player>,
        action: EntityInteractionType
    ) {}
}

pub struct EntityBase<W : WorldExtension> {
    world: NonNull<World<W>>,
    pub id: EntityId,

    pub position: DVec3,
    pub velocity: DVec3,
    pub yaw: f32,
    pub pitch: f32,

    pub last_position: DVec3,
    pub last_yaw: f32,
    pub last_pitch: f32,

    pub ticks_existed: u32,
}

impl<W : WorldExtension> EntityBase<W> {

    pub fn world<'a>(&self) -> &'a World<W> {
        unsafe { self.world.as_ref() }
    }

    pub fn world_mut<'a>(&mut self) -> &'a mut World<W> {
        unsafe { self.world.as_mut() }
    }
}

pub struct Entity<W : WorldExtension> {
    pub base: EntityBase<W>,
    appearance: Box<dyn EntityAppearance<W>>,
    extension: Box<dyn EntityExtension<W>>,
}

impl<W : WorldExtension> Entity<W> {

    pub fn new<A : EntityAppearance<W> + 'static, E : EntityExtension<W> + 'static>(
        world: &mut World<W>,
        entity_id: EntityId,
        position: DVec3,
        yaw: f32,
        pitch: f32,
        appearance: A,
        extension: E,
    ) -> Self {
        let mut base = EntityBase {
            world: NonNull::from_mut(world),
            id: entity_id,
            position,
            velocity: DVec3::ZERO,
            yaw,
            pitch,
            last_position: position,
            last_yaw: yaw,
            last_pitch: pitch,
            ticks_existed: 0,
        };
        appearance.initialize(&mut base);
        Self {
            base,
            appearance: Box::new(appearance),
            extension: Box::new(extension),
        }
    }

    pub fn tick(&mut self) {
        let base = &mut self.base;
        base.ticks_existed += 1;
        self.extension.tick(base);
        
        if base.position != base.last_position { 
            self.appearance.update_position(base);
            base.last_position = base.position;
        }
        if base.yaw != base.last_yaw || base.pitch != base.last_pitch {
            self.appearance.update_rotation(base);
            base.last_yaw = base.yaw;
            base.last_pitch = base.pitch;
        }
    }
    
    pub fn interact(&mut self, player: &mut Player<W::Player>, action: EntityInteractionType) {
        self.extension.interact(&mut self.base, player, action);
    }
    
    pub fn enter_view(&mut self, player: &mut Player<W::Player>) {
        self.appearance.enter_player_view(&mut self.base, player);
    }

    pub fn leave_view(&mut self, player: &mut Player<W::Player>) {
        self.appearance.enter_player_view(&mut self.base, player);
    }

    pub fn destroy(&mut self, packet: &mut DestroyEntites) {
        self.appearance.destroy(&mut self.base, packet);
    }
}