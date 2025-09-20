use crate::network::packets::packet_serialize::PacketSerializable;
use bytes::BytesMut;


#[repr(i32)]
#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum Particle {
    Explosion = 0,
    LargeExplosion = 1,
    HugeExplosion = 2,
    FireworkSpark = 3,
    WaterBubble = 4,
    WaterSplash = 5,
    WaterWake = 6,
    Suspended = 7,
    SuspendedDepth = 8,
    Crit = 9,
    MagicCrit = 10,
    Smoke = 11,
    SmokeNormal = 12,
    Spell = 13,
    SpellInstant =14,
    SpellMob = 15,
    SpellMobAmbient = 16,
    SpellWitch = 17,
    WaterDrip = 18,
    LavaDrip = 19,
    VillagerAngry = 20,
    VillagerHappy = 21,
    TownAura = 22,
    Note = 23,
    Portal = 24,
    EnchantmentTable = 25,
    Flame = 26,
    Lava = 27,
    Footstep = 28,
    Cloud = 29,
    Redstone = 30,
    SnowballPoof = 31,
    SnowShovel = 32,
    Slime = 33,
    Heart = 34,
    Barrier = 35,
    ItemCrack = 36,
    BlockCrack = 37,
    BlockDust = 38,
    Droplet = 39,
    ItemTake = 40,
    MobAppearance = 41,
}

impl PacketSerializable for Particle {
    fn write_size(&self) -> usize {
        (*self as i32).write_size()
    }
    fn write(&self, buf: &mut BytesMut) {
        (*self as i32).write(buf)
    }
}