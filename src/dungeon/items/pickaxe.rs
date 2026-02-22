use crate::dungeon::dungeon_player::DungeonPlayer;
use crate::dungeon::items::dungeon_items::DungeonItem;
use indoc::indoc;
use server::inventory::item_stack::ItemStack;
use server::network::binary::nbt::{NBTNode, NBT, TAG_COMPOUND_ID};
use server::player::packet_processing::BlockInteractResult;
use server::Player;
use std::collections::HashMap;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct Pickaxe;

impl DungeonItem for Pickaxe {

    fn on_interact(&self, _: &mut Player<DungeonPlayer>, _: Option<BlockInteractResult>) {
        // does nothing
    }

    fn item_stack(&self) -> ItemStack {
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

    fn can_move_in_inv(&self) -> bool {
        true
    }
}