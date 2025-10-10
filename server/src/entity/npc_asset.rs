use crate::player::player::{GameProfile, GameProfileProperty};
use std::collections::HashMap;
use fstr::FString;
use uuid::Uuid;

pub struct NpcAsset {
    pub display_name: Option<FString>,
    pub texture: FString,
    pub signature: FString,
}

impl NpcAsset {

    pub fn get_profile(self) -> GameProfile {
        GameProfile {
            uuid: Uuid::new_v4(),
            username: self.display_name.unwrap_or(FString::EMPTY),
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
