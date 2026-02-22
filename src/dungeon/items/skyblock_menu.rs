use crate::dungeon::dungeon_player::DungeonPlayer;
use crate::dungeon::items::dungeon_items::DungeonItem;
use indoc::indoc;
use server::inventory::item_stack::ItemStack;
use server::network::binary::nbt::NBT;
use server::player::packet_processing::BlockInteractResult;
use server::Player;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct SkyblockMenu;

impl DungeonItem for SkyblockMenu {
    fn on_interact(&self, _: &mut Player<DungeonPlayer>, _: Option<BlockInteractResult>) {}

    fn item_stack(&self) -> ItemStack {
        ItemStack {
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
        }
    }

    fn can_move_in_inv(&self) -> bool {
        false
    }
}