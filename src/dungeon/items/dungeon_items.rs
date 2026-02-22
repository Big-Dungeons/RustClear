use crate::dungeon::dungeon_player::DungeonPlayer;
use crate::dungeon::items::aspect_of_the_void::AspectOfTheVoid;
use crate::dungeon::items::ender_pearl::EnderPearl;
use crate::dungeon::items::hyperion::Hyperion;
use crate::dungeon::items::magical_map::MagicalMap;
use crate::dungeon::items::pickaxe::Pickaxe;
use crate::dungeon::items::skyblock_menu::SkyblockMenu;
use crate::dungeon::items::spirit_sceptre::SpiritSceptre;
use crate::dungeon::items::superboom::SuperboomTNT;
use crate::dungeon::items::tactical_insertion::TacticalInsertion;
use enum_dispatch::enum_dispatch;
use glam::IVec3;
use server::inventory::item::Item;
use server::inventory::item_stack::ItemStack;
use server::player::packet_processing::BlockInteractResult;
use server::Player;

#[enum_dispatch]
pub trait DungeonItem {

    fn on_interact(&self, player: &mut Player<DungeonPlayer>, block: Option<BlockInteractResult>);
    
    fn on_start_dig(&self, _player: &mut Player<DungeonPlayer>, _position: IVec3) {}

    fn item_stack(&self) -> ItemStack;

    fn can_move_in_inv(&self) -> bool {
        true
    }
}

#[enum_dispatch(DungeonItem)]
#[derive(Copy, Clone, Hash, Eq, PartialEq)]
pub enum DungeonItems {
    TacticalInsertion,
    AspectOfTheVoid,
    SpiritSceptre,
    SkyblockMenu,
    SuperboomTNT,
    MagicalMap,
    EnderPearl,
    Hyperion,
    Pickaxe,
}

impl Item for DungeonItems {
    fn get_item_stack(&self) -> ItemStack {
        self.item_stack()
    }
    fn can_move_in_inventory(&self) -> bool {
        self.can_move_in_inv()
    }
}
