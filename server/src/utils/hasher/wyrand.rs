use crate::utils::hasher::rapidhash::mix;
use rand::{RngCore, SeedableRng};
use std::num::Wrapping;

const ADD: u64 = 0x2d358dccaa6c78a5;
const XOR: u64 = 0x8bb84b93962eacc9;

/// WyRand PRNG
///
/// this is not particularly cryptographically secure, but it's really fast and pretty good as far as im aware.
///
/// from [GitHub](https://github.com/wangyi-fudan/wyhash/blob/master/wyhash.h#L151)
pub struct WyRandRNG {
    pub state: Wrapping<u64>,
}

impl WyRandRNG {
    #[inline]
    pub const fn from_u64(seed: u64) -> Self {
        Self { state: Wrapping(seed) }
    }

    #[inline(always)]
    fn next_u64_inner(&mut self) -> u64 {
        self.state += ADD;
        mix(self.state.0, self.state.0 ^ XOR)
    }
}

impl RngCore for WyRandRNG {
    #[inline(always)]
    fn next_u32(&mut self) -> u32 {
        self.next_u64_inner() as u32
    }

    #[inline(always)]
    fn next_u64(&mut self) -> u64 {
        self.next_u64_inner()
    }

    #[inline(always)]
    fn fill_bytes(&mut self, dst: &mut [u8]) {
        let mut i = 0;

        while i + 8 <= dst.len() {
            dst[i..i + 8].copy_from_slice(&self.next_u64_inner().to_le_bytes());
            i += 8
        }

        if i < dst.len() {
            let len = dst.len() - i;
            let next = self.next_u64_inner().to_le_bytes();
            dst[i..].copy_from_slice(&next[..len]);
        }
    }
}

impl SeedableRng for WyRandRNG {
    type Seed = [u8; 8];

    fn from_seed(seed: Self::Seed) -> Self {
        Self::from_u64(u64::from_le_bytes(seed))
    }

    fn seed_from_u64(seed: u64) -> Self {
        Self::from_u64(seed)
    }
}