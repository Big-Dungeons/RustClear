use crate::dungeon::dungeon::Dungeon;
use crate::dungeon::items::dungeon_items::DungeonItem;
use crate::dungeon::room::room::Room;
use crate::inventory::item::get_item_stack;
use crate::inventory::item_stack::ItemStack;
use crate::network::protocol::play::clientbound::BlockChange;
use crate::player::packet_handling::BlockInteractResult;
use crate::player::player::{Player, PlayerExtension};
use crate::types::direction::Direction;

pub struct DungeonPlayer {
    
}

impl PlayerExtension for DungeonPlayer {
    type World = Dungeon;
    type Item = DungeonItem;

    fn tick(player: &mut Player<Self>) {
    }

    fn interact(player: &mut Player<Self>, item: Option<ItemStack>, block: Option<BlockInteractResult>) {
        if let Some(block) = block {
            
            let mut pos = block.position;
            match block.direction {
                Direction::Down => pos.y -= 1,
                Direction::Up => pos.y += 1,
                Direction::North => pos.z -= 1,
                Direction::South => pos.z += 1,
                Direction::West => pos.x -= 1,
                Direction::East => pos.x += 1,
            }
            let block = player.world().chunk_grid.get_block_at(pos.x, pos.y, pos.z);
            player.write_packet(&BlockChange {
                block_pos: pos,
                block_state: block.get_block_state_id(),
            });
            
            // todo: handle right clicking doors, and secrets here
        }
        
        // needs clone, but it is cheap
        let held_item = *player.inventory.get_hotbar_slot(player.held_slot as usize);
        
        if get_item_stack(&held_item) != item {
            player.sync_inventory();
        }
        
        if let Some(held_item) = held_item {
            held_item.on_right_click(player)
        }
    }
}

impl Player<DungeonPlayer> {
    
    // this functions is mostly a test
    pub fn current_room(&mut self) -> Option<&Room> {
        if let Some((index, _)) = self.world().extension.get_player_room(self) { 
            let room = &self.world().extension.rooms[index];
            return Some(room)
        }
        None
    }
    
}