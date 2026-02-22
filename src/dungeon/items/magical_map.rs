use crate::dungeon::dungeon_player::DungeonPlayer;
use crate::dungeon::items::dungeon_items::DungeonItem;
use indoc::indoc;
use server::inventory::item_stack::ItemStack;
use server::network::binary::nbt::NBT;
use server::player::packet_processing::BlockInteractResult;
use server::Player;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct MagicalMap;

impl DungeonItem for MagicalMap {
    fn on_interact(&self, _: &mut Player<DungeonPlayer>, _: Option<BlockInteractResult>) {}

    fn item_stack(&self) -> ItemStack {
        ItemStack {
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
        }
    }

    fn can_move_in_inv(&self) -> bool {
        false
    }
}