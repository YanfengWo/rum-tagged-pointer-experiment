//! Separate non-inlined projections for assembly inspection, not timing loops.
use rum_core::{Atom, Expr};

fn string_length(expr: &Expr) -> Option<usize> {
    if let Expr::Atom(Atom::String(value)) = expr {
        Some(value.len())
    } else {
        None
    }
}

#[inline(never)]
pub fn inspect_v1_string_length(value: &compact_v1::CompactExpr) -> Option<usize> {
    match value.view() {
        compact_v1::ExprView::Heap(expr) => string_length(expr),
        _ => None,
    }
}

#[inline(never)]
pub fn inspect_v2_string_length(value: &rum_compact_expr::CompactExpr) -> Option<usize> {
    match value.view() {
        rum_compact_expr::ExprView::Heap(expr) => string_length(expr),
        _ => None,
    }
}

#[inline(never)]
pub fn inspect_v1_hinted_string_length(value: &compact_v1_hinted::CompactExpr) -> Option<usize> {
    match value.view() {
        compact_v1_hinted::ExprView::Heap(expr) => string_length(expr),
        _ => None,
    }
}
