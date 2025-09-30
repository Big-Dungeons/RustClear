use crate::network::packets::packet_serialize::PacketSerializable;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone)]
pub enum Sound {
    EnderDragonHit,
    ZombieRemedy,
    FireIgnite,
    NoteHat,
}

impl Sound {
    fn get_sound(&self) -> &'static str {
        match self {
            Sound::EnderDragonHit => "mob.enderdragon.hit",
            Sound::ZombieRemedy => "mob.zombie.remedy",
            Sound::FireIgnite => "fire.ignite",
            Sound::NoteHat => "note.hat",
        }
    }
}

impl PacketSerializable for Sound {
    fn write_size(&self) -> usize {
        self.get_sound().write_size()
    }
    fn write(&self, buf: &mut BytesMut) {
        self.get_sound().write(buf)
    }
}