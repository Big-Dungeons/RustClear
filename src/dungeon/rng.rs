use bevy::prelude::{Deref, DerefMut, Resource};
use rapidhash::fast::SeedableState;
use rapidhash::rng::RapidRng;
use std::collections::{HashMap, HashSet};
use std::hash::BuildHasher;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct DHasher(SeedableState<'static>);

impl Default for DHasher {
    fn default() -> Self {
        Self(SeedableState::fixed())
    }
}

impl BuildHasher for DHasher {
    type Hasher = <SeedableState<'static> as BuildHasher>::Hasher;
    fn build_hasher(&self) -> Self::Hasher {
        self.0.build_hasher()
    }
}

// D = Deterministic
pub type DHashMap<K, V> = HashMap<K, V, DHasher>;
pub type DHashSet<T> = HashSet<T, DHasher>;

#[derive(Resource, Deref, DerefMut)]
pub struct SeededRng {
    pub rng: RapidRng
}

impl SeededRng {
    pub fn from_seed(seed: u64) -> Self {
        Self {
            rng: RapidRng::new(seed),
        }
    }
}