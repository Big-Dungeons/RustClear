use crate::constants::potions::PotionEffect;
use crate::dungeon::dungeon::Dungeon;
use crate::dungeon::items::dungeon_items::DungeonItem;
use crate::dungeon::room::room::Room;
use crate::inventory::item::get_item_stack;
use crate::inventory::item_stack::ItemStack;
use crate::network::protocol::play::clientbound::{AddEffect, BlockChange};
use crate::network::protocol::play::serverbound::PlayerDiggingAction;
use crate::player::packet_handling::BlockInteractResult;
use crate::player::player::{Player, PlayerExtension};
use crate::types::block_position::BlockPos;
use crate::types::direction::Direction;
use crate::world::world::World;

pub struct DungeonPlayer {
    pub is_ready: bool
}

impl PlayerExtension for DungeonPlayer {
    type World = Dungeon;
    type Item = DungeonItem;

    fn tick(player: &mut Player<Self>) {
        if player.ticks_existed % 60 == 0 {
            player.write_packet(&AddEffect {
                entity_id: player.entity_id,
                effect_id: PotionEffect::Haste,
                amplifier: 2,
                duration: 200,
                hide_particles: true,
            });
            player.write_packet(&AddEffect {
                entity_id: player.entity_id,
                effect_id: PotionEffect::NightVision,
                amplifier: 0,
                duration: 200,
                hide_particles: true,
            });
        }
    }

    fn dig(player: &mut Player<Self>, position: BlockPos, action: &PlayerDiggingAction) {
        let mut restore_block = false;
        match action {
            PlayerDiggingAction::StartDestroyBlock => {
                if let Some(item) = *player.inventory.get_hotbar_slot(player.held_slot as usize) {
                    if matches!(item, DungeonItem::Pickaxe) { 
                        restore_block = true;
                    }
                }
                
                // only doors can be interacted with left click I think
                let world = player.world_mut();
                player.try_open_door(world, &position);
            }
            PlayerDiggingAction::FinishDestroyBlock => {
                restore_block = true;
            }
            _ => {}
        }
        if restore_block { 
            let block = player.world().chunk_grid.get_block_at(position.x, position.y, position.z);
            player.write_packet(&BlockChange {
                block_pos: position,
                block_state: block.get_block_state_id(),
            })
        }
    }

    fn interact(player: &mut Player<Self>, item: Option<ItemStack>, block: Option<BlockInteractResult>) {
        if let Some(block) = block {
            {
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
            }
            
            let world = player.world_mut();
            player.try_open_door(world, &block.position);
        }

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

    pub fn ready(&mut self) {
        self.extension.is_ready = !self.extension.is_ready;
        self.world_mut().update_ready_status(self);
    }
    
    // this functions is mostly a test
    pub fn current_room(&self) -> Option<&Room> {
        let world = self.world();

        if let Some((index, _)) = world.get_player_room(self) {
            let room = &world.rooms[index];
            return Some(room)
        }
        None
    }
    
    pub fn try_open_door(&self, world: &mut World<Dungeon>, position: &BlockPos) {
        if world.has_started() {
            if let Some(room) = self.current_room() {
                for neighbour in room.neighbours() {
                    let door = unsafe { &mut world.extension_mut().doors[neighbour.door_index] };
                    if !door.is_open && door.contains(position) {
                        door.open(world)
                    }
                }
            }
        }
    }
    
}