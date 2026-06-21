use crate::core::network::packets::packet_serializable::PacketSerializable;
use bytes::BytesMut;

#[derive(Debug, Copy, Clone)]
pub enum Sound {
    RandomWoodClick,
    EnderDragonHit,
    RandomExplode,
    GhastFireball,
    ZombieRemedy,
    FireIgnite,
    DonkeyHit,
    RandomOrb,
    RandomPop,
    NoteHarp,
    NoteHat,
}

impl Sound {
    fn get_sound(&self) -> &'static str {
        match self {
            Sound::RandomWoodClick => "random.wood_click",
            Sound::EnderDragonHit => "mob.enderdragon.hit",
            Sound::RandomExplode => "random.explode",
            Sound::GhastFireball => "mob.ghast.fireball",
            Sound::ZombieRemedy => "mob.zombie.remedy",
            Sound::FireIgnite => "fire.ignite",
            Sound::DonkeyHit => "mob.horse.donkey.hit",
            Sound::RandomOrb => "random.orb",
            Sound::RandomPop => "random.pop",
            Sound::NoteHarp => "note.harp",
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