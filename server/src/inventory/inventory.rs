use crate::inventory::item::{get_item_stack, Item};
use crate::network::packets::packet_buffer::PacketBuffer;
use crate::network::protocol::play::clientbound::SetSlot;
use crate::network::protocol::play::serverbound::{ClickMode, ClickWindow};

pub struct Inventory<T : Item> {
    pub items: [Option<T>; 45],
    pub dragged_item: Option<T>
}

impl<T : Item> Inventory<T> {

    pub const fn new() -> Self {
        Self {
            items: [const { None }; 45],
            dragged_item: None
        }
    }

    pub fn set_slot(&mut self, index: usize, item: Option<T>) {
        if index >= 45 {
            return;
        }
        self.items[index] = item
    }

    pub fn get_hotbar_slot(&self, index: usize) -> &Option<T> {
        let index = index + 36;
        if index >= 36 && index <= 44 {
            return &self.items[index];
        }
        &None
    }

    pub fn handle_packet(
        &mut self, packet: &ClickWindow,
        packet_buffer: &mut PacketBuffer,
    ) -> bool {
        let mut requires_sync = false;

        // doesn't take into consideration items that can be split
        match packet.mode {
            ClickMode::NormalClick => {
                // doesn't take into consideration items that can be split
                if packet.slot_id < 0 {
                    packet_buffer.write_packet(&SetSlot {
                        window_id: -1,
                        slot: 0,
                        item_stack: get_item_stack(&self.dragged_item),
                    })
                } else {
                    let slot = packet.slot_id as usize;
                    if is_valid_range(slot) {
                        let item = &mut self.items[slot];
                        let dragged = &mut self.dragged_item;

                        if let Some(item) = &item {
                            if !item.can_move_in_inventory() {
                                return true
                            }
                        }
                        if get_item_stack(item) != packet.clicked_item {
                            requires_sync = true;
                        }
                        std::mem::swap(item, dragged);
                    }
                }
            }
            ClickMode::ShiftClick => {
                let slot = packet.slot_id as usize;
                if is_valid_range(slot) {
                    // check if it is moveable first
                    let item = &mut self.items[slot];
                    if let Some(item) = &item {
                        if !item.can_move_in_inventory() {
                            return true
                        }
                    }
                    
                    let item = item.take();
                    
                    if let Some(item) = &item {
                        if !item.can_move_in_inventory() { 
                            return true
                        }
                    }
                    
                    if get_item_stack(&item) != packet.clicked_item {
                        requires_sync = true;
                    }
                    
                    let range = if slot >= 36 { 9..36 } else { 36..45 };
                    for index in range {
                        if self.items[index].is_none() {
                            self.items[index] = item;
                            break;
                        }
                    }
                }
            }
            ClickMode::NumberKey => {
                let slot = packet.slot_id as usize;
                let button = packet.used_button as usize;

                if is_valid_range(slot) && button <= 9 {

                    let to_slot = 36 + button;
                    let item = &mut self.items[slot];

                    if let Some(item) = &item {
                        if !item.can_move_in_inventory() {
                            return true
                        }
                    }

                    // this is what hypixel did to allow ghost pickaxes
                    if to_slot == slot {
                        packet_buffer.write_packet(&SetSlot {
                            window_id: 0,
                            slot: slot as i16,
                            item_stack: get_item_stack(item),
                        });
                    } else {
                        let item = item.take();
                        let item_to = self.items[to_slot].take();
                        self.items[to_slot] = item;
                        self.items[slot] = item_to;
                    }
                }
            }
            ClickMode::Drop => {
                requires_sync = true;
            }
            _ => {}
        }
        requires_sync
    }
}

fn is_valid_range(index: usize) -> bool {
    index >= 9 && index <= 44
}