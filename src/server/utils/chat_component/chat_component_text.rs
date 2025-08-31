use crate::net::packets::packet_serialize::PacketSerializable;
use crate::server::utils::color::MCColors;
use bytes::BytesMut;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChatComponentText {
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
    siblings: Option<Vec<ChatComponentText>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "clickEvent")]
    click_event: Option<ClickEvent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "hoverEvent")]
    hover_event: Option<HoverEvent>,
}

impl ChatComponentText {
    pub fn serialize(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl PacketSerializable for ChatComponentText {
    fn write_size(&self) -> usize {
        self.serialize().write_size()
    }
    fn write(&self, buf: &mut BytesMut) {
        self.serialize().write(buf);
    }
}

/// Builder for ChatComponentText.
///
/// # Example:
/// ```
/// let chat = ChatComponentTextBuilder::new("RC")
///     .color(Color::Gold)
///     .bold()
///     .append(ChatComponentTextBuilder::new(" >> ").color(Color::Gray).build())
///     .append(
///         ChatComponentTextBuilder::new("Hello World!")
///             .color(Color::White)
///             .on_hover(HoverAction::ShowText,
///                 ChatComponentTextBuilder::new("Hello World!")
///                     .color(Color::Blue)
///                     .build()
///                 )
///             .build()
///         )
///     .build();
/// ```
pub struct ChatComponentTextBuilder {
    component: ChatComponentText,
}

impl ChatComponentTextBuilder {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            component: ChatComponentText {
                text: text.into(), /*.replace("&", "§")*/
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
    }

    pub const fn color(mut self, color: MCColors) -> Self {
        self.component.color = Some(color);
        self
    }

    pub const fn bold(mut self) -> Self {
        self.component.bold = Some(true);
        self
    }

    pub const fn italic(mut self) -> Self {
        self.component.italic = Some(true);
        self
    }

    pub const fn underlined(mut self) -> Self {
        self.component.underlined = Some(true);
        self
    }

    pub const fn strikethrough(mut self) -> Self {
        self.component.strikethrough = Some(true);
        self
    }

    pub const fn obfuscated(mut self) -> Self {
        self.component.obfuscated = Some(true);
        self
    }

    pub fn on_click(mut self, action: ClickAction, value: impl Into<String>) -> Self {
        self.component.click_event = Some(ClickEvent {
            action,
            value: value.into(),
        });
        self
    }

    pub fn on_hover(mut self, action: HoverAction, value: ChatComponentText) -> Self {
        self.component.hover_event = Some(HoverEvent {
            action,
            value: Box::new(value),
        });
        self
    }

    pub fn append(mut self, component: ChatComponentText) -> Self {
        if let Some(siblings) = &mut self.component.siblings {
            siblings.push(component);
        } else {
            self.component.siblings = Some(vec![component]);
        }
        self
    }

    pub fn build(self) -> ChatComponentText {
        self.component
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClickEvent {
    pub action: ClickAction,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HoverEvent {
    pub action: HoverAction,
    pub value: Box<ChatComponentText>,
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