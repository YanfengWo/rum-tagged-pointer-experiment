use super::*;

use std::hash::{BuildHasherDefault, Hash, Hasher};

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
