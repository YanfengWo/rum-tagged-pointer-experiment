//! Opt-in, conversion-local immutable Symbol payload sharing. No new unsafe.
//! Head/Args identities remain separate; full Symbol identity is the cache key.
use rum_core::{Atom, Expr, Symbol};
use rum_recursive_expr::Tree;
use std::collections::HashMap;
mod bridge;

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Stats {
    pub symbol_visits: usize,
    pub hits: usize,
    pub entries: usize,
    pub uncached: usize,
}

/// The same bridge algorithm with no symbol table or diagnostic increments.
pub fn plain<N: Tree>(expr: &Expr) -> N {
    bridge::convert(expr, &mut |atom| N::atom(atom.clone()))
}

/// Cache at most `limit` distinct full Symbol values during this conversion.
/// HashMap capacity may exceed its entry count. There is no global table, no
/// eviction and no cache lifetime after return. Other atoms are never cached.
/// A zero limit selects `plain` and returns zero diagnostic counters.
pub fn shared<N: Tree>(expr: &Expr, limit: usize) -> (N, Stats) {
    if limit == 0 {
        return (plain(expr), Stats::default());
    }
    let mut symbols: HashMap<Symbol, N> = HashMap::new();
    let mut stats = Stats::default();
    let result = bridge::convert(expr, &mut |atom| {
        let Atom::Symbol(symbol) = atom else {
            return N::atom(atom.clone());
        };
        stats.symbol_visits += 1;
        if let Some(node) = symbols.get(symbol) {
            stats.hits += 1;
            return node.clone();
        }
        let node = N::atom(atom.clone());
        if symbols.len() < limit {
            symbols.insert(*symbol, node.clone());
            stats.entries += 1;
        } else {
            stats.uncached += 1;
        }
        node
    });
    // The local map's extra owners drop here. Only graph owners survive.
    (result, stats)
}

#[cfg(test)]
mod tests;
