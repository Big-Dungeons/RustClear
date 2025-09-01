use crate::server::items::ether_transmission::handle_teleport;
use crate::server::items::etherwarp::handle_ether_warp;
use crate::server::items::item_stack::ItemStack;
use crate::server::player::player::Player;
use crate::server::utils::nbt::nbt::{NBTNode, NBT};
use crate::server::utils::nbt::serialize::TAG_COMPOUND_ID;
use indoc::indoc;
use std::collections::HashMap;

mod etherwarp;
pub mod item_stack;
mod ether_transmission;

/// List of items available to be used
#[derive(Copy, Debug, Clone, PartialEq)]
pub enum Item {
    SkyblockMenu,
    MagicalMap,
    AspectOfTheVoid,
    DiamondPickaxe,
    SpiritSceptre,
}

impl Item {

    pub fn on_right_click(&self, player: &mut Player) -> anyhow::Result<()> {
        match self {
            Item::AspectOfTheVoid => {
                if player.is_sneaking {
                    handle_ether_warp(player, player.world_mut());
                } else {
                    handle_teleport(player);
                }
            }
            Item::SpiritSceptre => {
                // spawn bats, they copy yaw and pitch of player, idk the speed or whatever but
                // when they hit a solid block they blow up in like 10 block radius (or square) or something
            }
            _ => {}
        }
        Ok(())
    }
    
    /// creates a vanilla item stack, including all nbt data.
    /// 
    /// this is only used for packets, we do not need to store this on the server.
    pub fn get_item_stack(&self) -> ItemStack {
        let mut stack = match self {
            Item::SkyblockMenu => ItemStack {
                item: 399,
                stack_size: 1,
                metadata: 0,
                tag_compound: Some(NBT::with_nodes(vec![
                    NBT::compound("display", vec![
                        NBT::string("Name", "§aSkyBlock Menu §7(Click)"),
                        NBT::list_from_string("Lore", indoc! {r#"
                            §7View all of your SkyBlock progress,
                            §7including your Skills, Collections,
                            §7Recipes, and more!

                            §eClick to Open!
                        "#})
                    ]),
                ])),
            },
            Item::MagicalMap => ItemStack {
                item: 358,
                stack_size: 1,
                metadata: 1,
                tag_compound: None,
            },
            Item::AspectOfTheVoid => ItemStack {
                item: 277,
                stack_size: 1,
                metadata: 0,
                tag_compound: Some(NBT::with_nodes(vec![
                    NBT::compound("display", vec![
                        NBT::string("Name", "§6Aspect of the Void"),
                        NBT::list_from_string("Lore", indoc! {r#"

                            §6Ability: Ether Transmission §e§lSNEAK RIGHT CLICK
                            §7Teleport to your targeted block up
                            §7to §a61 blocks §7away
                            §8Soulflow Cost: §30
                            §8Mana Cost: §30

                            §6§l§kU§r§6§l LEGENDARY SWORD §kU
                        "#})
                    ]),
                    NBT::compound("ExtraAttributes", vec![
                        NBT::string("id", "ASPECT_OF_THE_VOID"),
                    ]),
                ])),
            },
            Item::DiamondPickaxe => ItemStack {
                item: 278,
                stack_size: 1,
                metadata: 0,
                tag_compound: Some(NBT::with_nodes(vec![
                    NBT::list("ench", TAG_COMPOUND_ID, vec![
                        NBTNode::Compound({
                            let mut map = HashMap::new();
                            map.insert("lvl".into(), NBTNode::Short(10));
                            map.insert("id".into(), NBTNode::Short(32));
                            map
                        })
                    ]),
                    NBT::compound("display", vec![
                        NBT::list_from_string("Lore", indoc! {r#"
                            §8Breaking Power 4

                            §9Efficiency X
                            §7Increases how quickly your tool
                            §7breaks blocks.

                            §9§l§kE§r§9§l RARE PICKAXE §kE
                        "#}),
                        NBT::string("Name", "§9Diamond Pickaxe"),
                    ]),
                ])),
            },
            Item::SpiritSceptre => ItemStack {
                item: 0,
                stack_size: 0,
                metadata: 0,
                tag_compound: None,
            },
        };
        if let Some(ref mut tag) = stack.tag_compound {
            tag.nodes.insert("Unbreakable".into(), NBTNode::Byte(1));
            tag.nodes.insert("HideFlags".into(), NBTNode::Int(127));
        }
        stack
    }
}
