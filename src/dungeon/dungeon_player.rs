use crate::dungeon::dungeon::{Dungeon, DungeonState};
use crate::dungeon::items::ability::{Ability, ActiveAbility, Cooldown};
#[cfg(feature = "dungeon-breaker")]
use crate::dungeon::items::dungeon_breaker::dungeon_breaker_dig;
use crate::dungeon::items::dungeon_items::DungeonItem;
use crate::dungeon::room::room::Room;
use chrono::Local;
use glam::IVec3;
use indoc::{formatdoc, indoc};
use server::constants::PotionEffect;
use server::inventory::item::get_item_stack;
use server::inventory::item_stack::ItemStack;
use server::network::protocol::play::clientbound::{AddEffect, BlockChange, Chat};
use server::network::protocol::play::serverbound::PlayerDiggingAction;
use server::player::packet_handling::BlockInteractResult;
use server::player::sidebar::Sidebar;
use server::types::chat_component::ChatComponent;
use server::types::direction::Direction3D;
use server::{Player, PlayerExtension, World};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct DungeonPlayer {
    pub sidebar: Sidebar,
    pub is_ready: bool,

    pub current_room: Option<(Rc<RefCell<Room>>, Option<usize>)>,

    // maybe disallow multiple of the same,
    // however if you pair with cooldowns it should be fine
    pub active_abilities: Cell<Vec<ActiveAbility>>,
    pub cooldowns: HashMap<DungeonItem, Cooldown>,

    #[cfg(feature = "dungeon-breaker")]
    pub pickaxe_charges: usize,
    #[cfg(feature = "dungeon-breaker")]
    pub broken_blocks: Vec<(IVec3, server::block::blocks::Blocks, usize)>,

}

impl PlayerExtension for DungeonPlayer {
    type World = Dungeon;
    type Item = DungeonItem;

    fn tick(player: &mut Player<Self>) {
        if player.ticks_existed.is_multiple_of(60) {
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
                duration: 600,
                hide_particles: true,
            });
        }

        if player.ticks_existed.is_multiple_of(2) {
            DungeonPlayer::update_sidebar(player);
        }
        
        let mut abilities = player.active_abilities.take();
        abilities.retain_mut(|active| {
            active.ticks_active += 1;
            active.ability.tick(active.ticks_active, player);
            active.ticks_active != active.ability.duration()
        });
        player.active_abilities.set(abilities);

        player.cooldowns.retain(|_, cooldown| {
            cooldown.ticks_remaining -= 1;
            cooldown.ticks_remaining != 0
        });

        #[cfg(feature = "dungeon-breaker")]
        {
            if player.ticks_existed.is_multiple_of(20) {
                if player.extension.pickaxe_charges != 20 {
                    let min = std::cmp::min(20 - player.extension.pickaxe_charges, 2);
                    player.extension.pickaxe_charges += min;
                }
            }

            let chunk_grid = &mut player.world_mut().chunk_grid;
            player.extension.broken_blocks.retain_mut(|(position, block, ticks)| {
                *ticks -= 1;
                if *ticks == 0 {
                    // for last 3 seconds, every second plays a particle and sound
                    chunk_grid.set_block_at(*block, position.x, position.y, position.z);
                }
               *ticks != 0
            });
        }
    }

    fn dig(player: &mut Player<Self>, position: IVec3, action: &PlayerDiggingAction) {
        
        #[cfg(not(feature = "dungeon-breaker"))]
        {
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
                    DungeonPlayer::try_open_door(player, world, &position);
                }
                PlayerDiggingAction::FinishDestroyBlock => {
                    restore_block = true;
                }
                _ => {}
            }
            if restore_block {
                let block = player.world().chunk_grid.get_block_at(position.x, position.y, position.z);
                // println!("block {block:?}");
                player.write_packet(&BlockChange {
                    block_pos: position,
                    block_state: block.get_blockstate_id(),
                })
            }
        }

        #[cfg(feature = "dungeon-breaker")]
        {
            let world = player.world_mut();

            // try open doors with left click before continuing with dungeon breaker
            if DungeonPlayer::try_open_door(player, world, &position) {
                return;
            }

            if !dungeon_breaker_dig(player, world, position, action) {
                let block = world.chunk_grid.get_block_at(
                    position.x,
                    position.y,
                    position.z
                );
                player.write_packet(&BlockChange {
                    block_pos: position,
                    block_state: block.get_block_state_id(),
                })
            }
        }
    }

    fn interact(player: &mut Player<Self>, item: Option<ItemStack>, block: Option<BlockInteractResult>) {
        if let Some(block) = block {
            {
                let mut pos = block.position;
                match block.direction {
                    Direction3D::Down => pos.y -= 1,
                    Direction3D::Up => pos.y += 1,
                    Direction3D::North => pos.z -= 1,
                    Direction3D::South => pos.z += 1,
                    Direction3D::West => pos.x -= 1,
                    Direction3D::East => pos.x += 1,
                }
                let block = player.world().chunk_grid.get_block_at(pos.x, pos.y, pos.z);
                player.write_packet(&BlockChange {
                    block_pos: pos,
                    block_state: block.get_blockstate_id(),
                });
            }
            
            let world = player.world_mut();
            DungeonPlayer::try_open_door(player, world, &block.position);
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

impl DungeonPlayer {

    pub fn item_cooldown(&self, item: &DungeonItem) -> Option<&Cooldown> {
        self.cooldowns.get(item)
    }

    pub fn add_item_cooldown(&mut self, item: &DungeonItem, cooldown: Cooldown) {
        self.cooldowns.insert(*item, cooldown);
    }

    pub fn add_item_ability(&mut self, ability: Ability) {
        let active_ability = ActiveAbility {
            ability,
            ticks_active: 0,
        };
        self.active_abilities.get_mut().push(active_ability)
    }

    pub fn ready(player: &mut Player<Self>) {
        player.is_ready = !player.is_ready;
        Dungeon::update_ready_status(player.world_mut(), player);
    }

    pub fn get_current_room(&self) -> Option<Rc<RefCell<Room>>> {
        if let Some((room, _)) = &self.current_room {
            return Some(room.clone())
        }
        None
    }

    pub fn try_open_door(player: &mut Player<Self>, world: &mut World<Dungeon>, position: &IVec3) -> bool {
        if world.has_started() {
            if let Some(room_rc) = player.extension.get_current_room() {
                for neighbour in room_rc.borrow().neighbours() {
                    let mut door = neighbour.door.borrow_mut();

                    if !door.contains(position) || door.is_open {
                        continue;
                    }
                    if !door.can_open(world) {
                        // todo: proper chat message and sound
                        player.write_packet(&Chat {
                            component: ChatComponent::new("no key"),
                            chat_type: 0,
                        });
                        continue;
                    }
                    door.open(world);
                    drop(door);

                    neighbour.room.borrow_mut().discovered = true;
                    world.map.draw_room(&neighbour.room.borrow());
                    return true;
                }
            }
        }
        false
    }
    
    fn update_sidebar(player: &mut Player<DungeonPlayer>) {
        // really scuffed icl

        let now = Local::now();
        let date = now.format("%m/%d/%y").to_string();
        let time = now.format("%-I:%M%P").to_string();

        let room_id = if let Some(room_rc) = player.extension.get_current_room() {
            let room = room_rc.borrow();
            &*room.data.id.clone()
        } else {
            ""
        };

        let (sb_month, sb_day, day_suffix) = get_sb_date();
        let sidebar = &mut player.extension.sidebar;

        sidebar.push(&formatdoc! {r#"
                §e§lSKYBLOCK
                §7{date} §8local {room_id}

                {sb_month} {sb_day}{day_suffix}
                §7{time}
                 §7⏣ §cThe Catacombs §7(F7)

            "#,

        });

        let world = player.world();
        match &world.state {
            DungeonState::NotStarted | DungeonState::Starting { .. } => {
                // can't use one outside because of borrow checker
                let sidebar = &mut player.extension.sidebar;

                for player in world.players.iter() {
                    let color = if player.extension.is_ready { 'a' } else { 'c' };
                    sidebar.push(&format!("§{color}[M] §7{}", player.profile.username));
                }
                sidebar.new_line();
                if let DungeonState::Starting { starts_in_ticks } = world.state {
                    sidebar.push(&format!("Starting in: §a0§a:0{}", (starts_in_ticks / 20) + 1));
                    sidebar.new_line();
                }
            }
            DungeonState::Started { ticks } => {
                let sidebar = &mut player.extension.sidebar;

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

                sidebar.push(&formatdoc! {r#"
                        Keys: §c■ {has_blood_key} §8■ §a{wither_key_count}x
                        Time elapsed: §a§a{time}
                        Cleared: §c{clear_percent}% §8§8({score})

                    "#,
                    clear_percent = "0",
                    score = "0",
                });

                // temp
                #[cfg(feature = "dungeon-breaker")]
                sidebar.push(&formatdoc!{r#"
                    charges {charges}

                "#,
                    charges = player.extension.pickaxe_charges,
                });

                if world.players.len() == 1 {
                    sidebar.push(indoc! {r#"
                        §3§lSolo
                        
                    "#});
                } else {
                    for p in world.players.iter() {
                        if p.client_id != player.client_id {
                            sidebar.push(&format!("§e[M] §7{}", p.profile.username));
                        }
                    }
                    sidebar.new_line();
                }
            }
        }

        player.extension.sidebar.flush(&mut player.packet_buffer);
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
    // sb_month = SKYBLOCK_MONTHS[month], day = day_of_month, day_suffix = suffix
    (SKYBLOCK_MONTHS[month], day_of_month, suffix)
}
