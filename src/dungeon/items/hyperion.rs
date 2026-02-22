use crate::dungeon::dungeon_player::DungeonPlayer;
use crate::dungeon::items::aspect_of_the_void::instant_transmission;
use crate::dungeon::items::dungeon_items::DungeonItem;
use indoc::indoc;
use server::inventory::item_stack::ItemStack;
use server::network::binary::nbt::NBT;
use server::player::packet_processing::BlockInteractResult;
use server::Player;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct Hyperion;

impl DungeonItem for Hyperion {

    fn on_interact(&self, player: &mut Player<DungeonPlayer>, _: Option<BlockInteractResult>) {
        // todo: the small cd it has, wither impact, sounds
        instant_transmission(player, 10.0)
    }

    fn item_stack(&self) -> ItemStack {
        ItemStack {
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
        }
    }

}