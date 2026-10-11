//! An opt-in bounded prefix heuristic. It can miss late repetition or enable
//! sharing for an unrepresentative prefix; it is not a profitability guarantee.
use rum_core::{Atom, Expr, SharedArgs, SharedExpr};
use rum_recursive_expr::Tree;
use rum_symbol_sharing::Stats;
use std::collections::HashSet;

pub const MIN_ROOT_ARGS: usize = 64;
pub const NODE_BUDGET: usize = 256;
pub const MIN_REPEATS: usize = 8;
pub const CACHE_LIMIT: usize = 128;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    SmallRoot,
    Plain,
    Shared,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Probe {
    pub decision: Decision,
    pub nodes: usize,
    pub symbols: usize,
    pub repeats: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Report {
    pub probe: Probe,
    pub sharing: Option<Stats>,
}
enum Work<'a> {
    Node(&'a Expr),
    Head(&'a SharedExpr),
    List(&'a SharedArgs),
    Args(std::slice::Iter<'a, Expr>),
}

/// Preorder prefix of physical slots, including Call nodes. Shared Head/Args
/// storage is visited once. Lazy Args iterators avoid scanning or enqueueing a
/// whole wide vector. Scratch owners are gone before graph construction.
pub fn probe(expr: &Expr) -> Probe {
    let mut result = Probe {
        decision: Decision::SmallRoot,
        nodes: 0,
        symbols: 0,
        repeats: 0,
    };
    if !matches!(expr, Expr::Call { args, .. } if args.len() >= MIN_ROOT_ARGS) {
        return result;
    }
    result.decision = Decision::Plain;
    let mut pending = vec![Work::Node(expr)];
    let mut heads = HashSet::new();
    let mut lists = HashSet::new();
    let mut symbols = HashSet::new();
    while result.nodes < NODE_BUDGET {
        let Some(work) = pending.pop() else { break };
        match work {
            Work::Head(head) => {
                if heads.insert(head.ptr_key()) {
                    pending.push(Work::Node(head));
                }
            }
            Work::List(args) => {
                if lists.insert(args.ptr_key()) {
                    pending.push(Work::Args(args.iter()));
                }
            }
            Work::Args(mut args) => {
                if let Some(node) = args.next() {
                    pending.extend([Work::Args(args), Work::Node(node)]);
                }
            }
            Work::Node(node) => {
                result.nodes += 1;
                match node {
                    Expr::Atom(Atom::Symbol(symbol)) => {
                        result.symbols += 1;
                        if !symbols.insert(*symbol) {
                            result.repeats += 1;
                            if result.repeats == MIN_REPEATS {
                                result.decision = Decision::Shared;
                                break;
                            }
                        }
                    }
                    Expr::Call { head, args } => {
                        pending.extend([Work::List(args), Work::Head(head)]);
                    }
                    _ => {}
                }
            }
        }
    }
    result
}

/// Includes the fresh probe on every conversion; no result or table is cached.
/// The two branches compose the unchanged prior safe conversion APIs.
pub fn selective<N: Tree>(expr: &Expr) -> (N, Report) {
    let probe = probe(expr);
    let (node, sharing) = if probe.decision == Decision::Shared {
        let (node, stats) = rum_symbol_sharing::shared(expr, CACHE_LIMIT);
        (node, Some(stats))
    } else {
        (rum_symbol_sharing::plain(expr), None)
    };
    (node, Report { probe, sharing })
}

#[cfg(test)]
mod tests;
