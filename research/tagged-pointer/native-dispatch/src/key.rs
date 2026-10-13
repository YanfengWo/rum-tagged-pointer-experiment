//! Cache-key equality/hash identity, never lexical ordering or serialization.
#[cfg(any(feature = "tagged16", feature = "safe24"))]
use rum_core::Atom;
use rum_core::Symbol;
#[cfg(feature = "safe24")]
use rum_typed_layout::Tree;
#[cfg(any(feature = "tagged16", feature = "safe24"))]
use rum_typed_layout::{Inspect, SymbolKey};
use std::hash::{Hash, Hasher};
#[cfg(any(test, feature = "tagged16", feature = "safe24", feature = "packed16"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Probe {
    pub id: u64,
    pub identity: u64,
}
#[cfg(any(test, feature = "tagged16", feature = "safe24", feature = "packed16"))]
impl Probe {
    #[inline]
    pub fn from_symbol(s: Symbol) -> Self {
        Self {
            id: s.interned_id() as u64,
            identity: s.identity().unwrap_or(0),
        }
    }
    #[cfg(any(feature = "tagged16", feature = "safe24"))]
    #[inline]
    fn from_key(k: SymbolKey) -> Self {
        Self {
            id: k.interned_id() as u64,
            identity: k.identity().unwrap_or(0),
        }
    }
}
#[cfg(any(test, feature = "tagged16", feature = "safe24", feature = "packed16"))]
impl Hash for Probe {
    #[inline]
    fn hash<H: Hasher>(&self, h: &mut H) {
        h.write_u64(self.id.wrapping_mul(0x9e3779b97f4a7c15) ^ self.identity);
    }
}
#[cfg(feature = "tagged16")]
#[derive(Clone)]
pub(crate) struct Key(rum_typed_layout::Typed16);
#[cfg(all(not(feature = "tagged16"), feature = "safe24"))]
#[derive(Clone)]
pub(crate) struct Key(rum_typed_layout::Safe24);
#[cfg(all(
    not(feature = "tagged16"),
    not(feature = "safe24"),
    feature = "packed16"
))]
#[derive(Clone)]
pub(crate) struct Key(Probe);
#[cfg(not(any(feature = "tagged16", feature = "safe24", feature = "packed16")))]
#[derive(Clone)]
pub(crate) struct Key(Symbol);
impl Key {
    #[inline]
    pub fn new(s: Symbol) -> Self {
        #[cfg(feature = "tagged16")]
        {
            Self(rum_typed_layout::Typed16::atom(Atom::Symbol(s)))
        }
        #[cfg(all(not(feature = "tagged16"), feature = "safe24"))]
        {
            Self(rum_typed_layout::Safe24::atom(Atom::Symbol(s)))
        }
        #[cfg(all(
            not(feature = "tagged16"),
            not(feature = "safe24"),
            feature = "packed16"
        ))]
        {
            Self(Probe::from_symbol(s))
        }
        #[cfg(not(any(feature = "tagged16", feature = "safe24", feature = "packed16")))]
        {
            Self(s)
        }
    }
    #[cfg(any(test, feature = "tagged16", feature = "safe24", feature = "packed16"))]
    #[inline]
    pub fn probe(&self) -> Probe {
        #[cfg(any(feature = "tagged16", feature = "safe24"))]
        {
            let Inspect::Symbol(k) = self.0.inspect() else {
                unreachable!()
            };
            Probe::from_key(k)
        }
        #[cfg(all(
            not(feature = "tagged16"),
            not(feature = "safe24"),
            feature = "packed16"
        ))]
        {
            self.0
        }
        #[cfg(not(any(feature = "tagged16", feature = "safe24", feature = "packed16")))]
        {
            Probe::from_symbol(self.0)
        }
    }
}
impl PartialEq for Key {
    #[inline]
    fn eq(&self, b: &Self) -> bool {
        #[cfg(not(any(feature = "tagged16", feature = "safe24", feature = "packed16")))]
        {
            self.0 == b.0
        }
        #[cfg(any(feature = "tagged16", feature = "safe24", feature = "packed16"))]
        {
            self.probe() == b.probe()
        }
    }
}
impl Eq for Key {}
impl Hash for Key {
    #[inline]
    fn hash<H: Hasher>(&self, h: &mut H) {
        #[cfg(not(any(feature = "tagged16", feature = "safe24", feature = "packed16")))]
        {
            self.0.hash(h);
        }
        #[cfg(any(feature = "tagged16", feature = "safe24", feature = "packed16"))]
        {
            self.probe().hash(h);
        }
    }
}
#[cfg(not(any(feature = "tagged16", feature = "safe24", feature = "packed16")))]
impl std::borrow::Borrow<Symbol> for Key {
    fn borrow(&self) -> &Symbol {
        &self.0
    }
}

#[cfg(any(feature = "tagged16", feature = "safe24", feature = "packed16"))]
impl hashbrown::Equivalent<Key> for Probe {
    #[inline]
    fn equivalent(&self, key: &Key) -> bool {
        *self == key.probe()
    }
}
