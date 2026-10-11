//! Two-word representation: full integers and symbols are immediate; calls own
//! their existing head/argument Arc counts directly, with no outer allocation.
use crate::{Args, ArgsRef, Head, HeadRef, Storage, Tree, View, atom_ref, sealed};
use rum_core::{Atom, Expr, IntValue, Symbol, expr::SymbolText};
use std::{
    marker::PhantomData,
    mem::{ManuallyDrop, align_of, size_of},
    num::NonZeroU64,
    ptr,
    sync::Arc,
};

const MASK: usize = 7;
const OWNED: usize = 0;
const INTEGER: usize = 1;
const SYMBOL: usize = 2;
const CALL: usize = 3;

/// Closed representation whose owned pointers retain their original provenance.
/// No raw constructor or mutable access to these words is exposed.
#[repr(C)]
pub struct Tagged16 {
    first: *const (),
    second: *const (),
    ownership: PhantomData<Arc<Atom>>,
}

// The three pointer-bearing variants can reserve their low three address bits.
// These checks fail at build time instead of silently accepting another layout.
const _: () = {
    assert!(size_of::<usize>() == 8);
    assert!(size_of::<Tagged16>() == 16);
    assert!(align_of::<Tagged16>() >= 8);
    assert!(align_of::<Atom>() >= 8);
    assert!(align_of::<SymbolText>() >= 8);
};
const _: fn() = || {
    fn check<T: Send + Sync>() {}
    check::<Atom>();
    check::<SymbolText>();
};

// SAFETY: integers/identities have no shared memory; SymbolText references are
// immutable and static; owned atoms, heads and arguments are immutable Arc data.
// Their types are Send + Sync. Mutation of Args uses Arc's safe COW interface.
// Raw fields are private and every clone/drop transfers the appropriate counts.
unsafe impl Send for Tagged16 {}
unsafe impl Sync for Tagged16 {}

impl Tagged16 {
    #[inline]
    pub fn atom(value: Atom) -> Self {
        let (first, second) = match value {
            Atom::Integer(IntValue::Small(number)) => (
                ptr::without_provenance(INTEGER),
                ptr::without_provenance(number as usize),
            ),
            Atom::Symbol(symbol) => (
                (symbol.interned_text() as *const SymbolText)
                    .cast::<()>()
                    .map_addr(|address| address | SYMBOL),
                ptr::without_provenance(symbol.identity().unwrap_or(0) as usize),
            ),
            value => (Arc::into_raw(Arc::new(value)).cast(), ptr::null()),
        };
        Self {
            first,
            second,
            ownership: PhantomData,
        }
    }

    pub fn small(value: i64) -> Self {
        Self::atom(Atom::Integer(IntValue::Small(value)))
    }

    #[inline]
    pub fn call(head: Head<Self>, args: Args<Self>) -> Self {
        Self {
            first: Arc::into_raw(head.0)
                .cast::<()>()
                .map_addr(|address| address | CALL),
            second: Arc::into_raw(args.0).cast(),
            ownership: PhantomData,
        }
    }

    #[inline]
    fn tag(&self) -> usize {
        self.first.addr() & MASK
    }

    #[inline]
    fn atom_ptr(&self) -> *const Atom {
        self.first.cast()
    }

    #[inline]
    fn head_ptr(&self) -> *const Self {
        self.first.map_addr(|address| address & !MASK).cast()
    }

    #[inline]
    fn args_ptr(&self) -> *const Vec<Self> {
        self.second.cast()
    }

    #[inline]
    fn symbol(&self) -> Symbol {
        let raw = self
            .first
            .map_addr(|address| address & !MASK)
            .cast::<SymbolText>();
        // SAFETY: the SYMBOL constructor obtains this pointer solely from a
        // safe &'static SymbolText. map_addr restores the exact address while
        // retaining provenance. Clone/Drop never release this static storage.
        let text: &'static SymbolText = unsafe { &*raw };
        Symbol::from_interned_text(text, NonZeroU64::new(self.second.addr() as u64))
    }

    #[inline]
    pub fn view(&self) -> View<'_, Self> {
        match self.tag() {
            INTEGER => View::Atom(crate::AtomRef::Integer(crate::IntRef::Small(
                self.second.addr() as i64,
            ))),
            SYMBOL => View::Atom(crate::AtomRef::Symbol(self.symbol())),
            OWNED => {
                // SAFETY: this owner holds one live Arc<Atom> count and its
                // exact data pointer. The returned borrow is bounded by &self.
                View::Atom(atom_ref(unsafe { &*self.atom_ptr() }))
            }
            CALL => {
                // SAFETY: this call owns one live count for each exact pointer.
                // Both borrowed handles are bounded by view's &self lifetime;
                // creating them transfers no counts and exposes no mutation.
                let head = unsafe { HeadRef::from_raw(self.head_ptr()) };
                let args = unsafe { ArgsRef::from_raw(self.args_ptr()) };
                View::Call { head, args }
            }
            _ => unreachable!("private tagged representation invariant"),
        }
    }

    pub fn from_expr(expr: &Expr) -> Self {
        crate::from_expr(expr)
    }

    pub fn to_expr(&self) -> Expr {
        crate::to_expr(self)
    }

    fn into_call(self) -> (Head<Self>, Args<Self>) {
        debug_assert_eq!(self.tag(), CALL);
        let this = ManuallyDrop::new(self);
        // SAFETY: only CALL branches use this helper. Its two owned counts are
        // moved back into Arc, with exact restored pointers. ManuallyDrop
        // prevents the old representation from decrementing either count.
        let head = unsafe { Arc::from_raw(this.head_ptr()) };
        let args = unsafe { Arc::from_raw(this.args_ptr()) };
        (Head(head), Args(args))
    }

    pub fn into_parts(self) -> Result<(Head<Self>, Args<Self>), Atom> {
        if self.tag() == CALL {
            return Ok(self.into_call());
        }
        let this = ManuallyDrop::new(self);
        Err(match this.tag() {
            INTEGER => Atom::Integer(IntValue::Small(this.second.addr() as i64)),
            SYMBOL => Atom::Symbol(this.symbol()),
            OWNED => {
                // SAFETY: transfer the owner's one Arc<Atom> count. The old
                // owner cannot drop it again. Shared fallback clones Atom,
                // preserving its original variant, and releases this count.
                let atom = unsafe { Arc::from_raw(this.atom_ptr()) };
                Arc::try_unwrap(atom).unwrap_or_else(|shared| shared.as_ref().clone())
            }
            _ => unreachable!("private tagged representation invariant"),
        })
    }

    pub fn release_iterative(self) {
        crate::release(self)
    }
}

impl Clone for Tagged16 {
    #[inline]
    fn clone(&self) -> Self {
        match self.tag() {
            OWNED => {
                // SAFETY: &self retains the live allocation and exact pointer.
                // Arc gives the clone its own count; overflow aborts, not unwinds.
                unsafe { Arc::increment_strong_count(self.atom_ptr()) };
            }
            CALL => {
                // SAFETY: &self keeps both exact Arc data pointers alive. The
                // clone receives one count for each. Neither increment unwinds;
                // overflow follows Arc's abort behavior, so no partial clone
                // can escape or cause mismatched ownership.
                unsafe {
                    Arc::increment_strong_count(self.head_ptr());
                    Arc::increment_strong_count(self.args_ptr());
                }
            }
            INTEGER | SYMBOL => {}
            _ => unreachable!("private tagged representation invariant"),
        }
        Self {
            first: self.first,
            second: self.second,
            ownership: PhantomData,
        }
    }
}

impl Drop for Tagged16 {
    #[inline]
    fn drop(&mut self) {
        match self.tag() {
            OWNED => {
                // SAFETY: this owner transfers its one count to Arc for exactly
                // one release, using Arc::into_raw's original data pointer.
                drop(unsafe { Arc::from_raw(self.atom_ptr()) });
            }
            CALL => {
                // SAFETY: this owner transfers its two counts to ordinary Arc
                // locals. Restored pointers retain their original provenance.
                // Both locals exist before either is dropped, so unwinding
                // while releasing one also releases the other.
                let head = unsafe { Arc::from_raw(self.head_ptr()) };
                let args = unsafe { Arc::from_raw(self.args_ptr()) };
                drop(head);
                drop(args);
            }
            INTEGER | SYMBOL => {}
            _ => unreachable!("private tagged representation invariant"),
        }
    }
}

impl sealed::Sealed for Tagged16 {}
impl Storage for Tagged16 {
    fn take_call(self) -> Option<(Head<Self>, Args<Self>)> {
        // Dropping a non-call releases its owned atom without cloning it.
        (self.tag() == CALL).then(|| self.into_call())
    }
}
impl Tree for Tagged16 {
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
