use crate::{collector, walk};
use rum_core::{Expr, Symbol};
use rum_layout_design::Tagged16;
use rum_typed_layout::{Safe24, Typed16};
use std::collections::hash_map::RandomState;
#[derive(Clone, Copy, Debug)]
pub enum Case {
    Original,
    Previous,
    SafeFull,
    TypedFull,
    SafeKey,
    TypedDerived,
    TypedKey,
}
impl Case {
    pub const ALL: [Self; 7] = [
        Self::Original,
        Self::Previous,
        Self::SafeFull,
        Self::TypedFull,
        Self::SafeKey,
        Self::TypedDerived,
        Self::TypedKey,
    ];
    pub fn name(self) -> &'static str {
        match self {
            Self::Original => "original",
            Self::Previous => "previous16-full",
            Self::SafeFull => "safe24-full",
            Self::TypedFull => "typed16-full",
            Self::SafeKey => "safe24-key",
            Self::TypedDerived => "typed16-derived",
            Self::TypedKey => "typed16-key",
        }
    }
}
pub enum Graph {
    Original(Expr),
    Previous(Tagged16),
    Safe(Safe24),
    Typed(Typed16),
}
impl Graph {
    pub fn build(case: Case, e: &Expr) -> Self {
        match case {
            Case::Original => Self::Original(rum_typed_layout::original_copy(e)),
            Case::Previous => Self::Previous(rum_layout_design::from_expr(e)),
            Case::SafeFull | Case::SafeKey => Self::Safe(rum_typed_layout::from_expr(e)),
            _ => Self::Typed(rum_typed_layout::from_expr(e)),
        }
    }
    pub fn collect(&self, case: Case, hint: usize, seed: &RandomState) -> Vec<Symbol> {
        match (self, case) {
            (Self::Original(n), Case::Original) => {
                collector::collect(walk::Original::new(n), hint, seed)
            }
            (Self::Previous(n), Case::Previous) => {
                collector::collect(walk::Previous::new(n), hint, seed)
            }
            (Self::Safe(n), Case::SafeFull) => collector::collect(walk::Full::new(n), hint, seed),
            (Self::Typed(n), Case::TypedFull) => collector::collect(walk::Full::new(n), hint, seed),
            (Self::Safe(n), Case::SafeKey) => {
                collector::collect(walk::Cached::<_, true>::new(n), hint, seed)
            }
            (Self::Typed(n), Case::TypedDerived) => {
                collector::collect(walk::Cached::<_, false>::new(n), hint, seed)
            }
            (Self::Typed(n), Case::TypedKey) => {
                collector::collect(walk::Cached::<_, true>::new(n), hint, seed)
            }
            _ => unreachable!(),
        }
    }
}
