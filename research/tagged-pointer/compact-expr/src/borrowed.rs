//! Safe typed projections; constructing a view neither clones nor allocates.
use crate::ExprView;
use rum_core::{Atom, BigInt, Expr, IntValue, Rational, SharedArgs, SharedExpr, Symbol};

#[derive(Clone, Copy, Debug)]
pub enum IntRef<'a> {
    Small(i64),
    Big(&'a BigInt),
}

impl IntRef<'_> {
    /// Preserve IntValue::as_i64's variant contract, including a manual Big(7).
    pub fn as_i64(self) -> Option<i64> {
        match self {
            Self::Small(value) => Some(value),
            Self::Big(_) => None,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AtomRef<'a> {
    Integer(IntRef<'a>),
    Rational(&'a Rational),
    String(&'a str),
    Symbol(Symbol),
}

/// A read-only counterpart of rum's Expr enum. Call references retain the
/// original SharedExpr/SharedArgs handles, including pointer identity and Weak.
#[derive(Clone, Copy, Debug)]
pub enum ExprRef<'a> {
    Atom(AtomRef<'a>),
    Call {
        head: &'a SharedExpr,
        args: &'a SharedArgs,
    },
}

impl<'a> From<&'a Expr> for ExprRef<'a> {
    #[inline]
    fn from(expr: &'a Expr) -> Self {
        match expr {
            Expr::Atom(atom) => Self::Atom(match atom {
                Atom::Integer(IntValue::Small(value)) => AtomRef::Integer(IntRef::Small(*value)),
                Atom::Integer(IntValue::Big(value)) => AtomRef::Integer(IntRef::Big(value)),
                Atom::Rational(value) => AtomRef::Rational(value),
                Atom::String(value) => AtomRef::String(value),
                Atom::Symbol(value) => AtomRef::Symbol(*value),
            }),
            Expr::Call { head, args } => Self::Call { head, args },
        }
    }
}

impl<'a> From<ExprView<'a>> for ExprRef<'a> {
    #[inline]
    fn from(view: ExprView<'a>) -> Self {
        match view {
            ExprView::Integer(value) => Self::Atom(AtomRef::Integer(IntRef::Small(value))),
            ExprView::Heap(expr) => Self::from(expr),
        }
    }
}
