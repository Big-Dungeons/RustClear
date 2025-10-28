use crate::network::packets::packet_serialize::PacketSerializable;
use bytes::BytesMut;

#[repr(i8)]
#[derive(Copy, Clone)]
pub enum PotionEffect {
    Speed = 1,
    Slowness = 2,
    Haste = 3,
    MiningFatigue = 4,
    Strength = 5,
    InstantHealth = 6,
    InstantDamage = 7,
    JumpBoost = 8,
    Nausea = 9,
    Regeneration = 10,
    Resistance = 11,
    FireResistance = 12,
    WaterBreathing = 13,
    Invisibility = 14,
    Blindness = 15,
    NightVision = 16,
    Hunger = 17,
    Weakness = 18,
    Poison = 19,
    Wither = 20,
    HealthBoost = 21,
    Absorption = 22,
    Saturation = 23,
}

impl PacketSerializable for PotionEffect {
    fn write_size(&self) -> usize {
        (*self as i8).write_size()
    }
    fn write(&self, buf: &mut BytesMut) {
        (*self as i8).write(buf)
    }
}