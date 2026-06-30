use bevy::prelude::{Component, Entity};
use bytes::BytesMut;
use crate::core::entity::BevyEntityExt;
use crate::core::network::packets::BytesMutExt;
use crate::core::network::protocol::play::clientbound::EntityEquipment;
use crate::core::network::protocol::var_int::VarInt;
use crate::core::player::inventory::item_stack::ItemStack;

#[derive(Component)]
pub struct Equipment {
    pub hand: Option<ItemStack>,
    pub armor: [Option<ItemStack>; 4], // boots, leggings, chestplate, helmet
}

impl Equipment {
    pub fn new() -> Self {
        Self {
            hand: None,
            armor: [const { None }; 4],
        }
    }

    pub fn hand(mut self, item_stack: ItemStack) -> Self {
        self.hand = Some(item_stack);
        self
    }

    pub fn helmet(mut self, item_stack: ItemStack) -> Self {
        self.armor[3] = Some(item_stack);
        self
    }

    pub fn chestplate(mut self, item_stack: ItemStack) -> Self {
        self.armor[2] = Some(item_stack);
        self
    }

    pub fn leggings(mut self, item_stack: ItemStack) -> Self {
        self.armor[1] = Some(item_stack);
        self
    }

    pub fn boots(mut self, item_stack: ItemStack) -> Self {
        self.armor[0] = Some(item_stack);
        self
    }

    pub fn write_packets(&self, entity: Entity, packet_buffer: &mut BytesMut) {
        packet_buffer.write_packet(&EntityEquipment {
            entity_id: VarInt(entity.mc_id()),
            item_slot: 0,
            item_stack: self.hand.clone(),
        });
        for (index, item_stack) in self.armor.iter().enumerate() {
            packet_buffer.write_packet(&EntityEquipment {
                entity_id: VarInt(entity.mc_id()),
                item_slot: (index + 1) as i16,
                item_stack: item_stack.clone(),
            })
        }
    }
}