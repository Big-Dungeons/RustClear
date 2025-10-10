use std::time::Instant;

use bytes::Bytes;
use fstr::FString;

use crate::player::player::GameProfile;

pub enum RecordMessage {
    Start {
        seed: FString,
        rng_seed: u64,
        at: Instant,
    },
    Record {
        received: Instant,
        game_profile: GameProfile,
        packet: Bytes,
    },
    Save,
}