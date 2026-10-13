use super::*;
use rum_core::Symbol;
use std::cmp::Ordering;
fn symbols() -> Vec<Symbol> {
    ["Global`", "System`", "Other`", "Other`Nested`"]
        .into_iter()
        .flat_map(|ctx| {
            ["x", "a", "", "界", "aa", "a`"]
                .into_iter()
                .flat_map(move |name| {
                    [
                        Symbol::new(ctx, name),
                        Symbol::with_identity(ctx, name, 1),
                        Symbol::with_identity(ctx, name, u64::MAX),
                    ]
                })
        })
        .collect()
}
#[test]
fn exact_qualified_order_full_identity_and_stable_ties() {
    let input = symbols();
    for a in &input {
        for b in &input {
            let sa = Safe16::new(*a);
            let sb = Safe16::new(*b);
            let ta = Tagged16::new(*a);
            let tb = Tagged16::new(*b);
            assert_eq!(sa.compare(&sb), a.cmp_qualified_name(b));
            assert_eq!(ta.compare(&tb), a.cmp_qualified_name(b));
            assert_eq!(sa.symbol(), *a);
            assert_eq!(ta.symbol(), *a);
        }
    }
    let e = input
        .iter()
        .rev()
        .copied()
        .map(|s| Expr::Atom(Atom::Symbol(s)))
        .collect::<Vec<_>>();
    let expected = sort::<Symbol>(&e);
    assert_eq!(sort::<Safe16>(&e), expected);
    assert_eq!(sort::<Tagged16>(&e), expected);
    assert_eq!(try_sort_symbols(&e), Some(expected));
    assert_eq!(
        Symbol::new("Global`", "x").cmp_qualified_name(&Symbol::new("System`", "x")),
        Ordering::Equal
    );
}
#[test]
fn threshold_mixed_late_empty_and_thread_transfer() {
    let s = Expr::Atom(Atom::Symbol(Symbol::from_name("x")));
    assert_eq!(try_sort_symbols(&[]), None);
    assert_eq!(try_sort_symbols(&vec![s.clone(); 63]), None);
    assert_eq!(
        try_sort_symbols(&vec![s.clone(); 64]),
        Some(vec![s.clone(); 64])
    );
    let mut late = vec![s; 64];
    late.push(Expr::small_integer(0));
    assert_eq!(try_sort_symbols(&late), None);
    let value = Tagged16::new(Symbol::with_identity("Global`", "x", u64::MAX));
    let expected = value.symbol();
    assert_eq!(
        std::thread::spawn(move || value.symbol()).join().unwrap(),
        expected
    );
}
