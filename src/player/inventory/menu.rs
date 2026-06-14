use crate::network::packets::BytesMutExt;
use crate::network::protocol::play::clientbound::{OpenWindow, SetSlot, WindowItems};
use crate::network::protocol::play::serverbound::ClickMode;
use crate::player::inventory::item_stack::ItemStack;
use crate::types::chat_component::ChatComponent;
use bevy::prelude::{Add, Commands, Component, Entity, EntityEvent, On};
use bytes::BytesMut;

#[derive(Component)]
pub struct Menu {
    pub title: String,
    pub items: Vec<Option<ItemStack>>,
}

impl Menu {
    pub fn open_menu(&self, window_id: i8, buffer: &mut BytesMut) {
        debug_assert!(self.items.len().is_multiple_of(9));

        buffer.write_packet(&OpenWindow {
            window_id,
            inventory_type: "minecraft:container",
            window_title: ChatComponent::new(self.title.clone()),
            slot_count: self.items.len() as i8,
        });

        self.sync_menu(window_id, buffer);
    }

    pub fn sync_menu(&self, window_id: i8, buffer: &mut BytesMut) {
        buffer.write_packet(&WindowItems {
            window_id,
            items: &self.items,
        });
        buffer.write_packet(&SetSlot {
            window_id: -1,
            slot: 0,
            item_stack: None,
        })
    }
}


#[derive(EntityEvent)]
pub struct MenuClick {
    #[event_target]
    pub menu_entity: Entity,
    pub client: Entity,
    pub slot: usize,
    pub _click_mode: ClickMode,
}

// kind of scuffed,
// however there will only be a couple of very generic menus, so no point in overcomplicating it
#[derive(EntityEvent)]
pub struct UpdateMenu {
    #[event_target]
    pub menu_entity: Entity
}

pub(super) fn on_menu_init(
    event: On<Add, Menu>,
    mut commands: Commands
) {
    commands.trigger(UpdateMenu { menu_entity: event.entity });
}