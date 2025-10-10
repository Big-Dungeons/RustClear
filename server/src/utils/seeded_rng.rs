use crate::utils::hasher::wyrand::WyRandRNG;
use rand::RngCore;
use std::cell::UnsafeCell;
use std::rc::Rc;
// this can be moved to a value inside the dungeon type (since all rng should be per dungeon) if ever needed but itd need to be passed through to every function that needs it.
// it would also avoid the Rc overhead (and unsafecell but i think thats basically 0 overhead).
// technically doesnt need to be thread local since its only used in the main thread, but it does need it to call non const functions in the static...

// this is basically a 1:1 copy of rand::ThreadRng but with a seed, thus its logic of unsafeCell usage applies here as well.

/// A thread-local RNG that can be seeded.
///
/// All [SeededRNG]s on the same thread point to the same internal state, and thus will be affected by each other, including setting a seed.
#[derive(Clone)]
pub struct SeededRng {
    rng: Rc<UnsafeCell<WyRandRNG>>,
}

impl SeededRng {
    pub fn set_seed(seed: u64) {
        RNG_CORE.with(|rng| unsafe {
            *rng.get() = WyRandRNG::from_u64(seed);
        });
    }
}   

thread_local! {
    static RNG_CORE: Rc<UnsafeCell<WyRandRNG>> = Rc::new(UnsafeCell::new(WyRandRNG::from_u64(0)))
}

pub fn seeded_rng() -> SeededRng {
    SeededRng { rng: RNG_CORE.with(|rng| rng.clone()) }
}

impl RngCore for SeededRng {
    #[inline(always)]
    fn next_u32(&mut self) -> u32 {
        unsafe { &mut *self.rng.get() }.next_u32()
    }

    #[inline(always)]
    fn next_u64(&mut self) -> u64 {
        unsafe { &mut *self.rng.get() }.next_u64()
    }

    #[inline(always)]
    fn fill_bytes(&mut self, dest: &mut [u8]) {
        unsafe { &mut *self.rng.get() }.fill_bytes(dest)
    }
}