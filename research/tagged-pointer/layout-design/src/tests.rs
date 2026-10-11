use super::*;
use rum_core::{BigInt, Expr, IntValue, Rational, SharedArgs, SharedExpr, Symbol};
fn check<N: Tree>(e: &Expr) -> N {
    let n = from_expr::<N>(e);
    assert_eq!(to_expr(&n), *e);
    n
}
macro_rules! all {
    ($f:ident) => {{
        $f::<Safe24>();
        $f::<Tagged16>();
        $f::<Tagged8>();
    }};
}
#[test]
fn expected_sizes_and_thread_bounds() {
    assert_eq!(std::mem::size_of::<Safe24>(), 24);
    assert_eq!(std::mem::size_of::<Tagged16>(), 16);
    assert_eq!(std::mem::size_of::<Tagged8>(), 8);
    fn bounds<N: Tree + Send + Sync + 'static>() {
        fn yes<T: Send + Sync>() {}
        yes::<Head<N>>();
        yes::<Args<N>>();
        yes::<HeadRef<'static, N>>();
        yes::<ArgsRef<'static, N>>();
    }
    bounds::<Safe24>();
    bounds::<Tagged16>();
    bounds::<Tagged8>();
}
fn integers<N: Tree>() {
    for x in [
        i64::MIN,
        -(1 << 60) - 1,
        -(1 << 60),
        -1,
        0,
        1,
        (1 << 60) - 1,
        1 << 60,
        i64::MAX,
    ] {
        let n = check::<N>(&Expr::small_integer(x));
        assert!(matches!(n.view(),View::Atom(AtomRef::Integer(IntRef::Small(v))) if v==x));
        assert_eq!(
            n.clone().into_parts().err(),
            Some(Atom::Integer(IntValue::Small(x)))
        );
    }
}
#[test]
fn full_i64_and_immediate_boundaries_preserve_variants() {
    all!(integers)
}
fn symbols<N: Tree>() {
    let symbols = [
        Symbol::new("Global`", "x"),
        Symbol::new("Other`", "x"),
        Symbol::with_identity("Global`", "x", 1),
        Symbol::with_identity("Global`", "x", u64::MAX),
    ];
    for s in symbols {
        let n = check::<N>(&Expr::Atom(Atom::Symbol(s)));
        assert!(matches!(n.view(),View::Atom(AtomRef::Symbol(v)) if v==s));
        assert_eq!(n.clone().into_parts().err(), Some(Atom::Symbol(s)));
        drop(n);
    }
}
#[test]
fn context_plain_and_full_generated_identity_are_distinct() {
    all!(symbols)
}
fn owned<N: Tree>() {
    let atoms = [
        Atom::Integer(IntValue::Big(Box::new(BigInt::from(7)))),
        Atom::Integer(IntValue::Big(Box::new(BigInt::from(1) << 100))),
        Atom::String("owned\0value".into()),
        Atom::Rational(Box::new(Rational::new(1.into(), 3.into()).unwrap())),
    ];
    for atom in atoms {
        let n = N::atom(atom.clone());
        let sibling = n.clone();
        assert_eq!(n.into_parts().err(), Some(atom.clone()));
        assert_eq!(to_expr(&sibling), Expr::Atom(atom));
    }
}
#[test]
fn shared_owned_extraction_preserves_big_string_and_rational() {
    all!(owned)
}
fn aliases<N: Tree>() {
    let h = SharedExpr::new(Expr::symbol("f"));
    let a = SharedArgs::new(vec![Expr::symbol("a")]);
    let e = Expr::call(
        Expr::small_integer(0),
        vec![
            Expr::call_shared(h.clone(), a.clone()),
            Expr::call_shared(h, a),
            Expr::call(Expr::symbol("f"), vec![Expr::symbol("a")]),
        ],
    );
    let n = check::<N>(&e);
    let View::Call { args, .. } = n.view() else {
        panic!()
    };
    let args = args.slice();
    let View::Call { head: h0, args: a0 } = args[0].view() else {
        panic!()
    };
    let View::Call { head: h1, args: a1 } = args[1].view() else {
        panic!()
    };
    let View::Call { head: h2, args: a2 } = args[2].view() else {
        panic!()
    };
    assert!(h0.ptr_eq(&h1) && a0.ptr_eq(&a1));
    assert!(!h0.ptr_eq(&h2) && !a0.ptr_eq(&a2));
    let original = a0.clone_handle();
    let weak = original.downgrade();
    let mut changed = original.clone();
    changed.make_mut()[0] = N::atom(Atom::String("changed".into()));
    assert!(weak.is_alive() && !weak.ptr_eq(&changed));
    assert!(matches!(original[0].view(), View::Atom(AtomRef::Symbol(_))));
    drop(original);
    n.release_iterative();
    assert!(!weak.is_alive());
    let projected = to_expr(&changed[0]);
    assert_eq!(projected, Expr::string("changed"));
}
#[test]
fn aliases_cow_weak_and_changed_value_are_preserved() {
    all!(aliases)
}
fn weak_only<N: Tree>() {
    let mut args = Args::new(vec![N::atom(Atom::Symbol(Symbol::from_name("a")))]);
    let weak = args.downgrade();
    assert!(args.get_mut().is_none());
    args.make_mut()
        .push(N::atom(Atom::Integer(IntValue::Small(7))));
    assert!(!weak.is_alive() && !weak.ptr_eq(&args));
    assert_eq!(args.len(), 2);
}
#[test]
fn weak_only_cow_invalidates_previous_allocation() {
    all!(weak_only)
}
fn counts<N: Tree>() {
    let head = Head::new(N::atom(Atom::Symbol(Symbol::from_name("f"))));
    let args = Args::new(vec![N::atom(Atom::Integer(IntValue::Small(7)))]);
    let n = N::call(head.clone(), args.clone());
    let cloned = n.clone();
    let hc = Arc::strong_count(&head.0);
    let ac = Arc::strong_count(&args.0);
    for _ in 0..4 {
        let View::Call { head: h, args: a } = n.view() else {
            panic!()
        };
        assert!(matches!(h.node().view(), View::Atom(AtomRef::Symbol(_))));
        let _ = a.slice();
    }
    assert_eq!(Arc::strong_count(&head.0), hc);
    assert_eq!(Arc::strong_count(&args.0), ac);
    let (h, a) = n.into_parts().ok().unwrap();
    assert!(h.ptr_eq(&head) && a.ptr_eq(&args));
    drop(cloned);
    drop(h);
    drop(a);
    assert_eq!(Arc::strong_count(&head.0), 1);
    assert_eq!(Arc::strong_count(&args.0), 1);
}
#[test]
fn viewing_does_not_acquire_counts_and_consumption_transfers_handles() {
    all!(counts)
}
fn deep<N: Tree>() {
    let depth = if cfg!(miri) { 24 } else { 10_000 };
    let mut e = Expr::small_integer(7);
    for _ in 0..depth {
        e = Expr::call(Expr::symbol("f"), vec![e])
    }
    let n = from_expr::<N>(&e);
    n.clone().release_iterative();
    n.release_iterative();
    while let Expr::Call { args, .. } = e {
        e = args.into_vec().pop().unwrap()
    }
}
#[test]
fn deep_explicit_release_keeps_last_owner_rule() {
    all!(deep)
}
fn dag<N: Tree>() {
    let depth = if cfg!(miri) { 8 } else { 80 };
    let h = SharedExpr::new(Expr::symbol("f"));
    let mut e = Expr::small_integer(7);
    for _ in 0..depth {
        e = Expr::call_shared(h.clone(), SharedArgs::new(vec![e.clone(), e]))
    }
    let n = from_expr::<N>(&e);
    let mut current = &n;
    for _ in 0..depth {
        let View::Call { args, .. } = current.view() else {
            panic!()
        };
        current = &args.slice()[0]
    }
    assert!(matches!(
        current.view(),
        View::Atom(AtomRef::Integer(IntRef::Small(7)))
    ));
    n.release_iterative();
    drop(e); // Depth <=80; normal Arc reclamation does not expand shared paths.
}
#[test]
fn shared_dag_conversion_and_release_do_not_unfold() {
    all!(dag)
}
#[test]
fn original_copy_preserves_values_and_alias_classes_in_one_pass() {
    let h = SharedExpr::new(Expr::symbol("f"));
    let a = SharedArgs::new(vec![Expr::string("x")]);
    let e = Expr::call(
        Expr::symbol("g"),
        vec![
            Expr::call_shared(h.clone(), a.clone()),
            Expr::call_shared(h, a),
        ],
    );
    let n = original_copy(&e);
    assert_eq!(e, n);
    let Expr::Call { args, .. } = n else { panic!() };
    let (Expr::Call { head: h0, args: a0 }, Expr::Call { head: h1, args: a1 }) =
        (&args[0], &args[1])
    else {
        panic!()
    };
    assert!(SharedExpr::ptr_eq(h0, h1) && SharedArgs::ptr_eq(a0, a1));
}
