use crate::network::packets::packet_serialize::PacketSerializable;
use bytes::BytesMut;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClickEvent {
    pub action: ClickAction,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HoverEvent {
    pub action: HoverAction,
    pub value: Box<ChatComponent>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ClickAction {
    OpenUrl,
    RunCommand,
    SuggestCommand,
    ChangePage,
    CopyToClipboard,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum HoverAction {
    ShowText,
    ShowItem,
    ShowEntity,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MCColors {
    Black,
    DarkBlue,
    DarkGreen,
    DarkCyan,
    DarkRed,
    DarkPurple,
    Gold,
    Gray,
    DarkGray,
    Blue,
    Green,
    Aqua,
    Red,
    LightPurple,
    Yellow,
    White,
    Reset,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatComponent {
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    color: Option<MCColors>,
    #[serde(skip_serializing_if = "Option::is_none")]
    bold: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    italic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    underlined: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    strikethrough: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    obfuscated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "extra")]
    siblings: Option<Vec<ChatComponent>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "clickEvent")]
    click_event: Option<ClickEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "hoverEvent")]
    hover_event: Option<HoverEvent>,
}

impl ChatComponent {
    
    pub fn new(string: impl Into<String>) -> Self {
        Self {
            text: string.into(),
            color: None,
            bold: None,
            italic: None,
            underlined: None,
            strikethrough: None,
            obfuscated: None,
            siblings: None,
            click_event: None,
            hover_event: None,
        }
    }
    
    #[inline(always)]
    pub const fn color(mut self, color: MCColors) -> Self {
        self.color = Some(color);
        self
    }

    #[inline(always)]
    pub const fn bold(mut self) -> Self {
        self.bold = Some(true);
        self
    }

    #[inline(always)]
    pub const fn italic(mut self) -> Self {
        self.italic = Some(true);
        self
    }

    #[inline(always)]
    pub const fn underlined(mut self) -> Self {
        self.underlined = Some(true);
        self
    }
    
    #[inline(always)]
    pub const fn strikethrough(mut self) -> Self {
        self.strikethrough = Some(true);
        self
    }

    #[inline(always)]
    pub const fn obfuscated(mut self) -> Self {
        self.obfuscated = Some(true);
        self
    }

    #[inline(always)]
    pub fn on_click(mut self, action: ClickAction, value: impl Into<String>) -> Self {
        self.click_event = Some(ClickEvent {
            action,
            value: value.into(),
        });
        self
    }

    #[inline(always)]
    pub fn on_hover(mut self, action: HoverAction, value: ChatComponent) -> Self {
        self.hover_event = Some(HoverEvent {
            action,
            value: Box::new(value),
        });
        self
    }

    #[inline(always)]
    pub fn append(mut self, component: ChatComponent) -> Self {
        if let Some(siblings) = &mut self.siblings {
            siblings.push(component);
        } else {
            self.siblings = Some(vec![component]);
        }
        self
    }
    
    pub fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl PacketSerializable for ChatComponent {
    fn write_size(&self) -> usize {
        self.serialize().write_size()
    }
    fn write(&self, buf: &mut BytesMut) {
        self.serialize().write(buf);
    }
}