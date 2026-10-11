use super::*;
use rum_core::{SharedArgs, SharedExpr, Symbol};
use rum_recursive_expr::{AtomRef, Compact, Safe, View, to_expr};

fn check<N: Tree>(expr: &Expr) -> (N, Report) {
    let (node, report) = selective::<N>(expr);
    assert_eq!(to_expr(&node), *expr);
    assert!(report.probe.nodes <= NODE_BUDGET);
    assert!(report.probe.repeats <= MIN_REPEATS);
    assert_eq!(
        report.sharing.is_some(),
        report.probe.decision == Decision::Shared
    );
    (node, report)
}
fn both(expr: &Expr, wanted: Decision) -> Report {
    let (compact, c) = check::<Compact>(expr);
    let (safe, s) = check::<Safe>(expr);
    assert_eq!(c, s);
    assert_eq!(c.probe.decision, wanted);
    compact.release_iterative();
    safe.release_iterative();
    c
}
fn list(nodes: Vec<Expr>) -> Expr {
    Expr::call(Expr::symbol("f"), nodes)
}
#[test]
fn small_root_skips_probe_even_if_all_symbols_repeat() {
    let r = both(&list(vec![Expr::symbol("f"); 63]), Decision::SmallRoot);
    assert_eq!((r.probe.nodes, r.probe.symbols, r.probe.repeats), (0, 0, 0));
    both(&Expr::symbol("f"), Decision::SmallRoot);
}
#[test]
fn unique_prefix_exhausts_exact_budget_and_uses_plain() {
    let r = both(
        &list((0..300).map(|n| Expr::symbol(&format!("u{n}"))).collect()),
        Decision::Plain,
    );
    assert_eq!(
        (r.probe.nodes, r.probe.symbols, r.probe.repeats),
        (256, 255, 0)
    );
}
#[test]
fn repeated_prefix_stops_at_eight_and_uses_bounded_sharing() {
    let r = both(&list(vec![Expr::symbol("f"); 64]), Decision::Shared);
    assert_eq!(
        (r.probe.nodes, r.probe.symbols, r.probe.repeats),
        (10, 9, 8)
    );
    assert_eq!(r.sharing.unwrap().hits, 64);
}
#[test]
fn late_repetition_is_explicitly_missed() {
    let mut nodes: Vec<_> = (0..300)
        .map(|n| Expr::symbol(&format!("late{n}")))
        .collect();
    nodes.extend(vec![Expr::symbol("f"); 300]);
    let expr = list(nodes);
    both(&expr, Decision::Plain);
    let (node, stats) = rum_symbol_sharing::shared::<Compact>(&expr, CACHE_LIMIT);
    assert_eq!(stats.hits, 300); // The head's cached f would serve the late tail.
    node.release_iterative();
}
#[test]
fn early_repetition_can_select_an_unrepresentative_prefix() {
    let mut nodes = vec![Expr::symbol("f"); 8];
    nodes.extend((0..300).map(|n| Expr::symbol(&format!("tail{n}"))));
    let r = both(&list(nodes), Decision::Shared);
    let stats = r.sharing.unwrap();
    assert_eq!((stats.hits, stats.entries, stats.uncached), (8, 128, 173));
}
#[test]
fn full_symbol_identity_is_used_by_probe_and_shared_branch() {
    let identities = [
        Symbol::new("Global`", "x"),
        Symbol::new("Other`", "x"),
        Symbol::with_identity("Global`", "x", 1),
        Symbol::with_identity("Global`", "x", 2),
    ];
    let expr = Expr::call(
        Expr::small_integer(0),
        (0..64)
            .map(|n| Expr::Atom(Atom::Symbol(identities[n % 4])))
            .collect(),
    );
    let r = both(&expr, Decision::Shared);
    assert_eq!(r.sharing.unwrap().entries, 4);
    assert_eq!(r.probe.symbols, 12);
}
#[test]
fn shared_storage_does_not_inflate_probe_repetition() {
    let head = SharedExpr::new(Expr::symbol("f"));
    let args = SharedArgs::new(vec![Expr::symbol("a")]);
    let calls = vec![Expr::call_shared(head, args); 64];
    let r = both(&Expr::call(Expr::small_integer(0), calls), Decision::Plain);
    assert_eq!(
        (r.probe.nodes, r.probe.symbols, r.probe.repeats),
        (68, 2, 0)
    );
}
fn aliases<N: Tree>() {
    let head = SharedExpr::new(Expr::symbol("f"));
    let args = SharedArgs::new(vec![Expr::symbol("a"); 64]);
    let mut nodes = vec![
        Expr::call_shared(head.clone(), args.clone()),
        Expr::call_shared(head, args),
        list(vec![Expr::symbol("a"); 64]),
    ];
    nodes.extend(vec![Expr::symbol("a"); 61]);
    let (node, report) = check::<N>(&list(nodes));
    assert_eq!(report.probe.decision, Decision::Shared);
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
    let mut changed = a0.clone();
    let Expr::Atom(integer) = Expr::small_integer(7) else {
        unreachable!()
    };
    changed.make_mut()[0] = N::atom(integer);
    assert!(weak.is_alive() && !weak.ptr_eq(&changed));
    assert!(matches!(a0[0].view(), View::Atom(AtomRef::Symbol(_))));
    node.release_iterative();
    assert!(!weak.is_alive());
    drop(changed);
}
#[test]
fn selected_branch_preserves_head_args_alias_cow_and_weak() {
    aliases::<Compact>();
    aliases::<Safe>();
}
#[test]
fn call_only_prefix_is_bounded_without_reading_the_tail() {
    let depth = if cfg!(miri) { 300 } else { 10_000 };
    let mut chain = Expr::small_integer(1);
    for _ in 0..depth {
        chain = Expr::call(Expr::small_integer(0), vec![chain]);
    }
    let mut nodes = vec![Expr::small_integer(0); 63];
    nodes.insert(0, chain);
    let mut expr = Expr::call(Expr::small_integer(0), nodes);
    let r = probe(&expr);
    assert_eq!((r.decision, r.nodes, r.symbols), (Decision::Plain, 256, 0));
    // Avoid the pre-existing original representation's recursive Drop.
    let mut pending = vec![std::mem::replace(&mut expr, Expr::small_integer(0))];
    while let Some(node) = pending.pop() {
        if let Expr::Call { head, args } = node {
            pending.push(head.into_owned());
            pending.extend(args.into_vec());
        }
    }
}
