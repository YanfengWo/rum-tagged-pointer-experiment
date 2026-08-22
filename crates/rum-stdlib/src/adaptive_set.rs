use std::collections::{HashSet, hash_map::RandomState};
use std::hash::{BuildHasher, Hash};

const INDEX_THRESHOLD: usize = 16;

pub(crate) enum AdaptiveSet<T> {
    Linear(Vec<T>),
    Indexed(HashSet<T>),
}

impl<T: Eq + Hash> AdaptiveSet<T> {
    pub(crate) fn new() -> Self {
        Self::Linear(Vec::new())
    }

    pub(crate) fn insert(&mut self, value: T) -> bool {
        match self {
            Self::Linear(values) => {
                if values.iter().any(|existing| existing == &value) {
                    return false;
                }
                values.push(value);
                if values.len() > INDEX_THRESHOLD {
                    let values = std::mem::take(values).into_iter().collect();
                    *self = Self::Indexed(values);
                }
                true
            }
            Self::Indexed(values) => values.insert(value),
        }
    }
}

impl<T: Eq + Hash> Default for AdaptiveSet<T> {
    fn default() -> Self {
        Self::new()
    }
}

pub(crate) struct StringFingerprintIndex {
    hashes: HashSet<u64>,
    hash_builder: RandomState,
}

impl StringFingerprintIndex {
    pub(crate) fn from_values<'a>(
        capacity: usize,
        values: impl IntoIterator<Item = &'a str>,
    ) -> Self {
        let hash_builder = RandomState::new();
        let mut hashes = HashSet::with_capacity(capacity);
        hashes.extend(values.into_iter().map(|value| hash_builder.hash_one(value)));
        Self {
            hashes,
            hash_builder,
        }
    }

    pub(crate) fn insert<'a>(
        &mut self,
        existing: impl IntoIterator<Item = &'a str>,
        value: &str,
    ) -> bool {
        let hash = self.hash_builder.hash_one(value);
        self.hashes.insert(hash) || !existing.into_iter().any(|item| item == value)
    }
}
