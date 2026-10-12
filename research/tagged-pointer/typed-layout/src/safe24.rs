//! Same cached-symbol/typed-payload policy as Typed16, with safe enum storage.
use crate::{Args, AtomRef, Head, Inspect, IntRef, Storage, SymbolKey, Tree, View, sealed};
use rum_core::{Atom, BigInt, IntValue, Rational, Symbol, expr::SymbolText};
use std::{num::NonZeroU64, sync::Arc};
#[derive(Clone)]
enum Repr {
    Small(i64),
    Plain {
        text: &'static SymbolText,
        id: u32,
    },
    Generated {
        text: &'static SymbolText,
        identity: NonZeroU64,
    },
    String(Arc<str>),
    Big(Arc<BigInt>),
    Rational(Arc<Rational>),
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
            Atom::Integer(IntValue::Big(n)) => Repr::Big(Arc::new(*n)),
            Atom::Rational(n) => Repr::Rational(Arc::new(*n)),
            Atom::String(s) => Repr::String(Arc::from(s)),
            Atom::Symbol(s) => match s.identity().and_then(NonZeroU64::new) {
                None => Repr::Plain {
                    text: s.interned_text(),
                    id: s.interned_id(),
                },
                Some(identity) => Repr::Generated {
                    text: s.interned_text(),
                    identity,
                },
            },
        })
    }
    #[inline]
    fn borrowed_atom(atom: &Atom) -> Self {
        match atom {
            Atom::String(s) => Self(Repr::String(Arc::from(s.as_str()))),
            Atom::Integer(IntValue::Big(n)) => Self(Repr::Big(Arc::new((**n).clone()))),
            Atom::Rational(n) => Self(Repr::Rational(Arc::new((**n).clone()))),
            other => Self::atom(other.clone()),
        }
    }
    #[inline]
    fn call(head: Head<Self>, args: Args<Self>) -> Self {
        Self(Repr::Call { head, args })
    }
    #[inline]
    fn view(&self) -> View<'_, Self> {
        View::Atom(match &self.0 {
            Repr::Small(n) => AtomRef::Integer(IntRef::Small(*n)),
            Repr::Plain { text, .. } => AtomRef::Symbol(Symbol::from_interned_text(text, None)),
            Repr::Generated { text, identity } => {
                AtomRef::Symbol(Symbol::from_interned_text(text, Some(*identity)))
            }
            Repr::String(s) => AtomRef::String(s),
            Repr::Big(n) => AtomRef::Integer(IntRef::Big(n)),
            Repr::Rational(n) => AtomRef::Rational(n),
            Repr::Call { head, args } => {
                return View::Call {
                    head: head.borrow(),
                    args: args.borrow(),
                };
            }
        })
    }
    #[inline]
    fn inspect(&self) -> Inspect<'_, Self> {
        match &self.0 {
            Repr::Small(n) => Inspect::Small(*n),
            Repr::Plain { id, .. } => Inspect::Symbol(SymbolKey::new(*id, None)),
            Repr::Generated { text, identity } => {
                Inspect::Symbol(SymbolKey::new(text.interned_id(), Some(*identity)))
            }
            Repr::String(s) => Inspect::String(s),
            Repr::Big(n) => Inspect::Big(n),
            Repr::Rational(n) => Inspect::Rational(n),
            Repr::Call { head, args } => Inspect::Call {
                head: head.borrow(),
                args: args.borrow(),
            },
        }
    }
    fn into_parts(self) -> Result<(Head<Self>, Args<Self>), Atom> {
        Err(match self.0 {
            Repr::Small(n) => Atom::Integer(IntValue::Small(n)),
            Repr::Plain { text, .. } => Atom::Symbol(Symbol::from_interned_text(text, None)),
            Repr::Generated { text, identity } => {
                Atom::Symbol(Symbol::from_interned_text(text, Some(identity)))
            }
            Repr::String(s) => Atom::String(s.to_string()),
            Repr::Big(n) => Atom::Integer(IntValue::Big(Box::new(
                Arc::try_unwrap(n).unwrap_or_else(|n| n.as_ref().clone()),
            ))),
            Repr::Rational(n) => Atom::Rational(Box::new(
                Arc::try_unwrap(n).unwrap_or_else(|n| n.as_ref().clone()),
            )),
            Repr::Call { head, args } => return Ok((head, args)),
        })
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
