//! Direct interned Symbols and independent Call handles. Public constructors
//! are safe; raw representation details stay inside the closed owner modules.
//! Requires the reviewed additive Symbol access patch in the isolated overlay.
//!
//! ```compile_fail
//! use rum_typed_layout::{Typed16,Tree,View};
//! fn escape() -> &'static str {
//!     let n=Typed16::atom(rum_core::Atom::String("x".into()));
//!     match n.view() { View::Atom(rum_typed_layout::AtomRef::String(s))=>s,_=>panic!() }
//! }
//! ```
//! ```compile_fail
//! use rum_typed_layout::{Args,Typed16,Tree};
//! let mut a=Args::new(vec![Typed16::atom(rum_core::Atom::String("x".into()))]);
//! let borrowed=&a[0];
//! a.make_mut().clear();
//! let _=borrowed.view();
//! ```
//! ```compile_fail
//! use rum_typed_layout::{Typed16,Tree,View,Head,Args};
//! fn escape() -> &'static Typed16 {
//!     let n=Typed16::call(Head::new(Typed16::small(0)),Args::new(vec![]));
//!     match n.view() { View::Call{head,..}=>head.node(),_=>panic!() }
//! }
//! ```
//! ```compile_fail
//! use rum_typed_layout::{Typed16,Tree,View,Head,Args};
//! let n=Typed16::call(Head::new(Typed16::small(0)),Args::new(vec![]));
//! let View::Call{head,..}=n.view() else {panic!()};
//! let _=n.into_parts();
//! let _=head.node();
//! ```
//! ```compile_fail
//! use rum_typed_layout::{Typed16,Inspect};
//! fn escape() -> &'static str {
//!     let n=Typed16::atom(rum_core::Atom::String("value".into()));
//!     match n.inspect() { Inspect::String(s)=>s,_=>panic!() }
//! }
//! ```
//! ```compile_fail
//! use rum_typed_layout::{Typed16,Inspect};
//! let n=Typed16::atom(rum_core::Atom::String("value".into()));
//! let Inspect::String(s)=n.inspect() else {panic!()};
//! let _=n.into_parts();
//! println!("{}",s);
//! ```
#![deny(unsafe_op_in_unsafe_fn)]
#[cfg(not(target_pointer_width = "64"))]
compile_error!("This experiment requires 64-bit pointers");
pub use rum_compact_expr::{AtomRef, IntRef};
use rum_core::{Atom, IntValue};
use std::{
    ops::Deref,
    sync::{Arc, Weak},
};
mod borrow;
mod bridge;
mod inspect;
mod safe24;
mod typed16;
pub use borrow::{ArgsRef, HeadRef};
pub use bridge::{from_expr, from_expr_moved, original_copy, to_expr};
pub use inspect::{Inspect, SymbolKey};
pub use safe24::Safe24;
pub use typed16::Typed16;

#[derive(Clone)]
pub struct Head<N>(pub(crate) Arc<N>);
impl<N> Head<N> {
    pub fn new(n: N) -> Self {
        Self(Arc::new(n))
    }
    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
    pub fn borrow(&self) -> HeadRef<'_, N> {
        // SAFETY: this Arc keeps its exact payload alive for the borrow.
        unsafe { HeadRef::from_raw(Arc::as_ptr(&self.0)) }
    }
}
impl<N> Deref for Head<N> {
    type Target = N;
    fn deref(&self) -> &N {
        &self.0
    }
}
#[derive(Clone)]
pub struct Args<N>(pub(crate) Arc<Vec<N>>);
impl<N> Args<N> {
    pub fn new(v: Vec<N>) -> Self {
        Self(Arc::new(v))
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
    pub fn borrow(&self) -> ArgsRef<'_, N> {
        // SAFETY: this Arc keeps its exact Vec payload alive for the borrow.
        unsafe { ArgsRef::from_raw(Arc::as_ptr(&self.0)) }
    }
}
impl<N: Clone> Args<N> {
    pub fn make_mut(&mut self) -> &mut Vec<N> {
        Arc::make_mut(&mut self.0)
    }
    pub fn into_vec(self) -> Vec<N> {
        Arc::try_unwrap(self.0).unwrap_or_else(|a| a.as_ref().clone())
    }
}
impl<N> Deref for Args<N> {
    type Target = [N];
    fn deref(&self) -> &[N] {
        &self.0
    }
}
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
        head: HeadRef<'a, N>,
        args: ArgsRef<'a, N>,
    },
}
#[inline]
pub(crate) fn atom_ref(atom: &Atom) -> AtomRef<'_> {
    match atom {
        Atom::Integer(IntValue::Small(n)) => AtomRef::Integer(IntRef::Small(*n)),
        Atom::Integer(IntValue::Big(n)) => AtomRef::Integer(IntRef::Big(n)),
        Atom::Rational(n) => AtomRef::Rational(n),
        Atom::String(n) => AtomRef::String(n),
        Atom::Symbol(n) => AtomRef::Symbol(*n),
    }
}
pub(crate) mod sealed {
    pub trait Sealed {}
}
pub trait Tree: sealed::Sealed + Clone + Sized {
    fn atom(atom: Atom) -> Self;
    fn borrowed_atom(atom: &Atom) -> Self {
        Self::atom(atom.clone())
    }
    fn inspect(&self) -> Inspect<'_, Self>;
    fn call(head: Head<Self>, args: Args<Self>) -> Self;
    fn view(&self) -> View<'_, Self>;
    fn into_parts(self) -> Result<(Head<Self>, Args<Self>), Atom>;
    fn release_iterative(self);
}
pub(crate) trait Storage: Sized {
    fn take_call(self) -> Option<(Head<Self>, Args<Self>)>;
}
pub(crate) fn release<N: Tree + Storage>(root: N) {
    let mut pending = vec![root];
    while let Some(node) = pending.pop() {
        if let Some((head, args)) = node.take_call() {
            if let Some(head) = Arc::into_inner(head.0) {
                pending.push(head)
            }
            if let Some(args) = Arc::into_inner(args.0) {
                pending.extend(args)
            }
        }
    }
}
#[cfg(test)]
mod tests;
#[cfg(test)]
mod typed_tests;
