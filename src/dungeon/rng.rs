use rapidhash::fast::SeedableState;
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