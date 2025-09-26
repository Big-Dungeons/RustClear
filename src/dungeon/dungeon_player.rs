use crate::constants::potions::PotionEffect;
use crate::dungeon::dungeon::{Dungeon, DungeonState};
use crate::dungeon::items::dungeon_items::DungeonItem;
use crate::dungeon::room::room::Room;
use crate::inventory::item::get_item_stack;
use crate::inventory::item_stack::ItemStack;
use crate::network::protocol::play::clientbound::{AddEffect, BlockChange};
use crate::network::protocol::play::serverbound::PlayerDiggingAction;
use crate::player::packet_handling::BlockInteractResult;
use crate::player::player::{Player, PlayerExtension};
use crate::player::sidebar::Sidebar;
use crate::types::block_position::BlockPos;
use crate::types::direction::Direction;
use crate::world::world::World;
use chrono::Local;
use indoc::{formatdoc, indoc};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct DungeonPlayer {
    pub is_ready: bool,
    pub sidebar: Sidebar,
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

        player.update_sidebar();
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
                    // todo: msg and sound when try open with no key
                    if door.can_open(world) && door.contains(position) {
                        door.open(world)
                    }
                }
            }
        }
    }


    fn update_sidebar(&mut self) {
        // really scuffed icl
        
        let now = Local::now();
        let date = now.format("%m/%d/%y").to_string();
        let time = now.format("%-I:%M%P").to_string();

        let room_id = if let Some(room) = self.current_room() {
            &*room.data.id.clone()
        } else {
            ""
        };

        let (sb_month, sb_day, day_suffix) = get_sb_date();
        let sidebar = &mut self.extension.sidebar;

        sidebar.push(&*formatdoc! {r#"
                §e§lSKYBLOCK
                §7{date} §8local {room_id}

                {sb_month} {sb_day}{day_suffix}
                §7{time}
                 §7⏣ §cThe Catacombs §7(F7)

            "#,

        });

        let world = self.world();
        match &world.state {
            DungeonState::NotStarted | DungeonState::Starting { .. } => {
                // can't use one outside because of borrow checker
                let sidebar = &mut self.extension.sidebar;

                for player in world.players.iter() {
                    let color = if player.extension.is_ready { 'a' } else { 'c' };
                    sidebar.push(&*format!("§{color}[M] §7{}", player.profile.username));
                }
                sidebar.new_line();
                if let DungeonState::Starting { starts_in_ticks } = world.state {
                    sidebar.push(&*format!("Starting in: §a0§a:0{}", (starts_in_ticks / 20) + 1));
                    sidebar.new_line();
                }
            }
            DungeonState::Started { ticks } => {
                let sidebar = &mut self.extension.sidebar;
                
                // this is scuffed but it works
                let seconds = ticks / 20;
                let time = if seconds >= 60 {
                    let minutes = seconds / 60;
                    let seconds = seconds % 60;
                    format!(
                        "{}{}m{}{}s",
                        if minutes < 10 { "0" } else { "" },
                        minutes,
                        if seconds < 10 { "0" } else { "" },
                        seconds
                    )
                } else {
                    let seconds = seconds % 60;
                    format!("{}{}s", if seconds < 10 { "0" } else { "" }, seconds)
                };
                // TODO: cleared percentage
                // clear percentage is based on amount of tiles that are cleared.
                
                let (has_blood_key, wither_key_count) = (
                    if world.blood_key_count != 0 { "§a✓" } else { "§c✖" },
                    world.wither_key_count,
                );
                
                sidebar.push(&*formatdoc! {r#"
                        Keys: §c■ {has_blood_key} §8■ §a{wither_key_count}x
                        Time elapsed: §a§a{time}
                        Cleared: §c{clear_percent}% §8§8({score})
                        
                    "#,
                    clear_percent = "0",
                    score = "0",
                });
                
                if world.players.len() == 1 { 
                    sidebar.push(indoc! {r#"
                        §3§lSolo
                        
                    "#});
                } else {
                    for player in world.players.iter() {
                        if player.client_id != self.client_id {
                            sidebar.push(&*format!("§e[M] §7{}", player.profile.username));
                        }
                    }
                    sidebar.new_line();
                }
            }
        }

        self.extension.sidebar.flush(&mut self.packet_buffer);
    }
}

fn get_sb_date() -> (&'static str, u64, &'static str) {
    const SKYBLOCK_EPOCH_START_MILLIS: u64 = 1_559_829_300_000;
    const SKYBLOCK_YEAR_MILLIS: u64 = 124 * 60 * 60 * 1000;
    const SKYBLOCK_MONTH_MILLIS: u64 = SKYBLOCK_YEAR_MILLIS / 12;
    const SKYBLOCK_DAY_MILLIS: u64 = SKYBLOCK_MONTH_MILLIS / 31;

    const SKYBLOCK_MONTHS: [&str; 12] = [
        "Early Spring", "Spring", "Late Spring",
        "Early Summer", "Summer", "Late Summer",
        "Early Autumn", "Autumn", "Late Autumn",
        "Early Winter", "Winter", "Late Winter",
    ];

    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
    let elapsed = now.saturating_sub(SKYBLOCK_EPOCH_START_MILLIS);
    let day = (elapsed % SKYBLOCK_YEAR_MILLIS) / SKYBLOCK_DAY_MILLIS;
    let month = (day / 31) as usize;
    let day_of_month = (day % 31) + 1;

    let suffix = match day_of_month % 100 {
        11..=13 => "th",
        _ => match day_of_month % 10 {
            1 => "st",
            2 => "nd",
            3 => "rd",
            _ => "th",
        },
    };
    //sb_month = SKYBLOCK_MONTHS[month], day = day_of_month, day_suffix = suffix
    (SKYBLOCK_MONTHS[month], day_of_month, suffix)
}
