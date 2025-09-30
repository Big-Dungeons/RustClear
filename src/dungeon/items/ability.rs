use crate::constants::Sound;
use crate::dungeon::dungeon_player::DungeonPlayer;
use crate::network::protocol::play::clientbound::PositionLook;
use crate::player::player::Player;
use glam::DVec3;

#[derive(Copy, Clone)]
pub struct Cooldown {
    pub ticks_remaining: usize,
    pub silent: bool,
}

impl Cooldown {
    
    pub fn from_ticks(ticks: usize, silent: bool) -> Self {
        Self {
            ticks_remaining: ticks,
            silent
        }
    }
    
    pub fn from_seconds(seconds: usize, silent: bool) -> Self {
        Self {
            ticks_remaining: seconds * 20,
            silent,
        }
    }
    
}

pub struct ActiveAbility {
    pub ability: Ability,
    pub ticks_active: usize,
}

pub enum Ability {
    TacticalInsertion {
        position: DVec3,
        yaw: f32,
        pitch: f32,
    }
}

impl Ability {
    pub fn tick(&mut self, ticks_active: usize, player: &mut Player<DungeonPlayer>) {
        match self {
            Ability::TacticalInsertion { position, yaw, pitch } => {
                let (sound, volume, sound_pitch) = match ticks_active {
                    1 => (Some(Sound::ZombieRemedy), 1.0, 0.75),
                    2 => (Some(Sound::ZombieRemedy), 1.0, 1.1),

                    10 => (Some(Sound::NoteHat), 0.8, 1.20),
                    20 => (Some(Sound::NoteHat), 0.85, 1.35),
                    30 => (Some(Sound::NoteHat), 0.85, 1.45),
                    40 => (Some(Sound::NoteHat), 0.9, 1.55),
                    50 => (Some(Sound::NoteHat), 1.0, 1.7),

                    // todo: correct these, these sound off by quite a lot
                    61 => (Some(Sound::ZombieRemedy), 0.7, 1.9),
                    63 => (Some(Sound::ZombieRemedy), 0.6, 1.75),
                    66 => (Some(Sound::ZombieRemedy), 0.5, 1.55),
                    _ => (None, 0.0, 0.0),
                };

                if let Some(sound) = sound {
                    player.play_sound(sound, volume, sound_pitch)
                }

                if ticks_active == 60 {
                    player.write_packet(&PositionLook {
                        x: position.x,
                        y: position.y,
                        z: position.z,
                        yaw: *yaw,
                        pitch: *pitch,
                        flags: 0,
                    })
                }
            }
        }
    }

    pub fn duration(&self) -> usize {
        match self {
            Ability::TacticalInsertion { .. } => 70,
        }
    }
}