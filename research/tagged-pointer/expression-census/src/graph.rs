//! Physical storage census and memoized logical shape; no DAG unfolding.
use rum_core::{Atom, Expr, IntValue, Symbol};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Default, Clone, Serialize)]
pub struct Counts {
    pub inline_integer: u64,
    pub other_small_integer: u64,
    pub big_integer: u64,
    pub rational: u64,
    pub string: u64,
    pub symbol: u64,
    pub call: u64,
}
impl Counts {
    pub fn total(&self) -> u64 {
        self.inline_integer
            .saturating_add(self.other_small_integer)
            .saturating_add(self.big_integer)
            .saturating_add(self.rational)
            .saturating_add(self.string)
            .saturating_add(self.symbol)
            .saturating_add(self.call)
    }
    pub(crate) fn add(&mut self, other: &Self, saturated: &mut bool) {
        macro_rules! field {
            ($name:ident) => {
                let (sum, overflow) = self.$name.overflowing_add(other.$name);
                self.$name = if overflow { u64::MAX } else { sum };
                *saturated |= overflow;
            };
        }
        field!(inline_integer);
        field!(other_small_integer);
        field!(big_integer);
        field!(rational);
        field!(string);
        field!(symbol);
        field!(call);
    }
    pub(crate) fn one(expr: &Expr) -> Self {
        let mut counts = Self::default();
        match expr {
            Expr::Call { .. } => counts.call = 1,
            Expr::Atom(Atom::Integer(IntValue::Small(value)))
                if (-(1_i64 << 60)..(1_i64 << 60)).contains(value) =>
            {
                counts.inline_integer = 1
            }
            Expr::Atom(Atom::Integer(IntValue::Small(_))) => counts.other_small_integer = 1,
            Expr::Atom(Atom::Integer(IntValue::Big(_))) => counts.big_integer = 1,
            Expr::Atom(Atom::Rational(_)) => counts.rational = 1,
            Expr::Atom(Atom::String(_)) => counts.string = 1,
            Expr::Atom(Atom::Symbol(_)) => counts.symbol = 1,
        }
        counts
    }
}

#[derive(Serialize)]
pub struct Census {
    pub physical: Counts,
    pub physical_nodes: u64,
    pub logical: Counts,
    pub logical_nodes: u64,
    pub logical_counts_saturated: bool,
    pub max_depth_nodes: usize,
    pub unique_heads: usize,
    pub unique_argument_lists: usize,
    pub repeated_head_handles: usize,
    pub repeated_argument_handles: usize,
    pub argument_length_sum: usize,
    pub argument_capacity_sum: usize,
    pub unique_symbols: usize,
    pub generated_symbol_occurrences: usize,
    pub unique_string_values: usize,
    pub string_length_sum: usize,
    pub string_capacity_sum: usize,
    pub max_arity: usize,
    pub call_arity_histogram: [usize; 7],
}

pub fn census(root: &Expr) -> Census {
    let mut physical = Counts::default();
    let mut heads = HashSet::new();
    let mut lists = HashSet::new();
    let mut symbols: HashSet<Symbol> = HashSet::new();
    let mut strings = HashSet::new();
    let (mut generated, mut string_len, mut string_cap, mut arg_len, mut arg_cap, mut max_arity) =
        (0, 0, 0, 0, 0, 0);
    let mut arities = [0; 7];
    let mut pending = vec![root];
    while let Some(node) = pending.pop() {
        physical.add(&Counts::one(node), &mut false);
        match node {
            Expr::Atom(Atom::Symbol(symbol)) => {
                symbols.insert(*symbol);
                generated += usize::from(symbol.identity().is_some());
            }
            Expr::Atom(Atom::String(text)) => {
                strings.insert(text.as_str());
                string_len += text.len();
                string_cap += text.capacity();
            }
            Expr::Call { head, args } => {
                let n = args.len();
                max_arity = max_arity.max(n);
                arities[match n {
                    0 => 0,
                    1 => 1,
                    2 => 2,
                    3..=8 => 3,
                    9..=64 => 4,
                    65..=1024 => 5,
                    _ => 6,
                }] += 1;
                if heads.insert(head.ptr_key()) {
                    pending.push(head);
                }
                if lists.insert(args.ptr_key()) {
                    arg_len += n;
                    arg_cap += args.capacity();
                    pending.extend(args.iter());
                }
            }
            _ => {}
        }
    }
    assert_eq!(physical.total() as usize, 1 + heads.len() + arg_len);
    let (logical, depth, saturated) = crate::shape::logical(root);
    let calls = physical.call as usize;
    Census {
        physical_nodes: physical.total(),
        logical_nodes: logical.total(),
        physical,
        logical,
        logical_counts_saturated: saturated,
        max_depth_nodes: depth,
        unique_heads: heads.len(),
        unique_argument_lists: lists.len(),
        repeated_head_handles: calls - heads.len(),
        repeated_argument_handles: calls - lists.len(),
        argument_length_sum: arg_len,
        argument_capacity_sum: arg_cap,
        unique_symbols: symbols.len(),
        generated_symbol_occurrences: generated,
        unique_string_values: strings.len(),
        string_length_sum: string_len,
        string_capacity_sum: string_cap,
        max_arity,
        call_arity_histogram: arities,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rum_core::{BigInt, SharedArgs, SharedExpr};
    #[test]
    fn shared_physical_slots_and_logical_multiplicity_differ() {
        let head = SharedExpr::new(Expr::symbol("f"));
        let args = SharedArgs::new(vec![Expr::small_integer(7), Expr::string("x")]);
        let child = Expr::call_shared(head.clone(), args);
        let root = Expr::call_shared(head, SharedArgs::new(vec![child.clone(), child]));
        let c = census(&root);
        assert_eq!(c.physical_nodes, 6);
        assert_eq!(c.logical_nodes, 10);
        assert_eq!(c.physical.call, 3);
        assert_eq!(c.physical.inline_integer, 1);
        assert_eq!(c.unique_heads, 1);
        assert_eq!(c.repeated_head_handles, 2);
        assert_eq!(c.unique_argument_lists, 2);
        assert_eq!(c.repeated_argument_handles, 1);
        assert_eq!(c.max_depth_nodes, 3);
    }
    #[test]
    fn nested_dag_depth_and_saturating_counts_without_expansion() {
        let head = SharedExpr::new(Expr::small_integer(0));
        let mut node = Expr::small_integer(7);
        for _ in 0..80 {
            node = Expr::call_shared(head.clone(), SharedArgs::new(vec![node.clone(), node]));
        }
        let c = census(&node);
        assert_eq!(c.max_depth_nodes, 81);
        assert_eq!(c.physical_nodes, 162);
        assert_eq!(c.unique_heads, 1);
        assert_eq!(c.unique_argument_lists, 80);
        assert!(c.logical_counts_saturated);
        assert_eq!(c.logical_nodes, u64::MAX);
    }
    #[test]
    fn variants_identity_and_capacity_are_counted_without_normalization() {
        let mut args = Vec::with_capacity(16);
        args.extend([
            Expr::small_integer(i64::MAX),
            Expr::Atom(Atom::Integer(IntValue::Big(Box::new(BigInt::from(7))))),
            Expr::Atom(Atom::Symbol(Symbol::with_identity("Global`", "x", 1))),
            Expr::Atom(Atom::Symbol(Symbol::with_identity("Global`", "x", 2))),
            Expr::symbol("x"),
        ]);
        let c = census(&Expr::call(Expr::symbol("f"), args));
        assert_eq!(c.argument_capacity_sum, 16);
        assert_eq!(c.argument_length_sum, 5);
        assert_eq!(c.physical.other_small_integer, 1);
        assert_eq!(c.physical.big_integer, 1);
        assert_eq!(c.unique_symbols, 4);
        assert_eq!(c.generated_symbol_occurrences, 2);
    }
}
