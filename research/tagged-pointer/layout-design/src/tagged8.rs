//! Eight-byte handle: immediate signed integers, static plain symbols, or Arc.
//! The fallback deliberately keeps the previous common body layout, so this is
//! a conservative eight-byte comparison, not its minimum possible heap cost.
use crate::{Args, AtomRef, Head, IntRef, Storage, Tree, View, atom_ref, sealed};
use rum_core::{Atom, IntValue, Symbol, expr::SymbolText};
use std::{
    marker::PhantomData,
    mem::{ManuallyDrop, align_of, size_of},
    num::NonZeroUsize,
    ptr::NonNull,
    sync::Arc,
};

const MIN: i64 = -(1_i64 << 60);
const MAX: i64 = (1_i64 << 60) - 1;
const MASK: usize = 7;
const SMALL: usize = 1;
const SYMBOL: usize = 2;

#[derive(Clone)]
enum Body {
    Atom(Atom),
    Call {
        head: Head<Tagged8>,
        args: Args<Tagged8>,
    },
}

#[repr(align(8))]
struct Payload(Body);

/// Closed, owning representation. No tagged address or raw constructor escapes.
#[repr(transparent)]
pub struct Tagged8 {
    pointer: NonNull<Payload>,
    ownership: PhantomData<Arc<Payload>>,
}

// SAFETY: heap payloads are immutable under Arc sharing; Head/Args retain Arc's
// COW policy. Static SymbolText is Sync; immediate integers share no allocation.
// The nonrecursive Atom and SymbolText bounds are checked below. Recursive
// children follow these same closed rules, with no mutable reference escape.
unsafe impl Send for Tagged8 {}
unsafe impl Sync for Tagged8 {}

impl sealed::Sealed for Tagged8 {}

impl Tagged8 {
    #[inline]
    pub fn atom(value: Atom) -> Self {
        match &value {
            Atom::Integer(IntValue::Small(number)) if (MIN..=MAX).contains(number) => {
                // No provenance is fabricated: this pointer is an integer carrier
                // and the SMALL branch never dereferences it or gives it to Arc.
                let bits = ((*number as usize) << 3) | SMALL;
                return Self {
                    pointer: NonNull::new(std::ptr::without_provenance_mut(bits)).unwrap(),
                    ownership: PhantomData,
                };
            }
            Atom::Symbol(symbol) if symbol.identity().is_none() => {
                let text = NonNull::from(symbol.interned_text()).cast::<Payload>();
                debug_assert_eq!(text.addr().get() & MASK, 0);
                // map_addr preserves the provenance of the permanent allocation.
                return Self {
                    pointer: text
                        .map_addr(|address| NonZeroUsize::new(address.get() | SYMBOL).unwrap()),
                    ownership: PhantomData,
                };
            }
            _ => {}
        }
        Self::heap(Body::Atom(value))
    }

    #[inline]
    pub fn small(value: i64) -> Self {
        Self::atom(Atom::Integer(IntValue::Small(value)))
    }

    #[inline]
    pub fn call(head: Head<Self>, args: Args<Self>) -> Self {
        Self::heap(Body::Call { head, args })
    }

    #[inline]
    fn heap(body: Body) -> Self {
        let pointer = NonNull::new(Arc::into_raw(Arc::new(Payload(body))).cast_mut()).unwrap();
        debug_assert_eq!(pointer.addr().get() & MASK, 0);
        Self {
            pointer,
            ownership: PhantomData,
        }
    }

    #[inline]
    fn tag(&self) -> usize {
        self.pointer.addr().get() & MASK
    }

    #[inline]
    fn integer(&self) -> i64 {
        (self.pointer.addr().get() as i64) >> 3
    }

    #[inline]
    fn symbol(&self) -> Symbol {
        let text = self
            .pointer
            .map_addr(|address| NonZeroUsize::new(address.get() & !MASK).unwrap())
            .cast::<SymbolText>();
        // SAFETY: SYMBOL is constructed only from an interned &'static text.
        // Clearing only the alignment tag restores its original address and
        // provenance. The leaked text remains live independently of this owner.
        let text: &'static SymbolText = unsafe { &*text.as_ptr() };
        Symbol::from_interned_text(text, None)
    }

    #[inline]
    pub fn view(&self) -> View<'_, Self> {
        match self.tag() {
            SMALL => View::Atom(AtomRef::Integer(IntRef::Small(self.integer()))),
            SYMBOL => View::Atom(AtomRef::Symbol(self.symbol())),
            0 => {
                // SAFETY: tag zero holds one live Arc count at the unchanged
                // Arc::into_raw data pointer; the borrow is bounded by &self.
                let body = &unsafe { &*self.pointer.as_ptr() }.0;
                match body {
                    Body::Atom(atom) => View::Atom(atom_ref(atom)),
                    Body::Call { head, args } => View::Call {
                        head: head.borrow(),
                        args: args.borrow(),
                    },
                }
            }
            _ => unreachable!("private constructors create only tags 0, 1 and 2"),
        }
    }

    /// Consume a unique payload or clone its contents when another owner exists.
    pub fn into_parts(self) -> Result<(Head<Self>, Args<Self>), Atom> {
        match self.tag() {
            SMALL => Err(Atom::Integer(IntValue::Small(self.integer()))),
            SYMBOL => Err(Atom::Symbol(self.symbol())),
            0 => {
                let this = ManuallyDrop::new(self);
                // SAFETY: transfer this owner's sole strong count into Arc;
                // ManuallyDrop prevents releasing that same count twice.
                let arc = unsafe { Arc::from_raw(this.pointer.as_ptr()) };
                let body = match Arc::try_unwrap(arc) {
                    Ok(payload) => payload.0,
                    Err(shared) => shared.0.clone(),
                };
                match body {
                    Body::Atom(atom) => Err(atom),
                    Body::Call { head, args } => Ok((head, args)),
                }
            }
            _ => unreachable!("invalid private tag"),
        }
    }

    pub fn release_iterative(self) {
        crate::release(self)
    }
}

impl Storage for Tagged8 {
    fn take_call(self) -> Option<(Head<Self>, Args<Self>)> {
        if self.tag() != 0 {
            return None;
        }
        let this = ManuallyDrop::new(self);
        // SAFETY: one count is transferred exactly as in into_parts. into_inner
        // releases it on every path, moving the body only for the last owner.
        let arc = unsafe { Arc::from_raw(this.pointer.as_ptr()) };
        match Arc::into_inner(arc).map(|payload| payload.0) {
            Some(Body::Call { head, args }) => Some((head, args)),
            _ => None,
        }
    }
}

impl Tree for Tagged8 {
    #[inline]
    fn atom(value: Atom) -> Self {
        Self::atom(value)
    }
    #[inline]
    fn call(head: Head<Self>, args: Args<Self>) -> Self {
        Self::call(head, args)
    }
    #[inline]
    fn view(&self) -> View<'_, Self> {
        self.view()
    }
    fn into_parts(self) -> Result<(Head<Self>, Args<Self>), Atom> {
        self.into_parts()
    }
    fn release_iterative(self) {
        self.release_iterative()
    }
}

impl Clone for Tagged8 {
    #[inline]
    fn clone(&self) -> Self {
        if self.tag() == 0 {
            // SAFETY: &self keeps the payload live while acquiring one count.
            // No fallible or user operation follows before forming the new owner.
            unsafe { Arc::increment_strong_count(self.pointer.as_ptr()) };
        }
        Self {
            pointer: self.pointer,
            ownership: PhantomData,
        }
    }
}

impl Drop for Tagged8 {
    #[inline]
    fn drop(&mut self) {
        if self.tag() == 0 {
            // SAFETY: release exactly this owner's count using the unchanged
            // Arc data pointer. Integer and permanent Symbol tags never use Arc.
            drop(unsafe { Arc::from_raw(self.pointer.as_ptr()) });
        }
    }
}

const _: () = {
    assert!(size_of::<Tagged8>() == 8);
    assert!(align_of::<Payload>() >= 8);
    assert!(align_of::<SymbolText>() >= 8);
};
const _: fn() = || {
    fn send_sync<T: Send + Sync>() {}
    send_sync::<Atom>();
    send_sync::<SymbolText>();
};
