use bevy::prelude::{Add, Component, On, Query};
use bytes::BytesMut;
use crate::network::packets::BytesMutExt;
use crate::network::protocol::play::clientbound::{DisplayScoreboard, ScoreboardObjective, Teams, TeamsAction, UpdateScore};
use crate::network::protocol::sized_string::SizedString;
use crate::network::protocol::var_int::VarInt;
use crate::player::PlayerPacketBuffer;

#[derive(Component, Default)]
pub struct Sidebar {
    objective_name: String,
    lines: Vec<SizedString<32>>,
    previous_lines: Vec<SizedString<32>>,
}

pub(super) fn init_sidebar_packets(
    event: On<Add, Sidebar>,
    mut query: Query<(&Sidebar, &mut PlayerPacketBuffer)>
) {
    let (sidebar, mut packet_buffer) = query
        .get_mut(event.entity)
        .expect("Added sidebar to an entity that isn't a player");

    packet_buffer.write_packet(&ScoreboardObjective {
        objective_name: SizedString::new(&sidebar.objective_name),
        objective_value: SizedString::new(&sidebar.objective_name),
        mode: 0,
        render_type: SizedString::new("integer"),
    });
    packet_buffer.write_packet(&DisplayScoreboard {
        position: 1,
        score_name: SizedString::new(&sidebar.objective_name),
    });
}

pub(super) fn flush_sidebar_packets(
    mut query: Query<(&mut Sidebar, &mut PlayerPacketBuffer)>
) {
    for (mut sidebar, mut packet_buffer) in query.iter_mut() {
        sidebar.flush_packets(&mut packet_buffer)
    }
}

impl Sidebar {

    pub fn new(objective_name: &str) -> Self {
        Self {
            objective_name: objective_name.to_string(),
            lines: Vec::new(),
            previous_lines: Vec::new(),
        }
    }

    pub fn push(&mut self, string: &str) -> &mut Self {
        self.lines.extend(string.lines().map(SizedString::new));
        self
    }

    pub fn new_line(&mut self) -> &mut Self {
        self.lines.push(SizedString::default());
        self
    }

    pub fn flush_packets(&mut self, packet_buffer: &mut BytesMut) {
        let old_length = self.previous_lines.len();
        let new_length = self.lines.len();
        let is_size_diff = new_length != old_length;

        if is_size_diff && old_length != 0 {
            for index in 1..old_length {
                let name = format!("{index}");
                let team = format!("team_{}", old_length - index);

                packet_buffer.write_packet(&UpdateScore {
                    name: hide_name(&name),
                    objective_value: SizedString::new(&self.objective_name),
                    value: VarInt(0),
                    action: VarInt(1),
                });
                packet_buffer.write_packet(&Teams {
                    name: SizedString::new(team),
                    display_name: SizedString::default(),
                    prefix: SizedString::default(),
                    suffix: SizedString::default(),
                    name_tag_visibility: SizedString::new("always"),
                    color: -1,
                    players: Vec::new(),
                    action: TeamsAction::Remove,
                    friendly_flags: 0,
                })
            }
        }

        for (index, line) in self.lines.iter().enumerate() {
            let previous = &self.previous_lines.get(index);

            if !is_size_diff && previous.is_some_and(|str| str == line) {
                continue;
            }

            if index == 0 {
                packet_buffer.write_packet(&ScoreboardObjective {
                    objective_name: SizedString::new(&self.objective_name),
                    mode: 2 /* update name */,
                    objective_value: line.clone(),
                    render_type: SizedString::new("integer"),
                })
            } else {
                let line_index = new_length - index;
                let name = format!("{index}");
                let team = format!("team_{line_index}");
                let (first_half, second_half) = split_string(line);

                if is_size_diff {
                    packet_buffer.write_packet(&Teams {
                        name: SizedString::new(&team),
                        display_name: SizedString::new(&team),
                        prefix: SizedString::default(),
                        suffix: SizedString::default(),
                        name_tag_visibility: SizedString::new("always"),
                        color: 15,
                        players: Vec::new(),
                        action: TeamsAction::Create,
                        friendly_flags: 3,
                    });
                    packet_buffer.write_packet(&UpdateScore {
                        name: hide_name(&name),
                        objective_value: SizedString::new(&self.objective_name),
                        value: VarInt(line_index as i32),
                        action: VarInt(0),
                    });
                }

                packet_buffer.write_packet(&Teams {
                    name: SizedString::new(&team),
                    display_name: SizedString::new(&team),
                    prefix: first_half,
                    suffix: second_half,
                    name_tag_visibility: SizedString::new("always"),
                    color: 15,
                    players: Vec::new(),
                    action: TeamsAction::Update,
                    friendly_flags: 3,
                });

                if is_size_diff {
                    packet_buffer.write_packet(&Teams {
                        name: SizedString::new(&team),
                        display_name: SizedString::new(&team),
                        prefix: SizedString::default(),
                        suffix: SizedString::default(),
                        name_tag_visibility: SizedString::new("always"),
                        color: -1,
                        players: vec![hide_name(&name)],
                        action: TeamsAction::AddPlayer,
                        friendly_flags: 0,
                    });
                }
            }
        }

        std::mem::swap(&mut self.lines, &mut self.previous_lines);
        self.lines.clear()
    }

}

fn hide_name(key: &str) -> SizedString<40> {
    let mut result = String::with_capacity(40);
    for char in key.chars() {
        result.push('§');
        result.push(char);
        result.push_str("§r")
    }
    SizedString::new(result)
}

// not optimal but i dont care enough right now
fn split_string(string: &str) -> (SizedString<16>, SizedString<16>) {
    let first = string.chars().take(16).collect::<String>().into();
    let second = {
        let mut result = string.chars().skip(16).collect::<String>();
        if let Some(c) = get_last_color_code(&first) {
            result = format!("§{}{}", c, result)
        }
        result.into()
    };
    (first, second)
}

fn get_last_color_code(string: &SizedString<16>) -> Option<char> {
    let mut chars = string.chars().rev().peekable();
    while let Some(c) = chars.next() {
        if c != '§' && chars.peek() == Some(&'§') {
            return Some(c)
        }
    }
    None
}