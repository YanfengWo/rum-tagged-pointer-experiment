//! An external, top-level representation experiment, not a replacement rum kernel.
//! All unsafe operations are private. Existing children retain rum's representation.
//!
//! ```compile_fail
//! use rum_compact_expr::CompactExpr;
//! fn escape() -> &'static rum_core::Expr {
//!     let value = CompactExpr::new(rum_core::Expr::string("owned"));
//!     match value.view() {
//!         rum_compact_expr::ExprView::Heap(expr) => expr,
//!         _ => unreachable!(),
//!     }
//! }
//! ```

#![deny(unsafe_op_in_unsafe_fn)]

#[cfg(not(target_pointer_width = "64"))]
compile_error!("This research prototype currently requires 64-bit pointers");

use rum_core::{Atom, Expr, IntValue};
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use std::sync::Arc;

pub const MIN_INLINE: i64 = -(1_i64 << 60);
pub const MAX_INLINE: i64 = (1_i64 << 60) - 1;
const MASK: usize = 7;
const INTEGER: usize = 1;
const HEAP: usize = 2;

#[derive(Clone, Copy, Debug)]
pub enum ExprView<'a> {
    Integer(i64),
    Heap(&'a Expr),
}

impl ExprView<'_> {
    pub fn to_expr(self) -> Expr {
        match self {
            Self::Integer(value) => Expr::small_integer(value),
            Self::Heap(expr) => expr.clone(),
        }
    }
}

#[repr(align(8))]
#[derive(Clone)]
struct Payload(Expr);

fn inline_value(expr: &Expr) -> Option<i64> {
    match expr {
        Expr::Atom(Atom::Integer(IntValue::Small(value)))
            if (MIN_INLINE..=MAX_INLINE).contains(value) =>
        {
            Some(*value)
        }
        _ => None,
    }
}

/// Owns one Arc strong count or one immediate integer. No raw constructors exist.
/// The pointer retains the allocation's provenance for the heap variant.
#[repr(transparent)]
pub struct CompactExpr {
    tagged: NonNull<Payload>,
    ownership: PhantomData<Arc<Payload>>,
}

// SAFETY: the only shared memory is an immutable Arc<Payload>. Payload contains
// Expr, which is Send + Sync; checked by compile-time assertions in the tests.
// Pointer tagging changes the address only. No mutable references are exposed.
unsafe impl Send for CompactExpr {}
unsafe impl Sync for CompactExpr {}

// Keep the Send/Sync safety premise enforced even in builds without tests.
const _: fn() = || {
    fn check<T: Send + Sync>() {}
    check::<Expr>();
};

impl CompactExpr {
    #[inline]
    pub fn new(expr: Expr) -> Self {
        let tagged = match inline_value(&expr) {
            Some(value) => {
                let bits = ((value as usize) << 3) | INTEGER;
                NonNull::new(std::ptr::without_provenance_mut(bits)).unwrap()
            }
            None => {
                let raw = Arc::into_raw(Arc::new(Payload(expr))).cast_mut();
                // Payload alignment reserves all three low bits. with_addr
                // preserves provenance; this tagged address is never dereferenced.
                debug_assert_eq!(raw.addr() & MASK, 0);
                NonNull::new(raw.with_addr(raw.addr() | HEAP)).unwrap()
            }
        };
        Self {
            tagged,
            ownership: PhantomData,
        }
    }

    fn is_inline(&self) -> bool {
        // The closed representation has only tags 001 and 010. Testing bit zero
        // distinguishes them without masking and comparing all reserved bits.
        self.tagged.addr().get() & INTEGER != 0
    }

    fn integer(&self) -> i64 {
        (self.tagged.addr().get() as i64) >> 3
    }

    // Private: callers establish the heap variant before using this pointer.
    fn heap_ptr(&self) -> *const Payload {
        let raw = self.tagged.as_ptr();
        raw.with_addr(raw.addr() & !MASK)
    }

    #[inline]
    pub fn view(&self) -> ExprView<'_> {
        if self.is_inline() {
            ExprView::Integer(self.integer())
        } else {
            // SAFETY: this owner holds a live strong count. Restoring the address
            // preserves Arc::into_raw provenance and Payload alignment. The
            // returned borrow is bounded by &self, including across clones.
            ExprView::Heap(&unsafe { &*self.heap_ptr() }.0)
        }
    }

    #[inline]
    pub fn to_expr(&self) -> Expr {
        self.view().to_expr()
    }

    #[inline]
    pub fn into_expr(self) -> Expr {
        if self.is_inline() {
            return Expr::small_integer(self.integer());
        }
        let this = ManuallyDrop::new(self);
        // SAFETY: transfer this owner's exactly one strong count to Arc. The
        // ManuallyDrop prevents a second decrement; no user callback precedes it.
        let arc = unsafe { Arc::from_raw(this.heap_ptr()) };
        match Arc::try_unwrap(arc) {
            Ok(payload) => payload.0,
            Err(shared) => shared.0.clone(),
        }
    }
}

impl Clone for CompactExpr {
    #[inline]
    fn clone(&self) -> Self {
        if !self.is_inline() {
            // SAFETY: &self keeps the allocation alive while Arc increments its
            // strong count. The new owner receives that count; overflow follows
            // Arc's abort behavior. This operation cannot unwind after increment.
            unsafe { Arc::increment_strong_count(self.heap_ptr()) };
        }
        Self {
            tagged: self.tagged,
            ownership: PhantomData,
        }
    }
}

impl Drop for CompactExpr {
    #[inline]
    fn drop(&mut self) {
        if !self.is_inline() {
            // SAFETY: exactly one strong count belongs to this owner. Restore
            // the original pointer and let Arc release/deallocate it exactly once.
            drop(unsafe { Arc::from_raw(self.heap_ptr()) });
        }
    }
}

/// Safe control: identical inline range, payload allocation and sharing policy.
#[derive(Clone)]
pub struct SafeExpr(SafeRepr);

#[derive(Clone)]
enum SafeRepr {
    Integer(i64),
    Heap(Arc<Payload>),
}

impl SafeExpr {
    #[inline]
    pub fn new(expr: Expr) -> Self {
        Self(match inline_value(&expr) {
            Some(value) => SafeRepr::Integer(value),
            None => SafeRepr::Heap(Arc::new(Payload(expr))),
        })
    }

    #[inline]
    pub fn view(&self) -> ExprView<'_> {
        match &self.0 {
            SafeRepr::Integer(value) => ExprView::Integer(*value),
            SafeRepr::Heap(payload) => ExprView::Heap(&payload.0),
        }
    }

    #[inline]
    pub fn to_expr(&self) -> Expr {
        self.view().to_expr()
    }

    #[inline]
    pub fn into_expr(self) -> Expr {
        match self.0 {
            SafeRepr::Integer(value) => Expr::small_integer(value),
            SafeRepr::Heap(arc) => match Arc::try_unwrap(arc) {
                Ok(payload) => payload.0,
                Err(shared) => shared.0.clone(),
            },
        }
    }
}

#[cfg(test)]
mod tests;
