use crate::dungeon::dungeon_player::DungeonPlayer;
use crate::dungeon::items::ability::{Ability, Cooldown};
use crate::dungeon::items::etherwarp::etherwarp;
use crate::dungeon::items::instant_transmission::instant_transmission;
use indoc::indoc;
use server::inventory::item::Item;
use server::inventory::item_stack::ItemStack;
use server::network::binary::nbt::TAG_COMPOUND_ID;
use server::network::binary::nbt::{NBTNode, NBT};
use server::network::protocol::play::clientbound::Chat;
use server::Player;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub enum DungeonItem {
    TacticalInsertion,
    AspectOfTheVoid,
    SkyblockMenu,
    MagicalMap,
    Hyperion,
    Pickaxe,
}

impl Item for DungeonItem {

    fn get_item_stack(&self) -> ItemStack {
        match self {
            DungeonItem::TacticalInsertion => ItemStack {
                item: 369,
                stack_size: 1,
                metadata: 0,
                tag_compound: Some(NBT::with_nodes(vec![
                    NBT::compound("display", vec![
                        NBT::string("Name", "§6Tactical Insertion"),
                        NBT::list_from_string("Lore", indoc! {r#"
                            §6Ability: Gorilla Tactics §e§lRIGHT CLICK
                            §7Marks your location and teleport back there
                            §7after §a3s§7.

                            §6§l§kU§r§6§l LEGENDARY §kU
                        "#})
                    ]),
                    NBT::compound("ExtraAttributes", vec![
                        NBT::string("id", "TACTICAL_INSERTION"),
                    ]),
                ])),
            },
            DungeonItem::AspectOfTheVoid => ItemStack {
                item: 277,
                stack_size: 1,
                metadata: 0,
                tag_compound: Some(NBT::with_nodes(vec![
                    NBT::compound("display", vec![
                        NBT::string("Name", "§6Aspect of the Void"),
                        NBT::list_from_string("Lore", indoc! {r#"

                            §6Ability: Instant Transmission §e§lRIGHT CLICK
                            §7Teleport §a12 blocks §7ahead of you and
                            §7gain §a+50 §r✦ Speed §7for §a3 seconds.

                            §6Ability: Ether Transmission §e§lSNEAK RIGHT CLICK
                            §7Teleport to your targeted block up
                            §7to §a61 blocks §7away

                            §6§l§kU§r§6§l LEGENDARY SWORD §kU
                        "#})
                    ]),
                    NBT::compound("ExtraAttributes", vec![
                        NBT::string("id", "ASPECT_OF_THE_VOID"),
                    ]),
                    NBT::byte("Unbreakable", 1),
                    NBT::byte("HideFlags", 127),
                ])),
            },
            DungeonItem::SkyblockMenu => ItemStack {
                item: 399,
                stack_size: 1,
                metadata: 0,
                tag_compound: Some(NBT::with_nodes(vec![
                    NBT::compound("display", vec![
                        NBT::string("Name", "§aSkyBlock Menu"),
                        NBT::list_from_string("Lore", indoc! {r#"
                            §7View all of your SkyBlock progress,
                            §7including your Skills, Collections,
                            §7Recipes, and more!
                        "#})
                    ]),
                ])),
            },
            DungeonItem::MagicalMap => ItemStack {
                item: 358,
                stack_size: 1,
                metadata: 1,
                tag_compound: Some(NBT::with_nodes(vec![
                    NBT::compound("display", vec![
                        NBT::string("Name", "§bMagical Map"),
                        NBT::list_from_string("Lore", indoc! {r#"
                            §7Shows the layout of the Dungeon as
                            §7it is explored and completed.
                        "#})
                    ]),
                ])),
            },
            DungeonItem::Hyperion => ItemStack {
                item: 267,
                stack_size: 1,
                metadata: 0,
                tag_compound: Some(NBT::with_nodes(vec![
                    NBT::compound("display", vec![
                        NBT::list_from_string("Lore", indoc! {r#"

                            §aScroll Abilities:
                            §6Ability: Wither Impact §e§lRIGHT CLICK
                            §7Teleport §a10 blocks§7 ahead of you.
                            §7Then implode dealing §c10,000
                            §7damage to nearby enemies. Also
                            §7applies the wither shield scroll
                            §7ability reducing damage taken and
                            §7granting an absorption shield for §e5
                            §7seconds.

                            §d§l§kE§r§d§l MYTHIC DUNGEON SWORD §kE
                        "#}),
                        NBT::string("Name", "§dHyperion"),
                    ]),
                    NBT::byte("Unbreakable", 1),
                    NBT::byte("HideFlags", 127),
                ])),
            },
            DungeonItem::Pickaxe => if cfg!(feature = "dungeon-breaker") {
                ItemStack {
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

                            §9§l§kE§r§9§l RARE PICKAXE §kE
                        "#}),
                            NBT::string("Name", "§cDungeon Breaker"),
                        ]),
                        NBT::byte("Unbreakable", 1),
                        NBT::byte("HideFlags", 127),
                    ])),
                }
            } else {
                ItemStack {
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
                        NBT::byte("Unbreakable", 1),
                        NBT::byte("HideFlags", 127),
                    ])),
                }
            }
        }
    }

    fn can_move_in_inventory(&self) -> bool {
        !matches!(self, DungeonItem::SkyblockMenu | DungeonItem::MagicalMap)
    }
}

impl DungeonItem {
    pub fn on_right_click(&self, player: &mut Player<DungeonPlayer>) {
        if let Some(cooldown) = player.extension.item_cooldown(self) {
            if !cooldown.silent {
                player.write_packet(&Chat::new(
                    &format!("§cThis ability is on cooldown for {}s.", cooldown.ticks_remaining / 20)
                ));
            }
            return;
        }
        match self {
            DungeonItem::TacticalInsertion => {
                player.extension.add_item_ability(Ability::TacticalInsertion {
                    position: player.position,
                    yaw: player.yaw,
                    pitch: player.pitch,
                });
                player.extension.add_item_cooldown(&DungeonItem::TacticalInsertion, Cooldown::from_seconds(20, false))
            }
            DungeonItem::AspectOfTheVoid => {
                if player.is_sneaking {
                    etherwarp(player);
                } else {
                    instant_transmission(player, 12.0);
                }
            }
            DungeonItem::Hyperion => {
                // todo: sounds, particles
                // add some cooldown system
                // and every 5 seconds allow wither shield to be activated?
                // and also for the wither impact itself as it as a very short cooldown
                instant_transmission(player, 10.0);
            }
            _ => {}
        }
    }
}