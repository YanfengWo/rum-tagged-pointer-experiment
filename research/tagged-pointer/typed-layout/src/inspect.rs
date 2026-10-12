//! Numeric equality/hash observations. Keys never reconstruct a Symbol.
use crate::{ArgsRef, HeadRef};
use rum_core::{BigInt, Rational, Symbol};
use std::num::NonZeroU64;

/// Equality/hash identity, not lexical ordering or an owning text handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SymbolKey {
    id: u32,
    identity: Option<NonZeroU64>,
}
impl SymbolKey {
    #[inline]
    pub fn from_symbol(s: Symbol) -> Self {
        Self::new(s.interned_id(), s.identity().and_then(NonZeroU64::new))
    }
    pub(crate) fn new(id: u32, identity: Option<NonZeroU64>) -> Self {
        Self { id, identity }
    }
    #[inline]
    pub fn interned_id(self) -> u32 {
        self.id
    }
    #[inline]
    pub fn identity(self) -> Option<u64> {
        self.identity.map(NonZeroU64::get)
    }
}
/// A read-only view with a numeric symbol key instead of a rebuilt Symbol.
pub enum Inspect<'a, N> {
    Small(i64),
    Big(&'a BigInt),
    Rational(&'a Rational),
    String(&'a str),
    Symbol(SymbolKey),
    Call {
        head: HeadRef<'a, N>,
        args: ArgsRef<'a, N>,
    },
}
