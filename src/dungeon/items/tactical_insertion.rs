use crate::dungeon::dungeon_player::DungeonPlayer;
use crate::dungeon::items::ability::{Ability, Cooldown};
use crate::dungeon::items::dungeon_items::DungeonItem;
use indoc::indoc;
use server::inventory::item_stack::ItemStack;
use server::network::binary::nbt::NBT;
use server::player::packet_processing::BlockInteractResult;
use server::Player;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct TacticalInsertion;

impl DungeonItem for TacticalInsertion {
    
    fn on_interact(&self, player: &mut Player<DungeonPlayer>, _: Option<BlockInteractResult>) {
        // re-do cooldowns
        if let Some(cd) = player.item_cooldown(&TacticalInsertion.into()) {
            player.send_message(&format!("§cThis ability is on cooldown for {}s.", cd.ticks_remaining / 20));
            return;
        }
        player.extension.add_item_ability(Ability::TacticalInsertion {
            position: player.position,
            yaw: player.yaw,
            pitch: player.pitch,
        });
        player.extension.add_item_cooldown(&TacticalInsertion.into(), Cooldown::from_seconds(20, false))
    }

    fn item_stack(&self) -> ItemStack {
        ItemStack {
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
        }
    }
}
