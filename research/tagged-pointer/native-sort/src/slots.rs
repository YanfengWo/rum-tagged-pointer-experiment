//! Full identities are retained in payloads, ignored by qualified-name sort.
use rum_core::{Symbol, expr::SymbolText};
use std::{cmp::Ordering, num::NonZeroU64};
pub trait Slot: Copy {
    fn new(s: Symbol) -> Self;
    fn symbol(self) -> Symbol;
    fn compare(&self, other: &Self) -> Ordering;
}
impl Slot for Symbol {
    fn new(s: Symbol) -> Self {
        s
    }
    fn symbol(self) -> Symbol {
        self
    }
    fn compare(&self, b: &Self) -> Ordering {
        self.cmp_qualified_name(b)
    }
}
#[derive(Clone, Copy)]
pub struct Safe16 {
    text: &'static SymbolText,
    identity: Option<NonZeroU64>,
}
impl Slot for Safe16 {
    fn new(s: Symbol) -> Self {
        Self {
            text: s.interned_text(),
            identity: s.identity().and_then(NonZeroU64::new),
        }
    }
    fn symbol(self) -> Symbol {
        Symbol::from_interned_text(self.text, self.identity)
    }
    fn compare(&self, b: &Self) -> Ordering {
        self.text.cmp_qualified_text(b.text)
    }
}
#[derive(Clone, Copy)]
pub struct Tagged16 {
    text: *const SymbolText,
    bits: u64,
}
const _: () = {
    assert!(std::mem::size_of::<usize>() == 8);
    assert!(std::mem::align_of::<SymbolText>() >= 8);
    assert!(std::mem::size_of::<Tagged16>() == 16);
    assert!(std::mem::size_of::<Safe16>() == 16);
};
// SAFETY: private constructors encode only authentic, permanent immutable text;
// numeric bits are never dereferenced. Both components have no mutable access.
unsafe impl Send for Tagged16 {}
unsafe impl Sync for Tagged16 {}
const _: fn() = || {
    fn check<T: Send + Sync>() {}
    check::<SymbolText>();
};
impl Tagged16 {
    fn text(&self) -> &'static SymbolText {
        // SAFETY: clearing the tag preserves the exact authentic text pointer's
        // provenance. The permanent interner contract keeps it alive forever.
        unsafe { &*self.text.map_addr(|a| a & !7) }
    }
}
impl Slot for Tagged16 {
    fn new(s: Symbol) -> Self {
        let identity = s.identity();
        let tag = if identity.is_some() { 2 } else { 1 };
        Self {
            text: (s.interned_text() as *const SymbolText).map_addr(|a| a | tag),
            bits: identity.unwrap_or(s.interned_id() as u64),
        }
    }
    fn symbol(self) -> Symbol {
        let identity = if self.text.addr() & 7 == 2 {
            NonZeroU64::new(self.bits)
        } else {
            None
        };
        Symbol::from_interned_text(self.text(), identity)
    }
    fn compare(&self, b: &Self) -> Ordering {
        if self.text.addr() & 7 == 1 && b.text.addr() & 7 == 1 && self.bits == b.bits {
            return Ordering::Equal;
        }
        self.text().cmp_qualified_text(b.text())
    }
}
