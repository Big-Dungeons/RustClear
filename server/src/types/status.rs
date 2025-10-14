use bytes::Bytes;
use crate::types::chat_component::ChatComponent;

pub struct Status {
    players: u32,
    max_players: u32,

    serialized_info: String,
    icon: &'static str,

    cached: Option<StatusBytes>,
}

impl Status {
    pub fn new(players: u32, max_players: u32, info: ChatComponent, icon: &'static str) -> Self {
        Self { players, max_players, serialized_info: info.serialize(), icon, cached: None, }
    }

    pub fn set(&mut self, update: StatusUpdate) {
        match update {
            StatusUpdate::Players(count) => self.players = count,
            StatusUpdate::MaxPlayers(count) => self.max_players = count,
            StatusUpdate::Info(component) => self.serialized_info = component.serialize(),
            StatusUpdate::Icon(icon_data) => self.icon = icon_data,
        }
        self.cached = None;
    }

    pub fn get(&mut self) -> StatusBytes {
        self.cached.get_or_insert_with(|| {
            StatusBytes(Bytes::from(format!(r#"{{
                "version": {{
                    "name": "1.8.9",
                    "protocol": 47
                }},
                "players": {{
                    "max": {},
                    "online": {}
                }},
                "description": {},
                "favicon": "data:image/png;base64,{}"
            }}"#, self.max_players, self.players, self.serialized_info, self.icon)))
        }).clone()
    }
}

pub enum StatusUpdate {
    Players(u32),
    MaxPlayers(u32),
    Info(ChatComponent),
    Icon(&'static str),
}

#[derive(Debug, Clone)]
pub struct StatusBytes(Bytes);
impl StatusBytes {
    #[inline(always)]
    pub fn get_str(&self) -> &str {
        // SAFETY: These should always be constructed from a String.
        unsafe { str::from_utf8_unchecked(&self.0) }
    }
}
