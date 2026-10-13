//! Safe native Sort entry: no recursive graph conversion or callback execution.
#![deny(unsafe_op_in_unsafe_fn)]
#[allow(dead_code)]
mod slots;
#[cfg(not(any(feature = "safe16", feature = "tagged16")))]
use rum_core::Symbol;
use rum_core::{Atom, Expr};
#[cfg(any(test, feature = "safe16"))]
use slots::Safe16;
use slots::Slot;
#[cfg(any(test, feature = "tagged16"))]
use slots::Tagged16;
fn sort<S: Slot>(items: &[Expr]) -> Vec<Expr> {
    let mut values = items
        .iter()
        .map(|e| {
            let Expr::Atom(Atom::Symbol(s)) = e else {
                unreachable!()
            };
            S::new(*s)
        })
        .collect::<Vec<_>>();
    values.sort_by(Slot::compare);
    values
        .into_iter()
        .map(|s| Expr::Atom(Atom::Symbol(s.symbol())))
        .collect()
}
/// Admits only wide, entirely symbolic operands. Mixed/small values use the
/// original structural sort. Generated identities survive stable name ties.
pub fn try_sort_symbols(items: &[Expr]) -> Option<Vec<Expr>> {
    if items.len() < 64
        || !items
            .iter()
            .all(|e| matches!(e, Expr::Atom(Atom::Symbol(_))))
    {
        return None;
    }
    #[cfg(feature = "tagged16")]
    {
        Some(sort::<Tagged16>(items))
    }
    #[cfg(all(not(feature = "tagged16"), feature = "safe16"))]
    {
        Some(sort::<Safe16>(items))
    }
    #[cfg(not(any(feature = "tagged16", feature = "safe16")))]
    {
        Some(sort::<Symbol>(items))
    }
}
#[cfg(test)]
mod tests;
