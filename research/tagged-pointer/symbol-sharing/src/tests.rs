use super::*;
use rum_core::{BigInt, IntValue, SharedArgs, SharedExpr};
use rum_recursive_expr::{Args, AtomRef, Compact, IntRef, Safe, View, to_expr};

fn check<N: Tree>(expr: &Expr, limit: usize) -> (N, Stats) {
    let (node, stats) = shared::<N>(expr, limit);
    assert_eq!(to_expr(&node), *expr);
    assert_eq!(
        stats.symbol_visits,
        stats.hits + stats.entries + stats.uncached
    );
    assert!(stats.entries <= limit);
    (node, stats)
}
fn repeated<N: Tree>() {
    let expr = Expr::call(Expr::small_integer(0), vec![Expr::symbol("a"); 8]);
    let (node, stats) = check::<N>(&expr, 4);
    assert_eq!(
        stats,
        Stats {
            symbol_visits: 8,
            hits: 7,
            entries: 1,
            uncached: 0
        }
    );
    assert_eq!(to_expr(&node.clone()), expr);
    node.release_iterative();
}
#[test]
fn repeated_symbols_share_then_release_for_both_owners() {
    repeated::<Compact>();
    repeated::<Safe>();
}
fn identities<N: Tree>() {
    let symbols = [
        Symbol::new("Global`", "x"),
        Symbol::new("Other`", "x"),
        Symbol::with_identity("Global`", "x", 1),
        Symbol::with_identity("Global`", "x", 2),
    ];
    let expr = Expr::call(
        Expr::small_integer(0),
        symbols
            .into_iter()
            .chain(symbols)
            .map(|symbol| Expr::Atom(Atom::Symbol(symbol)))
            .collect(),
    );
    let (node, stats) = check::<N>(&expr, 16);
    assert_eq!((stats.entries, stats.hits), (4, 4));
    let View::Call { args, .. } = node.view() else {
        panic!()
    };
    for (i, symbol) in symbols.iter().enumerate() {
        assert!(matches!(args[i].view(), View::Atom(AtomRef::Symbol(s)) if s == *symbol));
    }
}
#[test]
fn full_context_and_generated_identity_are_keys() {
    identities::<Compact>();
    identities::<Safe>();
}
fn aliases<N: Tree>() {
    let head = SharedExpr::new(Expr::symbol("f"));
    let args = SharedArgs::new(vec![Expr::symbol("a")]);
    let expr = Expr::call(
        Expr::small_integer(0),
        vec![
            Expr::call_shared(head.clone(), args.clone()),
            Expr::call_shared(head, args),
            Expr::call(Expr::symbol("f"), vec![Expr::symbol("a")]),
        ],
    );
    let (node, stats) = check::<N>(&expr, 8);
    assert_eq!((stats.entries, stats.hits), (2, 2));
    let View::Call { args, .. } = node.view() else {
        panic!()
    };
    let View::Call { head: h0, args: a0 } = args[0].view() else {
        panic!()
    };
    let View::Call { head: h1, args: a1 } = args[1].view() else {
        panic!()
    };
    let View::Call { head: h2, args: a2 } = args[2].view() else {
        panic!()
    };
    assert!(h0.ptr_eq(h1) && a0.ptr_eq(a1));
    assert!(!h0.ptr_eq(h2) && !a0.ptr_eq(a2));
    let weak = a0.downgrade();
    assert!(weak.is_alive());
    let mut changed = a0.clone();
    changed.make_mut()[0] = N::atom(Atom::Symbol(Symbol::from_name("changed")));
    assert!(!weak.ptr_eq(&changed) && weak.is_alive());
    assert!(matches!(a0[0].view(), View::Atom(AtomRef::Symbol(s)) if s.name() == "a"));
    drop(node);
    assert!(!weak.is_alive());
    // Weak-only COW invalidates the old handle even though no vector clone is needed.
    let mut unique = Args::new(vec![N::atom(Atom::Symbol(Symbol::from_name("a")))]);
    let weak = unique.downgrade();
    unique
        .make_mut()
        .push(N::atom(Atom::Symbol(Symbol::from_name("a"))));
    assert!(!weak.is_alive() && !weak.ptr_eq(&unique));
}
#[test]
fn head_args_alias_classes_cow_and_weak_are_preserved() {
    aliases::<Compact>();
    aliases::<Safe>();
}
fn limits<N: Tree>() {
    let expr = Expr::call(
        Expr::small_integer(0),
        ["a", "b", "c", "a", "c"].map(Expr::symbol).to_vec(),
    );
    let (_, stats) = check::<N>(&expr, 2);
    assert_eq!(
        stats,
        Stats {
            symbol_visits: 5,
            hits: 1,
            entries: 2,
            uncached: 2
        }
    );
    let (node, stats) = shared::<N>(&expr, 0);
    assert_eq!(stats, Stats::default());
    assert_eq!(to_expr(&node), expr);
    assert_eq!(to_expr(&plain::<N>(&expr)), expr);
}
#[test]
fn zero_limit_and_full_cache_fallback_are_correct() {
    limits::<Compact>();
    limits::<Safe>();
}
fn variants<N: Tree>() {
    let expr = Expr::call(
        Expr::symbol("f"),
        vec![
            Expr::small_integer(i64::MAX),
            Expr::Atom(Atom::Integer(IntValue::Big(Box::new(BigInt::from(7))))),
            Expr::string("owned\0text"),
            Expr::symbol("f"),
        ],
    );
    let (node, stats) = check::<N>(&expr, 4);
    assert_eq!(stats.symbol_visits, 2);
    let View::Call { args, .. } = node.view() else {
        panic!()
    };
    assert!(
        matches!(args[1].view(), View::Atom(AtomRef::Integer(IntRef::Big(n))) if *n == BigInt::from(7))
    );
}
#[test]
fn non_symbol_atoms_keep_their_original_variants() {
    variants::<Compact>();
    variants::<Safe>();
}
fn deep<N: Tree>() {
    let depth = if cfg!(miri) { 24 } else { 10_000 };
    let mut expr = Expr::small_integer(7);
    for _ in 0..depth {
        expr = Expr::call(Expr::symbol("f"), vec![expr]);
    }
    let (node, stats) = shared::<N>(&expr, 4);
    assert_eq!((stats.entries, stats.hits), (1, depth - 1));
    node.clone().release_iterative();
    node.release_iterative();
    while let Expr::Call { args, .. } = expr {
        expr = args.into_vec().pop().unwrap();
    }
}
#[test]
fn deep_explicit_release_handles_shared_symbol_payloads() {
    deep::<Compact>();
    deep::<Safe>();
}
fn dag<N: Tree>() {
    let head = SharedExpr::new(Expr::symbol("f"));
    let mut expr = Expr::small_integer(7);
    for _ in 0..80 {
        expr = Expr::call_shared(head.clone(), SharedArgs::new(vec![expr.clone(), expr]));
    }
    let (node, stats) = shared::<N>(&expr, 4);
    assert_eq!((stats.symbol_visits, stats.entries, stats.hits), (1, 1, 0));
    let mut current = &node;
    for _ in 0..80 {
        let View::Call { args, .. } = current.view() else {
            panic!()
        };
        current = &args[0];
    }
    assert!(matches!(
        current.view(),
        View::Atom(AtomRef::Integer(IntRef::Small(7)))
    ));
    node.release_iterative();
}
#[test]
fn nested_dag_is_not_unfolded_by_conversion() {
    dag::<Compact>();
    dag::<Safe>();
}
#[test]
fn consuming_one_cached_atom_keeps_its_sibling_valid() {
    macro_rules! exercise {
        ($owner:ty) => {{
            let expr = Expr::call(Expr::small_integer(0), vec![Expr::symbol("a"); 2]);
            let (node, _) = shared::<$owner>(&expr, 1);
            let (_, args) = node.into_parts().unwrap();
            let mut values = args.into_vec();
            let symbol = values.pop().unwrap().into_parts().err().unwrap();
            assert!(matches!(symbol, Atom::Symbol(s) if s.name() == "a"));
            assert!(matches!(values[0].view(), View::Atom(AtomRef::Symbol(s)) if s.name() == "a"));
        }};
    }
    exercise!(Compact);
    exercise!(Safe);
}
