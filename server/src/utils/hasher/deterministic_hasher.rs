use crate::utils::hasher::rapidhash::rapidhash_nano;
use std::collections::HashMap;
use std::hash::{BuildHasher, Hasher};
use std::u64;

use super::rapidhash::finish;

const DEFAULT_SEED: u64 = 0;

/// reimplementation of rapidhash nano from [GitHub](https://github.com/Nicoshev/rapidhash/blob/master/rapidhash.h#L432)
/// (nano because were basically never gonna be hashing more than 48 bytes)
/// probably overkill but its really fast and i was bored.
/// these hashes are NOT portable for primitive ints.
pub struct RapidHasher {
    state: u64,
}

/// A deterministic HashMap using RapidHash.
///
/// This is so seeded RNG choosing will actually be seeded and not affected by the random hashing of normal hash maps.
///
/// This should also be faster than the default so maybe it could be used in place of the default for anythign that doesnt need the added security of the default hashing.
pub type DeterministicHashMap<K, V> = HashMap<K, V, RapidHasher>;

impl Hasher for RapidHasher {
    #[inline(always)]
    fn finish(&self) -> u64 {
        self.state
    }

    #[inline(always)]
    fn write(&mut self, bytes: &[u8]) {
        self.state = rapidhash_nano(self.state, bytes)
    }

    #[inline(always)]
    fn write_u8(&mut self, i: u8) {
        let (a, b) = (((i as u64) << 45) | i as u64, i as u64);
        self.state = finish(a, b, self.state, 1);
    }

    #[inline(always)]
    fn write_u16(&mut self, i: u16) {
        let (hi, lo) = ((i >> 8) as u64, (i & 0xFF) as u64);
        self.state = finish((hi << 45) | lo, lo, self.state, 2);
    }

    #[inline(always)]
    fn write_u32(&mut self, i: u32) {
        self.state = write_32(self.state ^ 4, i);
    }

    #[inline(always)]
    fn write_u64(&mut self, i: u64) {
        self.state = write_64(self.state ^ 8, i);
    }

    #[inline(always)]
    fn write_u128(&mut self, i: u128) {
        let (a, b) = ((i >> 64) as u64, i as u64); // mask is done automatically by casting down
        self.state = finish(a, b, self.state ^ 16, 16);
    }

    #[inline(always)]
    fn write_usize(&mut self, i: usize) {
        self.state = if size_of::<usize>() == 4 {
            write_32(self.state ^ 4, i as u32)
        } else {
            write_64(self.state ^ 8, i as u64)
        };
    }
}

impl Default for RapidHasher {
    fn default() -> Self {
        Self {
            state: DEFAULT_SEED
        }
    }
}


impl BuildHasher for RapidHasher {
    type Hasher = Self;

    fn build_hasher(&self) -> Self::Hasher {
        Self::default()
    }
}

#[inline(always)]
const fn write_64(seed: u64, i: u64) -> u64 {
    finish(i >> 32, i & 0xFFFF_FFFF, seed, 8)
}

#[inline(always)]
const fn write_32(seed: u64, i: u32) -> u64 { // no mask stuff is needed since for 4 bits it just reads both as the same...
    finish(i as u64, i as u64, seed, 4)
}