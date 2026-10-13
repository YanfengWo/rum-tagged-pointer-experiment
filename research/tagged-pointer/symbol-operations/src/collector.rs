//! Same adaptive algorithm as pinned runtime UniqueSymbolCollector; generic
//! observations and a shared RandomState seed are the disclosed adaptations.
use rum_core::Symbol;
use std::collections::{HashSet, hash_map::RandomState};
use std::hash::Hash;
pub trait Observation: Copy + Eq + Hash {
    fn resolve(self) -> Symbol;
}
impl Observation for Symbol {
    fn resolve(self) -> Symbol {
        self
    }
}
pub fn collect<K: Observation>(
    input: impl Iterator<Item = K>,
    hint: usize,
    seed: &RandomState,
) -> Vec<Symbol> {
    let mut symbols = Vec::<K>::new();
    let mut index: Option<HashSet<K, RandomState>> = None;
    for symbol in input {
        if let Some(index) = &mut index {
            if index.insert(symbol) {
                symbols.push(symbol);
            }
            continue;
        }
        if symbols.iter().any(|existing| *existing == symbol) {
            continue;
        }
        if symbols.len() >= 16 {
            let capacity = hint.max(symbols.len() + 1);
            let mut set = HashSet::with_capacity_and_hasher(capacity, seed.clone());
            set.extend(symbols.iter().copied());
            let inserted = set.insert(symbol);
            debug_assert!(inserted);
            index = Some(set);
        }
        symbols.push(symbol);
    }
    symbols.into_iter().map(Observation::resolve).collect()
}
