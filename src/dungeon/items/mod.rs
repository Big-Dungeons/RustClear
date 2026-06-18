pub mod etherwarp;
pub mod skyblock_menu;
pub mod pickaxe;

use crate::dungeon::items::etherwarp::AspectOfTheVoid;
use crate::dungeon::items::pickaxe::Pickaxe;
use crate::dungeon::items::skyblock_menu::SkyblockMenu;
use crate::core::player::inventory::item_stack::ItemStack;
use crate::core::player::inventory::Item;
use enum_dispatch::enum_dispatch;

#[enum_dispatch(Item)]
pub enum DungeonItem {
    AspectOfTheVoid,
    SkyblockMenu,
    Pickaxe,
}

pub fn get_item_stack(item: &Option<DungeonItem>) -> Option<ItemStack> {
    item.as_ref().map(|item| item.item_stack())
}