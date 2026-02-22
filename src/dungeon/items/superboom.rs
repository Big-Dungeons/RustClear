use crate::dungeon::dungeon_player::DungeonPlayer;
use crate::dungeon::items::ability::Cooldown;
use crate::dungeon::items::dungeon_items::DungeonItem;
use glam::IVec3;
use indoc::indoc;
use server::constants::Sound;
use server::inventory::item_stack::ItemStack;
use server::network::binary::nbt::NBT;
use server::player::packet_processing::BlockInteractResult;
use server::Player;

#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub struct SuperboomTNT;

impl DungeonItem for SuperboomTNT {

    fn on_interact(&self, player: &mut Player<DungeonPlayer>, block: Option<BlockInteractResult>) {
        if let Some(block) = block {
            if player.item_cooldown(&SuperboomTNT.into()).is_some() {
                return;
            }

            player.sync_inventory();
            player.play_sound_at(
                Sound::RandomExplode,
                1.0,
                0.8,
                block.position.as_dvec3()
            );

            player.add_item_cooldown(&SuperboomTNT.into(), Cooldown::from_ticks(10, true))
            // get current room, iterate over crypts and walls and explode
        }
    }

    fn on_start_dig(&self, player: &mut Player<DungeonPlayer>, position: IVec3) {
        if player.item_cooldown(&SuperboomTNT.into()).is_some() {
            return;
        }

        player.play_sound_at(
            Sound::RandomExplode,
            1.0,
            0.8,
            position.as_dvec3()
        );

        player.add_item_cooldown(&SuperboomTNT.into(), Cooldown::from_ticks(7, true))
    }

    fn item_stack(&self) -> ItemStack {
        ItemStack {
            item: 46,
            stack_size: 64,
            metadata: 0,
            tag_compound: Some(NBT::with_nodes(vec![
                NBT::compound("display", vec![
                    NBT::list_from_string("Lore", indoc! {r#"
                            §7Breaks weak walls. Can be used to
                            §7blow up Crypts in §cThe Catacombs and
                            §5Crystal Hollows§7.

                            §9§lRARE
                        "#}),
                    NBT::string("Name", "§9Superboom TNT"),
                ]),
            ])),
        }
    }
}