//! Recursive external representation; the rum kernel is unchanged.
//! Heads and argument vectors retain separate Arc identities, including integer
//! heads. Conversions preserve aliases within each conversion, not old addresses.
//!
//! ```compile_fail
//! use rum_recursive_expr::{Compact, View, AtomRef};
//! fn escape() -> &'static str {
//!     let value = Compact::from_expr(&rum_core::Expr::string("owned"));
//!     match value.view() {
//!         View::Atom(AtomRef::String(text)) => text,
//!         _ => unreachable!(),
//!     }
//! }
//! ```
//!
//! ```compile_fail
//! use rum_recursive_expr::{Args, Compact};
//! fn mutate_borrowed() {
//!     let mut args = Args::new(vec![Compact::small(1)]);
//!     let child = &args[0];
//!     args.make_mut().clear();
//!     let _ = child.view();
//! }
//! ```
#![deny(unsafe_op_in_unsafe_fn)]
#[cfg(not(target_pointer_width = "64"))]
compile_error!("This research prototype requires 64-bit pointers");

pub use rum_compact_expr::{AtomRef, IntRef};
use rum_core::{Atom, Expr, IntValue};
use std::{
    ops::Deref,
    sync::{Arc, Weak},
};
mod conversion;
mod owner;

#[derive(Clone)]
pub struct Head<N>(Arc<N>);
impl<N> Head<N> {
    pub fn new(value: N) -> Self {
        Self(Arc::new(value))
    }
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    pub(crate) fn key(&self) -> usize {
        Arc::as_ptr(&self.0).addr()
    }
}
impl<N> Deref for Head<N> {
    type Target = N;
    fn deref(&self) -> &N {
        &self.0
    }
}

#[derive(Clone)]
pub struct Args<N>(Arc<Vec<N>>);
impl<N> Args<N> {
    pub fn new(values: Vec<N>) -> Self {
        Self(Arc::new(values))
    }
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    pub fn get_mut(&mut self) -> Option<&mut Vec<N>> {
        Arc::get_mut(&mut self.0)
    }
    pub fn downgrade(&self) -> WeakArgs<N> {
        WeakArgs(Arc::downgrade(&self.0))
    }
    pub(crate) fn key(&self) -> usize {
        Arc::as_ptr(&self.0).addr()
    }
}
impl<N: Clone> Args<N> {
    pub fn make_mut(&mut self) -> &mut Vec<N> {
        Arc::make_mut(&mut self.0)
    }
    pub fn into_vec(self) -> Vec<N> {
        Arc::try_unwrap(self.0).unwrap_or_else(|shared| shared.as_ref().clone())
    }
}
impl<N> Deref for Args<N> {
    type Target = [N];
    fn deref(&self) -> &[N] {
        &self.0
    }
}

#[derive(Clone)]
pub struct WeakArgs<N>(Weak<Vec<N>>);
impl<N> WeakArgs<N> {
    pub fn is_alive(&self) -> bool {
        self.0.strong_count() != 0
    }
    pub fn ptr_eq(&self, args: &Args<N>) -> bool {
        std::ptr::eq(self.0.as_ptr(), Arc::as_ptr(&args.0))
    }
}

pub enum View<'a, N> {
    Atom(AtomRef<'a>),
    Call {
        head: &'a Head<N>,
        args: &'a Args<N>,
    },
}

#[derive(Clone)]
enum Body<N> {
    Atom(Atom),
    Call { head: Head<N>, args: Args<N> },
}

fn atom_ref(atom: &Atom) -> AtomRef<'_> {
    match atom {
        Atom::Integer(IntValue::Small(value)) => AtomRef::Integer(IntRef::Small(*value)),
        Atom::Integer(IntValue::Big(value)) => AtomRef::Integer(IntRef::Big(value)),
        Atom::Rational(value) => AtomRef::Rational(value),
        Atom::String(value) => AtomRef::String(value),
        Atom::Symbol(value) => AtomRef::Symbol(*value),
    }
}

mod sealed {
    pub trait Sealed {}
}
/// Closed set of representations, with identical conversion and sharing policy.
pub trait Tree: sealed::Sealed + Clone + Sized {
    fn atom(value: Atom) -> Self;
    fn call(head: Head<Self>, args: Args<Self>) -> Self;
    fn view(&self) -> View<'_, Self>;
    /// Explicit stack-based release for very deep trees. Ordinary Drop remains
    /// recursive, like original rum; this method is not automatically invoked.
    fn release_iterative(self);
}

macro_rules! representation {
    ($name:ident, $storage:ident) => {
        #[derive(Clone)]
        pub struct $name(owner::$storage<Body<$name>>);
        impl sealed::Sealed for $name {}
        impl $name {
            #[inline]
            pub fn atom(value: Atom) -> Self {
                if let Atom::Integer(IntValue::Small(number)) = &value {
                    if (owner::MIN..=owner::MAX).contains(number) {
                        return Self(owner::$storage::small(*number));
                    }
                }
                Self(owner::$storage::heap(Body::Atom(value)))
            }
            pub fn small(value: i64) -> Self {
                Self::atom(Atom::Integer(IntValue::Small(value)))
            }
            #[inline]
            pub fn call(head: Head<Self>, args: Args<Self>) -> Self {
                Self(owner::$storage::heap(Body::Call { head, args }))
            }
            #[inline]
            pub fn view(&self) -> View<'_, Self> {
                match self.0.view() {
                    owner::View::Small(value) => View::Atom(AtomRef::Integer(IntRef::Small(value))),
                    owner::View::Heap(Body::Atom(atom)) => View::Atom(atom_ref(atom)),
                    owner::View::Heap(Body::Call { head, args }) => View::Call { head, args },
                }
            }
            pub fn from_expr(expr: &Expr) -> Self {
                conversion::from_expr(expr)
            }
            pub fn to_expr(&self) -> Expr {
                conversion::to_expr(self)
            }
            /// Move a unique body's children, or clone their Arc counts when shared.
            pub fn into_parts(self) -> Result<(Head<Self>, Args<Self>), Atom> {
                match self.0.into_owned() {
                    owner::Owned::Small(value) => Err(Atom::Integer(IntValue::Small(value))),
                    owner::Owned::Heap(Body::Atom(atom)) => Err(atom),
                    owner::Owned::Heap(Body::Call { head, args }) => Ok((head, args)),
                }
            }
            pub fn release_iterative(self) {
                let mut pending = vec![self];
                while let Some(value) = pending.pop() {
                    if let Some(Body::Call { head, args }) = value.0.take() {
                        if let Some(head) = Arc::into_inner(head.0) {
                            pending.push(head);
                        }
                        if let Some(args) = Arc::into_inner(args.0) {
                            pending.extend(args);
                        }
                    }
                }
            }
        }
        impl Tree for $name {
            fn atom(value: Atom) -> Self {
                Self::atom(value)
            }
            fn call(head: Head<Self>, args: Args<Self>) -> Self {
                Self::call(head, args)
            }
            #[inline]
            fn view(&self) -> View<'_, Self> {
                self.view()
            }
            fn release_iterative(self) {
                self.release_iterative()
            }
        }
    };
}
representation!(Compact, Packed);
representation!(Safe, Safe);

pub fn from_expr<N: Tree>(expr: &Expr) -> N {
    conversion::from_expr(expr)
}
pub fn to_expr<N: Tree>(expr: &N) -> Expr {
    conversion::to_expr(expr)
}

const _: fn() = || {
    fn check<T: Send + Sync>() {}
    check::<Compact>();
    check::<Safe>();
};

#[cfg(test)]
mod tests;
