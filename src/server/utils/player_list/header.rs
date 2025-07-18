use crate::server::utils::chat_component::chat_component_text::{ChatComponentText, ChatComponentTextBuilder, HoverAction};
use crate::server::utils::color::MCColors;

pub fn header() -> ChatComponentText {
    ChatComponentTextBuilder::new("")
        .append(
            ChatComponentTextBuilder::new("You are playing on ")
                .color(MCColors::Aqua)
                .append(
                    ChatComponentTextBuilder::new("MC.HYPIXEL.NET")
                        .color(MCColors::Yellow)
                        .bold()
                        .on_hover(
                            HoverAction::ShowText,
                            ChatComponentTextBuilder::new("§c§lDo not trust unknown links!").build(),
                        )
                        .build()
                )
                .build()
        )
        .append(
            ChatComponentTextBuilder::new("\n").build()
        )
        .append(
            ChatComponentTextBuilder::new("")
                .append(
                    ChatComponentTextBuilder::new("§s").build()
                )
                .build()
        )
        .build()
}