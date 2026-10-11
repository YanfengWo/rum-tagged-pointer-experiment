use crate::{Args, Head, Storage, Tree, View, atom_ref, sealed};
use rum_core::expr::SymbolText;
use rum_core::{Atom, IntValue, Symbol};
use std::{num::NonZeroU64, sync::Arc};

/// Safe Rust control with the same storage decisions as Tagged16.
#[derive(Clone)]
enum Repr {
    Small(i64),
    Symbol {
        text: &'static SymbolText,
        identity: Option<NonZeroU64>,
    },
    Owned(Arc<Atom>),
    Call {
        head: Head<Safe24>,
        args: Args<Safe24>,
    },
}
#[derive(Clone)]
pub struct Safe24(Repr);
impl sealed::Sealed for Safe24 {}
impl Tree for Safe24 {
    #[inline]
    fn atom(atom: Atom) -> Self {
        Self(match atom {
            Atom::Integer(IntValue::Small(n)) => Repr::Small(n),
            Atom::Symbol(s) => Repr::Symbol {
                text: s.interned_text(),
                identity: s.identity().and_then(NonZeroU64::new),
            },
            atom => Repr::Owned(Arc::new(atom)),
        })
    }
    #[inline]
    fn call(head: Head<Self>, args: Args<Self>) -> Self {
        Self(Repr::Call { head, args })
    }
    #[inline]
    fn view(&self) -> View<'_, Self> {
        match &self.0 {
            Repr::Small(n) => View::Atom(crate::AtomRef::Integer(crate::IntRef::Small(*n))),
            Repr::Symbol { text, identity } => View::Atom(crate::AtomRef::Symbol(
                Symbol::from_interned_text(text, *identity),
            )),
            Repr::Owned(atom) => View::Atom(atom_ref(atom)),
            Repr::Call { head, args } => View::Call {
                head: head.borrow(),
                args: args.borrow(),
            },
        }
    }
    fn into_parts(self) -> Result<(Head<Self>, Args<Self>), Atom> {
        match self.0 {
            Repr::Small(n) => Err(Atom::Integer(IntValue::Small(n))),
            Repr::Symbol { text, identity } => {
                Err(Atom::Symbol(Symbol::from_interned_text(text, identity)))
            }
            Repr::Owned(atom) => Err(Arc::try_unwrap(atom).unwrap_or_else(|a| a.as_ref().clone())),
            Repr::Call { head, args } => Ok((head, args)),
        }
    }
    fn release_iterative(self) {
        crate::release(self)
    }
}
impl Storage for Safe24 {
    fn take_call(self) -> Option<(Head<Self>, Args<Self>)> {
        if let Repr::Call { head, args } = self.0 {
            Some((head, args))
        } else {
            None
        }
    }
}
