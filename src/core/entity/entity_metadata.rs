use crate::core::entity::entity_metadata_serializable::EntityMetadataSerializable;
use crate::core::network::packets::packet_serializable::PacketSerializable;
use crate::core::types::entity_variant::EntityVariant;
use bevy::ecs::component::Component;
use enumset::{EnumSet, EnumSetType};
use glam::Vec3;
use macros::entity_metadata;

#[derive(EnumSetType, Debug)]
pub enum MobFlags {
    Burning = 0,
    Crouching = 1,
    Sprinting = 3,
    // Eating = 4,
    Invisible = 5,
}

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

#[derive(EnumSetType, Debug)]
pub enum ArmorStandFlags {
    Small,
    ShowArms,
    NoBasePlate,
    Marker,
}

entity_metadata! {
    #[derive(Debug, Clone, Copy, Component)]
    pub enum EntityMetadata {
        Player {
            10 => pub layers: EnumSet<SkinLayers> = EnumSet::all(),
        },
        ArmorStand {
            10 => pub armor_stand_flags: EnumSet<ArmorStandFlags> = EnumSet::default(),
            11 => pub head: Vec3 = Vec3::ZERO,
            12 => pub body: Vec3 = Vec3::ZERO,
            13 => pub left_arm: Vec3 = Vec3::new(-10.0, 0.0, -10.0),
            14 => pub right_arm: Vec3 = Vec3::new(-15.0, 0.0, 10.0),
            15 => pub left_leg: Vec3 = Vec3::new(-1.0, 0.0, -1.0),
            16 => pub right_leg: Vec3 = Vec3::new(1.0, 0.0, 1.0),
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
            EntityMetadata::ArmorStand(_) => EntityVariant::ArmorStand,
            EntityMetadata::Zombie(_) => EntityVariant::Zombie,
            EntityMetadata::Bat(_) => EntityVariant::Bat,
        }
    }
}