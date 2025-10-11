use crate::dungeon::dungeon_player::DungeonPlayer;
use server::inventory::item_stack::ItemStack;
use server::inventory::menu::Menu;
use server::network::binary::nbt::TAG_COMPOUND_ID;
use server::network::binary::nbt::{NBTNode, NBT};
use server::network::protocol::play::serverbound::ClickWindow;
use server::Player;
use std::collections::HashMap;

pub struct MortMenu;

impl Menu<DungeonPlayer> for MortMenu {
    fn container_name(&self, _: &mut Player<DungeonPlayer>) -> &str {
        "Ready Up"
    }

    fn container_items(&self, player: &mut Player<DungeonPlayer>) -> Vec<Option<ItemStack>> {
        // background
        let mut items = vec![
            Some(ItemStack {
                item: 160,
                stack_size: 1,
                metadata: 15,
                tag_compound: Some(NBT::with_nodes(vec![NBT::compound(
                    "display",
                    vec![NBT::string("Name", "")]
                )])),
            });
            54
        ];

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
                NBT::compound(
                    "display",
                    vec![
                        NBT::string("Name", &format!("§7{}", player.profile.username)),
                        NBT::list_from_string("Lore", &item_name.to_string()),
                    ],
                ),
                NBT::compound(
                    "SkullOwner",
                    vec![
                        NBT::string("Id", &player.profile.uuid.hyphenated().to_string()),
                        NBT::compound(
                            "Properties",
                            vec![NBT::list(
                                "textures",
                                TAG_COMPOUND_ID,
                                vec![NBTNode::Compound(HashMap::from([(
                                    "Value".into(),
                                    NBTNode::String(
                                        player.profile.properties["textures"].value.clone(),
                                    ),
                                )]))],
                            )],
                        ),
                    ],
                ),
            ])),
        });
        items[13] = Some(ItemStack {
            item: 95,
            stack_size: 1,
            metadata: color,
            tag_compound: Some(NBT::with_nodes(vec![NBT::compound(
                "display",
                vec![NBT::string("Name", item_name)],
            )])),
        });

        items
    }

    fn click_window(&mut self, player: &mut Player<DungeonPlayer>, packet: &ClickWindow) {
        match packet.slot_id {
            4 | 13 => DungeonPlayer::ready(player),
            // 49 => {
            // close
            // },
            _ => {}
        }
    }
}
