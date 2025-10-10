use fstr::FString;

use crate::dungeon::dungeon_player::DungeonPlayer;
use crate::inventory::item::get_item_stack;
use crate::inventory::item_stack::ItemStack;
use crate::network::binary::nbt::serialize::TAG_COMPOUND_ID;
use crate::network::binary::nbt::{NBTNode, NBT};
use crate::network::protocol::play::clientbound::{OpenWindow, SetSlot, WindowItems};
use crate::network::protocol::play::serverbound::ClickWindow;
use crate::player::player::{Player, PlayerExtension};
use crate::types::chat_component::ChatComponent;
use std::collections::HashMap;

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


// test
pub enum DungeonMenu {
    Mort,
}

impl Menu<DungeonPlayer> for DungeonMenu {
    
    fn container_name(&self, _: &mut Player<DungeonPlayer>) -> &str {
        match self {
            DungeonMenu::Mort => "Ready Up",
        }
    }

    fn container_items(&self, player: &mut Player<DungeonPlayer>) -> Vec<Option<ItemStack>> {
        match self {
            DungeonMenu::Mort => {
                // background
                let mut items = vec![
                    Some(ItemStack {
                        item: 160,
                        stack_size: 1,
                        metadata: 15,
                        tag_compound: Some(NBT::with_nodes(vec![
                            NBT::compound("display", vec![
                                NBT::string("Name", "")
                            ])
                        ])),
                    }); 
                54];

                let (item_name, color) = if player.extension.is_ready {
                    ("§aReady", 13)
                } else {
                    ("§cNot Ready", 14)
                };

                items[4] = Some(ItemStack {
                    item: 397,
                    stack_size: 1,
                    metadata: 3,
                    tag_compound: Some(NBT::with_nodes(vec![
                        NBT::compound("display", vec![
                            NBT::string("Name", &format!("§7{}", player.profile.username)),
                            NBT::list_from_string("Lore", &item_name.to_string())
                        ]),
                        NBT::compound("SkullOwner", vec![
                            NBT::string("Id", &player.profile.uuid.hyphenated().to_string()),
                            NBT::compound("Properties", vec![
                                NBT::list("textures", TAG_COMPOUND_ID, vec![
                                    NBTNode::Compound(HashMap::from([(
                                        "Value".into(),
                                        NBTNode::String(player.profile.properties["textures"].value.clone())
                                    )]))
                                ])
                            ])
                        ]),
                    ])),
                });
                items[13] = Some(ItemStack {
                    item: 95,
                    stack_size: 1,
                    metadata: color,
                    tag_compound: Some(NBT::with_nodes(vec![
                        NBT::compound("display", vec![
                            NBT::string("Name", item_name)
                        ])
                    ])),
                });
                
                items
            }
        }
    }

    fn click_window(&mut self, player: &mut Player<DungeonPlayer>, packet: &ClickWindow) {
        match packet.slot_id {
            4 | 13 => {
                player.ready()
            }
            // 49 => {
            // close
            // },
            _ => {}
        }
    }
}