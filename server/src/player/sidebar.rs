use crate::network::binary::var_int::VarInt;
use crate::network::packets::packet_buffer::PacketBuffer;
use crate::network::protocol::play::clientbound::{DisplayScoreboard, ScoreboardObjective, Teams, UpdateScore};
use crate::types::sized_string::SizedString;
use std::convert::Into;
use std::ops::Deref;

// todo rework

const OBJECTIVE_NAME: SizedString<16> = unsafe { SizedString::slice_truncated(*b"SBScoreboard") };

pub struct Sidebar {
    lines: Vec<SizedString<64>>,
    previous: Vec<SizedString<64>>,
}

impl Sidebar {
    
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            previous: Vec::new(),
        }
    }
    
    pub fn push(&mut self, string: &str) -> &mut Self {
        self.lines.extend(string.lines().map(|str| SizedString::truncated(str)));
        self
    }
    
    pub fn new_line(&mut self) -> &mut Self {
        self.lines.push(SizedString::EMPTY);
        self
    }

    pub fn write_init_packets(&self, packet_buffer: &mut PacketBuffer) {
        packet_buffer.write_packet(&ScoreboardObjective {
            objective_name: OBJECTIVE_NAME,
            objective_value: OBJECTIVE_NAME.deref().into(),
            mode: 0,
        });
        packet_buffer.write_packet(&DisplayScoreboard {
            position: 1,
            score_name: OBJECTIVE_NAME,
        });
    }
    
    pub fn flush(&mut self, packet_buffer: &mut PacketBuffer) {
        
        // still uses a lot of non-sized strings
        // i don't to write
        
        let old_len = self.previous.len();
        let new_len = self.lines.len();
        let is_size_different = new_len != old_len;

        if is_size_different && old_len != 0 {
            for (index, _) in self.previous.iter().enumerate() {
                if index == 0 {
                    continue
                }

                let name = format!("{}", index);
                let team = format!("team_{}", old_len - index);

                packet_buffer.write_packet(&UpdateScore {
                    name: hide_name(&name),
                    objective: OBJECTIVE_NAME,
                    value: VarInt(0),
                    action: VarInt(1),
                });
                packet_buffer.write_packet(&Teams {
                    name: team.into(),
                    display_name: SizedString::EMPTY,
                    prefix: SizedString::EMPTY,
                    suffix: SizedString::EMPTY,
                    name_tag_visibility: "always".into(),
                    color: -1,
                    players: vec![],
                    action: REMOVE_TEAM,
                    friendly_flags: 0,
                })
            }
        }

        for index in 0..new_len {

            let current_line = &self.lines[index];
            let previous_line = &self.previous.get(index);

            if !is_size_different && previous_line.is_some_and(|str| str == current_line) {
                continue;
            }
            
            // index 0 is header
            if index == 0 { 
                packet_buffer.write_packet(&ScoreboardObjective {
                    objective_name: OBJECTIVE_NAME,
                    objective_value: *current_line,
                    // render_type: "integer",
                    mode: UPDATE_NAME,
                })
            } else {
                let line_index = new_len - index;
                let name = format!("{}", index);
                let team = format!("team_{}", line_index);

                let (first_half, second_half) = split_string(current_line);

                if is_size_different {
                    packet_buffer.write_packet(&Teams {
                        name: SizedString::truncated(&team),
                        display_name: SizedString::truncated(&team),
                        prefix: SizedString::EMPTY,
                        suffix: SizedString::EMPTY,
                        name_tag_visibility: "always".into(),
                        color: 15,
                        players: vec![],
                        action: CREATE_TEAM,
                        friendly_flags: 3,
                    });
                    packet_buffer.write_packet(&UpdateScore {
                        name: hide_name(&name),
                        objective: OBJECTIVE_NAME,
                        value: VarInt(line_index as i32),
                        action: VarInt(0),
                    });
                }

                packet_buffer.write_packet(&Teams {
                    name: SizedString::truncated(&team),
                    display_name: SizedString::truncated(&team),
                    prefix: first_half,
                    suffix: second_half,
                    name_tag_visibility: "always".into(),
                    color: 15,
                    players: vec![],
                    action: UPDATE_TEAM,
                    friendly_flags: 3,
                });

                if is_size_different {
                    packet_buffer.write_packet(&Teams {
                        name: SizedString::truncated(&team),
                        display_name: SizedString::truncated(&team),
                        prefix: SizedString::EMPTY,
                        suffix: SizedString::EMPTY,
                        name_tag_visibility: "always".into(),
                        color: -1,
                        players: vec![hide_name(&name)],
                        action: ADD_PLAYER,
                        friendly_flags: 0,
                    });
                }
            }
        }
        
        std::mem::swap(&mut self.lines, &mut self.previous);
        self.lines.clear()
    }
}

fn hide_name(key: &str) -> SizedString<40> {
    debug_assert!(key.len() < 40, "hide_name key is too long");
    let mut result = String::new();
    for char in key.chars() {
        result.push('§');
        result.push(char);
        result.push_str("§r")
    }
    result.into()
}

fn split_string(string: &SizedString<64>) -> (SizedString<32>, SizedString<32>) {
    let mut first_half = String::with_capacity(32);
    let mut second_half = String::with_capacity(32);
    let mut last_char = None;
    let mut last_color_code = None;
    for (i, c) in string.chars().enumerate() {
        if i < 16 {
            if last_char == Some('§') {
                last_color_code = Some(c);
            }
            last_char = Some(c);
            first_half.push(c);
        } else {
            if let Some(last_code) = last_color_code {
                second_half.push('§');
                second_half.push(last_code);
                last_color_code = None;
            }
            second_half.push(c);
        }
    }

    (first_half.into(), second_half.into())
}

// for team packet:
const CREATE_TEAM: i8 = 0;
const REMOVE_TEAM: i8 = 1;
const UPDATE_TEAM: i8 = 2;
const ADD_PLAYER: i8 = 3;
// const REMOVE_PLAYER: i8 = 4;

// for scoreboard objective packet
// const ADD_OBJECTIVE: i8 = 0;
const UPDATE_NAME: i8 = 2;