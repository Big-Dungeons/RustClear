use crate::entity::entity_metadata_serializable::MetadataSerializable;
use crate::network::packets::packet_serialize::PacketSerializable;
use enumset::{EnumSet, EnumSetType};
use macros::entity_metadata_serializable;

#[derive(EnumSetType)]
pub enum SkinLayers {
    Cape,
    Jacket,
    LeftSleeve,
    RightSleeve,
    LeftPantsLeg,
    RightPantsLeg,
    Hat
}

entity_metadata_serializable! {
    #[derive(Copy, Clone)]
    pub struct PlayerMetadata {
        10 => pub layers: EnumSet<SkinLayers> = EnumSet::all(),
    }
}

entity_metadata_serializable! {
    #[derive(Debug, Copy, Clone)]
    pub enum EntityMetadata {
        Zombie {
            12 => pub is_baby: bool = false,
            13 => pub is_villager: bool = false,
        },
        Bat {
            16 => pub hanging: bool = false,
        }
    }
}


// entity_metadata_serializable! {
//     pub enum ObjectMetadata {
//         FallingBlock {
//
//         }
//     }
// }