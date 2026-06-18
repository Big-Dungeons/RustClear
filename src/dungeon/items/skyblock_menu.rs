use crate::core::player::inventory::item_stack::ItemStack;
use crate::core::player::inventory::Item;
use indoc::indoc;

pub struct SkyblockMenu;

impl Item for SkyblockMenu {
    fn item_stack(&self) -> ItemStack {
        ItemStack::new()
            .item_id(399)
            .name("§aSkyBlock Menu")
            .lore(indoc! {r#"
                §7View all of your SkyBlock progress,
                §7including your Skills, Collections,
                §7Recipes, and more!
            "#})
    }
}
