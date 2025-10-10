use crate::player::player::{GameProfile, GameProfileProperty};
use std::collections::HashMap;
use uuid::Uuid;

pub struct NpcAsset {
    pub display_name: Option<String>,
    pub texture: String,
    pub signature: String,
}

impl NpcAsset {

    pub fn get_profile(self) -> GameProfile {
        GameProfile {
            uuid: Uuid::new_v4(),
            username: self.display_name.unwrap_or(String::new()),
            properties: HashMap::from([(
                "textures".into(),
                GameProfileProperty {
                    value: self.texture,
                    signature: Some(self.signature),
                },
            )]),
        }
    }

}
