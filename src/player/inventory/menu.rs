use crate::player::inventory::item_stack::ItemStack;
use bevy::prelude::{Component, Entity};
use std::collections::HashMap;
use bytes::BytesMut;
use crate::network::packets::BytesMutExt;
use crate::network::protocol::play::clientbound::{OpenWindow, SetSlot, WindowItems};
use crate::types::chat_component::ChatComponent;

#[derive(Component)]
pub struct Menu {
    pub title: String,
    pub items: Vec<Option<ItemStack>>,
    pub callbacks: HashMap<usize, fn(Entity)>,
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