use super::*;

use std::hash::{BuildHasher, BuildHasherDefault, Hash, Hasher};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct SymbolStabilityCacheEntry {
    pub(in crate::runtime) generation: u64,
    pub(in crate::runtime) stable: bool,
}

pub(in crate::runtime) type SymbolCountMap =
    HashMap<Symbol, u32, BuildHasherDefault<RuntimeCacheHasher>>;
pub type SymbolExprMap = HashMap<Symbol, Expr, BuildHasherDefault<RuntimeCacheHasher>>;

pub(in crate::runtime) type SymbolSet =
    std::collections::HashSet<Symbol, BuildHasherDefault<RuntimeCacheHasher>>;

pub(in crate::runtime) type DefinitionMap =
    HashMap<Symbol, SymbolDefinition, BuildHasherDefault<RuntimeCacheHasher>>;

pub(in crate::runtime) type BuiltinSpecMap =
    HashMap<Symbol, BuiltinSpec, BuildHasherDefault<RuntimeCacheHasher>>;

pub(in crate::runtime) type AttributeMap =
    HashMap<Symbol, AttributeSet, BuildHasherDefault<RuntimeCacheHasher>>;

pub(in crate::runtime) type SymbolStabilityCache =
    HashMap<Symbol, SymbolStabilityCacheEntry, BuildHasherDefault<RuntimeCacheHasher>>;

pub(in crate::runtime) type AttributeSetCache =
    HashMap<Symbol, AttributeSet, BuildHasherDefault<RuntimeCacheHasher>>;

pub(in crate::runtime) type BuiltinDispatchSpecCache =
    HashMap<Symbol, Option<BuiltinSpec>, BuildHasherDefault<RuntimeCacheHasher>>;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(in crate::runtime) struct SymbolCallEffectKey {
    pub(in crate::runtime) symbol: Symbol,
    pub(in crate::runtime) args_empty: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct CallEffectCacheEntry {
    pub(in crate::runtime) generation: u64,
    pub(in crate::runtime) has_effect: bool,
}

pub(in crate::runtime) type SymbolCallEffectCache =
    HashMap<SymbolCallEffectKey, CallEffectCacheEntry, BuildHasherDefault<RuntimeCacheHasher>>;

// `pub` only so the promoted `SymbolExprMap` alias has a fully-public interface;
// the private parent module keeps it unreachable outside the crate.
#[derive(Clone, Copy, Debug, Default)]
pub struct RuntimeCacheHasher(u64);

impl Hasher for RuntimeCacheHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut hash = if self.0 == 0 {
            0xcbf29ce484222325_u64
        } else {
            self.0
        };
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x100000001b3);
        }
        self.0 = hash;
    }

    fn write_u64(&mut self, value: u64) {
        self.0 = value;
    }
}

pub(in crate::runtime) type CallStabilityCache =
    HashMap<CallStabilityCacheKey, CallStabilityCacheEntry, BuildHasherDefault<RuntimeCacheHasher>>;

const ASSOCIATION_LOOKUP_INDEX_THRESHOLD: usize = 16;
// Bound retained side data independently of the number of Associations a
// session evaluates. Wider values keep the allocation-free borrowed scan.
const ASSOCIATION_LOOKUP_CACHE_MAX_ASSOCIATIONS: usize = 4;
const ASSOCIATION_LOOKUP_CACHE_MAX_KEYS: usize = 65_536;
const NO_ASSOCIATION_KEY_POSITION: usize = usize::MAX;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum AssociationLookupCacheProbe {
    Indexed(Option<usize>),
    Build,
    Scan,
}

#[derive(Debug, Default)]
pub(in crate::runtime) struct AssociationLookupCache {
    // A second lookup admits an index. The weak witness prevents pointer reuse
    // from producing a false hit without pinning the Association's COW data.
    pending: Option<WeakSharedArgs>,
    entries: Vec<AssociationLookupCacheEntry>,
    indexed_keys: usize,
    #[cfg(test)]
    work: AssociationLookupWork,
}

#[derive(Debug)]
struct AssociationLookupCacheEntry {
    args: WeakSharedArgs,
    index: AssociationKeyIndex,
}

#[derive(Debug)]
struct AssociationKeyIndex {
    // Store positions behind randomized structural hashes instead of cloning
    // keys. The explicit chain preserves correctness under hash collisions.
    hash_builder: std::collections::hash_map::RandomState,
    buckets: HashMap<u64, usize>,
    next_collision: Vec<usize>,
}

impl AssociationLookupCache {
    pub(in crate::runtime) fn probe(
        &mut self,
        args: &SharedArgs,
        key: &Expr,
    ) -> AssociationLookupCacheProbe {
        self.remove_stale_entries();

        if let Some(index) = self
            .entries
            .iter()
            .position(|entry| entry.args.ptr_eq(args))
        {
            let entry = self.entries.remove(index);
            self.indexed_keys = self.indexed_keys.saturating_sub(entry.index.len());
            let (position, comparisons) = entry.index.position(args, key);
            self.record_indexed_probe(comparisons);
            self.indexed_keys = self.indexed_keys.saturating_add(entry.index.len());
            self.entries.push(entry);
            return AssociationLookupCacheProbe::Indexed(position);
        }

        if args.len() <= ASSOCIATION_LOOKUP_INDEX_THRESHOLD
            || args.len() > ASSOCIATION_LOOKUP_CACHE_MAX_KEYS
        {
            return AssociationLookupCacheProbe::Scan;
        }

        if self
            .pending
            .as_ref()
            .is_some_and(|pending| pending.ptr_eq(args))
        {
            self.pending = None;
            return AssociationLookupCacheProbe::Build;
        }

        self.pending = Some(args.downgrade());
        AssociationLookupCacheProbe::Scan
    }

    pub(in crate::runtime) fn insert_index<'a>(
        &mut self,
        args: &SharedArgs,
        keys: impl Iterator<Item = &'a Expr>,
    ) {
        if args.len() <= ASSOCIATION_LOOKUP_INDEX_THRESHOLD
            || args.len() > ASSOCIATION_LOOKUP_CACHE_MAX_KEYS
        {
            return;
        }

        self.remove_stale_entries();
        while self.entries.len() >= ASSOCIATION_LOOKUP_CACHE_MAX_ASSOCIATIONS
            || self.indexed_keys.saturating_add(args.len()) > ASSOCIATION_LOOKUP_CACHE_MAX_KEYS
        {
            let evicted = self.entries.remove(0);
            self.indexed_keys = self.indexed_keys.saturating_sub(evicted.index.len());
        }

        let index = AssociationKeyIndex::new(keys, args.len());
        self.record_index_build(index.len());
        self.indexed_keys = self.indexed_keys.saturating_add(index.len());
        self.entries.push(AssociationLookupCacheEntry {
            args: args.downgrade(),
            index,
        });
    }

    pub(in crate::runtime) fn memory_proxy_bytes(&self) -> usize {
        self.entries
            .capacity()
            .saturating_mul(std::mem::size_of::<AssociationLookupCacheEntry>())
            .saturating_add(
                self.entries
                    .iter()
                    .map(|entry| entry.index.memory_proxy_bytes())
                    .sum::<usize>(),
            )
    }

    fn remove_stale_entries(&mut self) {
        let mut index = 0;
        while index < self.entries.len() {
            if self.entries[index].args.is_alive() {
                index += 1;
                continue;
            }
            let stale = self.entries.remove(index);
            self.indexed_keys = self.indexed_keys.saturating_sub(stale.index.len());
        }
        if self
            .pending
            .as_ref()
            .is_some_and(|pending| !pending.is_alive())
        {
            self.pending = None;
        }
    }

    #[cfg(test)]
    pub(in crate::runtime) fn record_linear_comparison(&mut self) {
        self.work.linear_comparisons += 1;
    }

    #[cfg(not(test))]
    #[inline(always)]
    pub(in crate::runtime) fn record_linear_comparison(&mut self) {}

    #[cfg(test)]
    fn record_indexed_probe(&mut self, comparisons: usize) {
        self.work.indexed_probes += 1;
        self.work.indexed_comparisons += comparisons;
    }

    #[cfg(not(test))]
    #[inline(always)]
    fn record_indexed_probe(&mut self, _comparisons: usize) {}

    #[cfg(test)]
    fn record_index_build(&mut self, keys: usize) {
        self.work.index_builds += 1;
        self.work.indexed_keys += keys;
    }

    #[cfg(not(test))]
    #[inline(always)]
    fn record_index_build(&mut self, _keys: usize) {}

    #[cfg(test)]
    pub(in crate::runtime) fn work(&self) -> AssociationLookupWork {
        self.work
    }
}

impl AssociationKeyIndex {
    fn new<'a>(keys: impl Iterator<Item = &'a Expr>, len: usize) -> Self {
        let hash_builder = std::collections::hash_map::RandomState::new();
        let mut buckets = HashMap::with_capacity(len);
        let mut next_collision = Vec::with_capacity(len);
        for (position, key) in keys.enumerate() {
            let hash = hash_builder.hash_one(key);
            let previous = buckets
                .insert(hash, position)
                .unwrap_or(NO_ASSOCIATION_KEY_POSITION);
            next_collision.push(previous);
        }
        Self {
            hash_builder,
            buckets,
            next_collision,
        }
    }

    fn position(&self, args: &SharedArgs, key: &Expr) -> (Option<usize>, usize) {
        let hash = self.hash_builder.hash_one(key);
        let Some(mut position) = self.buckets.get(&hash).copied() else {
            return (None, 0);
        };
        let mut comparisons = 0;

        while position != NO_ASSOCIATION_KEY_POSITION {
            let Expr::Call {
                args: rule_args, ..
            } = &args[position]
            else {
                debug_assert!(false, "cached association entries stay canonical rules");
                return (None, comparisons);
            };
            debug_assert_eq!(rule_args.len(), 2);
            comparisons += 1;
            if rule_args.get(0) == Some(key) {
                return (Some(position), comparisons);
            }
            position = self.next_collision[position];
        }

        (None, comparisons)
    }

    fn len(&self) -> usize {
        self.next_collision.len()
    }

    fn memory_proxy_bytes(&self) -> usize {
        self.buckets
            .capacity()
            .saturating_mul(std::mem::size_of::<(u64, usize)>())
            .saturating_add(
                self.next_collision
                    .capacity()
                    .saturating_mul(std::mem::size_of::<usize>()),
            )
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::runtime) struct AssociationLookupWork {
    pub(in crate::runtime) linear_comparisons: usize,
    pub(in crate::runtime) indexed_probes: usize,
    pub(in crate::runtime) indexed_comparisons: usize,
    pub(in crate::runtime) index_builds: usize,
    pub(in crate::runtime) indexed_keys: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct CallStabilityCacheKey {
    head: usize,
    args: usize,
}

impl Hash for CallStabilityCacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let head = self.head as u64;
        let args = self.args as u64;
        state.write_u64(head.rotate_left(17) ^ args.wrapping_mul(0x9e3779b97f4a7c15));
    }
}

impl CallStabilityCacheKey {
    pub(in crate::runtime) fn new(head: &SharedExpr, args: &SharedArgs) -> Self {
        Self {
            head: head.ptr_key(),
            args: args.ptr_key(),
        }
    }
}

#[derive(Clone, Debug)]
pub(in crate::runtime) struct CallStabilityCacheEntry {
    pub(in crate::runtime) generation: u64,
    pub(in crate::runtime) head: SharedExpr,
    pub(in crate::runtime) args: SharedArgs,
    pub(in crate::runtime) stable: bool,
    pub(in crate::runtime) bloom: StabilityBloom,
    pub(in crate::runtime) dirty_snapshot: StabilityBloom,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::runtime) struct StabilityBloom {
    pub(in crate::runtime) regular: u128,
    pub(in crate::runtime) generated: u128,
}

impl StabilityBloom {
    pub(in crate::runtime) fn regular(bit: u128) -> Self {
        Self {
            regular: bit,
            generated: 0,
        }
    }

    pub(in crate::runtime) fn generated(bit: u128) -> Self {
        Self {
            regular: 0,
            generated: bit,
        }
    }

    pub(in crate::runtime) fn union_assign(&mut self, rhs: Self) {
        self.regular |= rhs.regular;
        self.generated |= rhs.generated;
    }

    pub(in crate::runtime) fn intersects(self, rhs: Self) -> bool {
        (self.regular & rhs.regular) != 0 || (self.generated & rhs.generated) != 0
    }
}
