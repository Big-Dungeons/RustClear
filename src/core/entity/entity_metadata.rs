use crate::core::entity::entity_metadata_serializable::EntityMetadataSerializable;
use crate::core::network::packets::packet_serializable::PacketSerializable;
use crate::core::types::entity_variant::EntityVariant;
use bevy::ecs::component::Component;
use enumset::{EnumSet, EnumSetType};
use macros::entity_metadata;

#[derive(EnumSetType, Debug)]
pub enum SkinLayers {
    Cape,
    Jacket,
    LeftSleeve,
    RightSleeve,
    LeftPantsLeg,
    RightPantsLeg,
    Hat,
}

entity_metadata! {
    #[derive(Debug, Clone, Copy, Component)]
    pub enum EntityMetadata {
        Player {
            10 => pub layers: EnumSet<SkinLayers> = EnumSet::all(),
        },
        Zombie {
            12 => pub is_baby: bool = false,
            13 => pub is_villager: bool = false,
        },
        Bat {
            0 => pub flags: u8 = 0,
            16 => pub hanging: bool = false,
        }
    }
}

impl EntityMetadata {
    pub fn get_variant(&self) -> EntityVariant {
        match self {
            EntityMetadata::Player(_) => unreachable!(),
            EntityMetadata::Zombie(_) => EntityVariant::Zombie,
            EntityMetadata::Bat(_) => EntityVariant::Bat,
        }
    }
}
