use crate::core::player::sidebar::Sidebar;
use crate::core::player::Username;
use crate::dungeon::player::readying::ReadyStatus;
use crate::dungeon::player::update_room::CurrentRoom;
use crate::dungeon::rooms::room_data::RoomData;
use crate::dungeon::rooms::Room;
use crate::dungeon::DungeonState;
use bevy::prelude::{Entity, Query, Res, State};
use indoc::{formatdoc, indoc};
use std::time::{SystemTime, UNIX_EPOCH};

pub(super) fn update_sidebar(
    mut query: Query<(Entity, &mut Sidebar, &CurrentRoom)>,
    player_query: Query<(Entity, &Username, &ReadyStatus)>,
    room_query: Query<(&Room, &RoomData)>,
    state: Res<State<DungeonState>>,
) {
    for (client, mut sidebar, current_room) in query.iter_mut() {
        sidebar.push("§e§lSKYBLOCK");

        let now = chrono::Local::now();
        let date = now.format("%m/%d/%y").to_string();
        let time = now.format("%-I:%M%P").to_string();

        let room_id = if let Some(entity) = **current_room && let Ok((_, data)) = room_query.get(entity) {
            data.id.as_str()
        } else {
            ""
        };

        let (sb_month, sb_day, day_suffix) = get_sb_date();

        sidebar.push(&formatdoc! {r#"
            §7{date} §8local {room_id}

            {sb_month} {sb_day}{day_suffix}
            §7{time}
             §7⏣ §cThe Catacombs §7(F7)

        "#});

        match state.get() {
            DungeonState::NotStarted | DungeonState::Starting { .. } => {
                for (_, username, ready) in player_query.iter() {
                    let color = if ready.0 { 'a' } else { 'c' };
                    sidebar.push(&format!("§{color}[M] §7{}", username.0));
                }
                sidebar.new_line();
                if let DungeonState::Starting { starts_in_ticks } = state.get() {
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
                let score = 0;
                
                sidebar.push(&formatdoc! {r#"
                        Keys: §c■ {has_blood_key} §8■ §a{wither_key_count}x
                        Time elapsed: §a§a{time}
                        Cleared: §c{clear_percent}% §r§8({score})

                    "#,
                });

                if player_query.iter().len() == 1 {
                    sidebar.push(indoc! {r#"
                        §3§lSolo

                    "#});
                } else {
                    for (entity, username, _) in player_query.iter() {
                        if entity != client {
                            sidebar.push(&format!("§e[M] §7{}", username.0));
                        }
                    }
                    sidebar.new_line();
                }
            }
        }

        // todo: remove once, the display above hotbar is implemented
        if let Some(entity) = **current_room && let Ok((room, _)) = room_query.get(entity) {
            sidebar.push(&format!("secrets: {}/{}", room.found_secrets, room.secrets.len()));
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