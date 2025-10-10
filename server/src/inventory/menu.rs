
use crate::inventory::item::get_item_stack;
use crate::inventory::item_stack::ItemStack;
use crate::network::protocol::play::clientbound::{OpenWindow, SetSlot, WindowItems};
use crate::network::protocol::play::serverbound::ClickWindow;
use crate::player::player::{Player, PlayerExtension};
use crate::types::chat_component::ChatComponent;

pub trait Menu<P : PlayerExtension> {
    
    fn container_name(&self, player: &mut Player<P>) -> &str;
    
    fn container_items(&self, player: &mut Player<P>) -> Vec<Option<ItemStack>>;
    
    fn click_window(&mut self, player: &mut Player<P>, packet: &ClickWindow);
    
}

pub enum OpenContainer<P : PlayerExtension> {
    None,
    Inventory,
    Menu(Box<dyn Menu<P>>)
}

impl<P : PlayerExtension> OpenContainer<P> {
    
    pub fn open(&mut self, player: &mut Player<P>) {
        if let OpenContainer::None = self {
            return;
        }
        match self {
            OpenContainer::Inventory => {
                if player.inventory.dragged_item.is_some() { 
                    player.write_packet(&SetSlot {
                        window_id: -1,
                        slot: 0,
                        item_stack: get_item_stack(&player.inventory.dragged_item),
                    })
                }
            }
            OpenContainer::Menu(menu) => {
                let name = menu.container_name(player);
                let items = menu.container_items(player);
                debug_assert!(items.len() % 9 == 0);
                
                player.write_packet(&OpenWindow {
                    window_id: player.window_id,
                    inventory_type: "minecraft:container".into(),
                    window_title: ChatComponent::new(name),
                    slot_count: items.len() as u8,
                });
                player.write_packet(&WindowItems {
                    window_id: player.window_id,
                    items,
                })
            }
            _ => {}
        }
    }

    pub fn click_window(&mut self, player: &mut Player<P>, packet: &ClickWindow) {
        match self {
            OpenContainer::None => {
                player.sync_inventory()
            }
            OpenContainer::Inventory => {
                if player.inventory.handle_packet(packet, &mut player.packet_buffer) {
                    player.sync_inventory();
                    player.write_packet(&SetSlot {
                        window_id: -1,
                        slot: 0,
                        item_stack: get_item_stack(&player.inventory.dragged_item),
                    })
                }
            }
            OpenContainer::Menu(menu) => {
                if player.window_id != packet.window_id {
                    return;
                }
                menu.click_window(player, packet);
                self.sync_container(player);
            }
        }
    }
    
    pub fn sync_container(&mut self, player: &mut Player<P>) {
        match self {
            OpenContainer::Inventory => {
                // the rest of the inventory is synced in player
                player.write_packet(&SetSlot {
                    window_id: -1,
                    slot: 0,
                    item_stack: get_item_stack(&player.inventory.dragged_item),
                })
            }
            OpenContainer::Menu(menu) => {
                let items = menu.container_items(player);
                debug_assert!(items.len() % 9 == 0);
                
                player.write_packet(&WindowItems {
                    window_id: player.window_id,
                    items,
                });
                player.write_packet(&SetSlot {
                    window_id: -1,
                    slot: 0,
                    item_stack: None,
                })
            }
            _ => {}
        }
    }
}