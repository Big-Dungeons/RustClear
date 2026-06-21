use crate::core::chunk::chunk_grid::ChunkGrid;
use crate::core::network::packets::BytesMutExt;
use crate::core::network::protocol::play::clientbound::SoundEffect;
use crate::core::types::sound::Sound;
use bevy::prelude::{Message, MessageReader, ResMut};
use glam::DVec3;

#[derive(Message, Copy, Clone)]
pub struct LocalSound {
    pub sound: Sound,
    pub volume: f32,
    pub pitch: f32,
    pub position: DVec3,
}

pub(super) fn handle_local_sounds(
    mut events: MessageReader<LocalSound>,
    mut chunks: ResMut<ChunkGrid>,
) {
    // might be more optimal to collect the sounds into map of chunk position -> sound
    // then loop over and write packets

    for LocalSound { sound, volume, pitch, position } in events.read() {
        if let Some(chunk) = chunks.get_mut_from_position(*position) {
            chunk.packet_buffer.write_packet(&SoundEffect::new(*sound, *position, *volume, *pitch))
        }
    }
}