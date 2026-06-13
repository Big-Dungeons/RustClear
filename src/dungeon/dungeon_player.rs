use crate::block::block_rotation::Rotate;
use crate::dungeon::items::etherwarp::{use_aspect_of_the_void, AspectOfTheVoid};
use crate::dungeon::items::pickaxe::Pickaxe;
use crate::dungeon::items::skyblock_menu::SkyblockMenu;
use crate::dungeon::items::DungeonItem;
use crate::dungeon::rooms::Room;
use crate::dungeon::{DungeonState, EntranceRoom};
use crate::entity::components::transform::Transform;
use crate::network::packets::BytesMutExt;
use crate::network::protocol::play::clientbound::Chat;
use crate::player::inventory::Inventory;
use crate::player::sidebar::Sidebar;
use crate::player::{GlobalPacketBuffer, Player, Username};
use crate::TEST_WORLD;
use bevy::app::{App, Update};
use bevy::prelude::{Add, Commands, Component, Deref, Entity, EntityEvent, On, Plugin, Query, Res, ResMut};
use glam::ivec3;
use indoc::{formatdoc, indoc};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct DungeonPlayerPlugin;

impl Plugin for DungeonPlayerPlugin {
    fn build(&self, app: &mut App) {
        // no crash trying to access entrance despite it not existing
        if !TEST_WORLD {
            app.add_observer(init_player);
        }
        app
            .add_observer(add_items)
            .add_observer(on_player_ready)
            .add_systems(Update, (
                update_sidebar,
                use_aspect_of_the_void
            ));
    }
}

#[derive(Component, Deref)]
pub struct ReadyStatus(pub bool);

#[derive(EntityEvent)]
pub struct PlayerReadyEvent {
    pub entity: Entity,
}

fn on_player_ready(
    event: On<PlayerReadyEvent>,
    mut player_query: Query<(Entity, &mut ReadyStatus, &Username)>,
    mut global: ResMut<GlobalPacketBuffer>,
    mut state: ResMut<DungeonState>,
) {
    assert!(!matches!(*state, DungeonState::Started { .. }), "tried to ready up when dungeon has already started");
    let mut should_start = true;

    for (entity, mut ready, username) in player_query.iter_mut() {
        if entity == event.entity {
            let is_ready = !ready.0;
            let message = format!("§7{} {}!", username.0, if is_ready { "§ais now ready" } else { "§cis no longer ready" });
            global.write_packet(&Chat::new(&message));
            *ready = ReadyStatus(is_ready);
        }

        if !ready.0 {
            should_start = false
        }
    }

    if should_start {
        *state = DungeonState::Starting { starts_in_ticks: 100 }
    } else {
        *state = DungeonState::NotStarted
    }
}

fn init_player(
    event: On<Add, Player>,
    mut player_query: Query<&mut Transform>,
    room_query: Query<&Room>,
    entrance_room: Res<EntranceRoom>,
    mut commands: Commands
) {
    let mut transform = player_query.get_mut(event.entity).unwrap();

    let room = room_query.get(entrance_room.entity).unwrap();
    let mut position = room.relative_to_world(ivec3(15, 72, 18)).as_dvec3();
    position.x += 0.5;
    position.z += 0.5;
    let yaw = 180.0.rotate(room.rotation);

    *transform = Transform {
        position,
        yaw,
        pitch: 0.0,
    };

    commands
        .entity(event.entity)
        .insert((
            Sidebar::new("SBScoreboard"),
            ReadyStatus(false)
        ));
}

fn add_items(event: On<Add, Player>, mut query: Query<&mut Inventory>) {
    let mut inventory = query.get_mut(event.entity).unwrap();
    inventory.set_slot(37, Some(DungeonItem::from(AspectOfTheVoid)));
    inventory.set_slot(39, Some(DungeonItem::from(Pickaxe)));
    inventory.set_slot(44, Some(DungeonItem::from(SkyblockMenu)));
}

fn update_sidebar(
    mut sidebar_query: Query<(Entity, &mut Sidebar)>,
    player_query: Query<(Entity, &Username, &ReadyStatus)>,
    state: Res<DungeonState>,
) {
    for (client, mut sidebar) in sidebar_query.iter_mut() {
        sidebar.push("§e§lSKYBLOCK");

        let now = chrono::Local::now();
        let date = now.format("%m/%d/%y").to_string();
        let time = now.format("%-I:%M%P").to_string();

        // todo:
        let room_id = "";
        let (sb_month, sb_day, day_suffix) = get_sb_date();

        sidebar.push(&formatdoc! {r#"
            §7{date} §8local {room_id}

            {sb_month} {sb_day}{day_suffix}
            §7{time}
             §7⏣ §cThe Catacombs §7(F7)

        "#});

        match *state {
            DungeonState::NotStarted | DungeonState::Starting { .. } => {
                for (_, username, ready) in player_query.iter() {
                    let color = if ready.0 { 'a' } else { 'c' };
                    sidebar.push(&format!("§{color}[M] §7{}", username.0));
                }
                sidebar.new_line();
                if let DungeonState::Starting { starts_in_ticks } = *state {
                    sidebar.push(&format!("Starting in: §a0§a:0{}", (starts_in_ticks / 20) + 1));
                    sidebar.new_line();
                }
            }
            DungeonState::Started { ticks } => {
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
                let (has_blood_key, wither_key_count) = (
                    // if world.blood_key_count != 0 { "§a✓" } else { "§c✖" },
                    // world.wither_key_count,
                    "§a✓", 0
                );

                let clear_percent = 0;
                sidebar.push(&formatdoc! {r#"
                        Keys: §c■ {has_blood_key} §8■ §a{wither_key_count}x
                        Time elapsed: §a§a{time}
                        Cleared: §c{clear_percent}% §r§8({score})

                    "#,
                    score = "0",
                });

                if player_query.iter().len() == 1 {
                    sidebar.push(indoc! {r#"
                        §3§lSolo

                    "#});
                } else {
                    for (entity, username, ready) in player_query.iter() {
                        if entity != client {
                            sidebar.push(&format!("§e[M] §7{}", username.0));
                        }
                    }
                    sidebar.new_line();
                }
            }
        }

        sidebar.push("§emc.hypixel.net");
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