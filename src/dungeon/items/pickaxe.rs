use crate::player::inventory::item_stack::ItemStack;
use crate::player::inventory::Item;
use indoc::indoc;

pub struct Pickaxe;

impl Item for Pickaxe {
    fn item_stack(&self) -> ItemStack {
        ItemStack::new()
            .item_id(278)
            .name("§9Diamond Pickaxe")
            .lore(indoc! {r#"
                §8Breaking Power 4

                §9Efficiency X
                §7Increases how quickly your tool
                §7breaks blocks.

                §9§l§kE§r§9§l RARE PICKAXE §kE
            "#})
            .enchant(32, 10)
            .hide_all_flags()
            .unbreakable()
    }
}