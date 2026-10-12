//! Two pointer words, cached plain ids, typed Arc payloads. All fields private.
use crate::{
    Args, ArgsRef, AtomRef, Head, HeadRef, Inspect, IntRef, Storage, SymbolKey, Tree, View,
    atom_ref, sealed,
};
use rum_core::{Atom, BigInt, IntValue, Rational, Symbol, expr::SymbolText};
use std::{
    marker::PhantomData,
    mem::{align_of, size_of},
    num::NonZeroU64,
    ptr,
    sync::Arc,
};
mod lifecycle;
const MASK: usize = 7;
const OWNED: usize = 0;
const INTEGER: usize = 1;
const GENERATED: usize = 2;
const CALL: usize = 3;
const STRING: usize = 4;
const BIG: usize = 5;
const RATIONAL: usize = 6;
const PLAIN: usize = 7;
#[repr(C)]
pub struct Typed16 {
    first: *const (),
    second: *const (),
    ownership: PhantomData<Arc<Atom>>,
}
const _: () = {
    assert!(size_of::<usize>() == 8 && size_of::<Typed16>() == 16);
    assert!(align_of::<Typed16>() >= 8 && align_of::<Atom>() >= 8);
    assert!(
        align_of::<BigInt>() >= 8 && align_of::<Rational>() >= 8 && align_of::<SymbolText>() >= 8
    );
};
const _: fn() = || {
    fn check<T: Send + Sync>() {}
    check::<Atom>();
    check::<SymbolText>();
    check::<BigInt>();
    check::<Rational>();
};
// SAFETY: closed states own immutable, Send+Sync Arc data or permanent text.
// Numeric words are never dereferenced. Every tag has its exact count protocol.
unsafe impl Send for Typed16 {}
unsafe impl Sync for Typed16 {}
impl Typed16 {
    #[inline]
    fn words(first: *const (), second: *const ()) -> Self {
        Self {
            first,
            second,
            ownership: PhantomData,
        }
    }
    #[inline]
    fn heap<T>(value: Arc<T>, tag: usize) -> Self {
        let raw = Arc::into_raw(value).cast::<()>();
        debug_assert_eq!(raw.addr() & MASK, 0);
        Self::words(raw.map_addr(|a| a | tag), ptr::null())
    }
    #[inline]
    fn string(value: Arc<str>) -> Self {
        let len = value.len();
        let raw = Arc::into_raw(value);
        let data = raw.cast::<u8>();
        // No assumption about Arc<str>'s data alignment. Never tag an unaligned
        // address. Recover the count and fall back to the checked Atom layout.
        if data.addr() & MASK != 0 {
            // SAFETY: exact pointer and metadata returned by into_raw above.
            let value = unsafe { Arc::from_raw(raw) };
            return Self::heap(Arc::new(Atom::String(value.to_string())), OWNED);
        }
        Self::words(
            data.cast::<()>().map_addr(|a| a | STRING),
            ptr::without_provenance(len),
        )
    }
    #[inline]
    pub fn atom(atom: Atom) -> Self {
        match atom {
            Atom::Integer(IntValue::Small(n)) => Self::small(n),
            Atom::Integer(IntValue::Big(n)) => Self::heap(Arc::new(*n), BIG),
            Atom::Rational(n) => Self::heap(Arc::new(*n), RATIONAL),
            Atom::String(s) => Self::string(Arc::from(s)),
            Atom::Symbol(s) => Self::symbol_owner(s),
        }
    }
    #[inline]
    pub fn borrowed_atom(atom: &Atom) -> Self {
        match atom {
            Atom::String(s) => Self::string(Arc::from(s.as_str())),
            Atom::Integer(IntValue::Big(n)) => Self::heap(Arc::new((**n).clone()), BIG),
            Atom::Rational(n) => Self::heap(Arc::new((**n).clone()), RATIONAL),
            other => Self::atom(other.clone()),
        }
    }
    #[inline]
    fn symbol_owner(s: Symbol) -> Self {
        let (tag, bits) = if let Some(identity) = s.identity() {
            (GENERATED, identity as usize)
        } else {
            (PLAIN, s.interned_id() as usize)
        };
        Self::words(
            (s.interned_text() as *const SymbolText)
                .cast::<()>()
                .map_addr(|a| a | tag),
            ptr::without_provenance(bits),
        )
    }
    #[inline]
    pub fn small(n: i64) -> Self {
        Self::words(
            ptr::without_provenance(INTEGER),
            ptr::without_provenance(n as usize),
        )
    }
    #[inline]
    pub fn call(head: Head<Self>, args: Args<Self>) -> Self {
        Self::words(
            Arc::into_raw(head.0).cast::<()>().map_addr(|a| a | CALL),
            Arc::into_raw(args.0).cast(),
        )
    }
    #[inline]
    fn tag(&self) -> usize {
        self.first.addr() & MASK
    }
    #[inline]
    fn data<T>(&self) -> *const T {
        self.first.map_addr(|a| a & !MASK).cast()
    }
    #[inline]
    fn args_ptr(&self) -> *const Vec<Self> {
        self.second.cast()
    }
    #[inline]
    fn str_ptr(&self) -> *const str {
        ptr::slice_from_raw_parts(self.data::<u8>(), self.second.addr()) as *const str
    }
    #[inline]
    fn text(&self) -> &'static SymbolText {
        // SAFETY: plain/generated constructors receive authentic permanent text.
        unsafe { &*self.data::<SymbolText>() }
    }
    #[inline]
    fn identity(&self) -> Option<NonZeroU64> {
        if self.tag() == GENERATED {
            NonZeroU64::new(self.second.addr() as u64)
        } else {
            None
        }
    }
    #[inline]
    fn symbol(&self) -> Symbol {
        Symbol::from_interned_text(self.text(), self.identity())
    }
    #[inline]
    fn key(&self) -> SymbolKey {
        let id = if self.tag() == PLAIN {
            self.second.addr() as u32
        } else {
            self.text().interned_id()
        };
        SymbolKey::new(id, self.identity())
    }
    #[inline]
    pub fn view(&self) -> View<'_, Self> {
        // SAFETY for leaf arms: their exact original typed Arc pointer is restored
        // and this owner holds a strong count for the returned borrow. For str,
        // metadata is its saved byte length; bytes remain valid immutable UTF-8.
        let atom = match self.tag() {
            INTEGER => AtomRef::Integer(IntRef::Small(self.second.addr() as i64)),
            PLAIN | GENERATED => AtomRef::Symbol(self.symbol()),
            STRING => AtomRef::String(unsafe { &*self.str_ptr() }),
            BIG => AtomRef::Integer(IntRef::Big(unsafe { &*self.data::<BigInt>() })),
            RATIONAL => AtomRef::Rational(unsafe { &*self.data::<Rational>() }),
            OWNED => atom_ref(unsafe { &*self.data::<Atom>() }),
            CALL => return self.call_view(),
            _ => unreachable!(),
        };
        View::Atom(atom)
    }
    #[inline]
    fn call_view(&self) -> View<'_, Self> {
        // SAFETY: CALL owns both exact Arc payloads for this source borrow.
        View::Call {
            head: unsafe { HeadRef::from_raw(self.data::<Self>()) },
            args: unsafe { ArgsRef::from_raw(self.args_ptr()) },
        }
    }
    #[inline]
    pub fn inspect(&self) -> Inspect<'_, Self> {
        if matches!(self.tag(), PLAIN | GENERATED) {
            return Inspect::Symbol(self.key());
        }
        match self.view() {
            View::Atom(AtomRef::Integer(IntRef::Small(n))) => Inspect::Small(n),
            View::Atom(AtomRef::Integer(IntRef::Big(n))) => Inspect::Big(n),
            View::Atom(AtomRef::Rational(n)) => Inspect::Rational(n),
            View::Atom(AtomRef::String(s)) => Inspect::String(s),
            View::Call { head, args } => Inspect::Call { head, args },
            View::Atom(AtomRef::Symbol(_)) => unreachable!(),
        }
    }
    pub fn release_iterative(self) {
        crate::release(self)
    }
}
impl sealed::Sealed for Typed16 {}
impl Tree for Typed16 {
    #[inline]
    fn atom(atom: Atom) -> Self {
        Self::atom(atom)
    }
    #[inline]
    fn borrowed_atom(atom: &Atom) -> Self {
        Self::borrowed_atom(atom)
    }
    #[inline]
    fn call(head: Head<Self>, args: Args<Self>) -> Self {
        Self::call(head, args)
    }
    #[inline]
    fn view(&self) -> View<'_, Self> {
        self.view()
    }
    #[inline]
    fn inspect(&self) -> Inspect<'_, Self> {
        self.inspect()
    }
    fn into_parts(self) -> Result<(Head<Self>, Args<Self>), Atom> {
        self.into_parts()
    }
    fn release_iterative(self) {
        self.release_iterative()
    }
}
