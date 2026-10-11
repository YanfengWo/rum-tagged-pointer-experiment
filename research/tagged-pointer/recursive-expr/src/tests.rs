use super::*;
use rum_core::{BigInt, Rational, SharedArgs, SharedExpr, Symbol};

fn variants<N: Tree>() {
    let mut input = vec![
        Expr::small_integer(i64::MIN),
        Expr::small_integer(i64::MAX),
        Expr::small_integer(owner::MIN),
        Expr::small_integer(owner::MAX),
        Expr::small_integer(owner::MIN - 1),
        Expr::small_integer(owner::MAX + 1),
        Expr::small_integer(-1),
        Expr::small_integer(0),
        Expr::small_integer(1),
        Expr::Atom(Atom::Integer(IntValue::Big(Box::new(BigInt::from(7))))),
        Expr::integer(BigInt::from(1) << 100),
        Expr::string("中文\0owned"),
        Expr::rational(Rational::new(BigInt::from(2), BigInt::from(3)).unwrap()),
        Expr::Atom(Atom::Symbol(Symbol::with_identity("Global`", "x", 123))),
        Expr::call(Expr::small_integer(4), vec![]),
    ];
    input.push(Expr::call(Expr::symbol("f"), input.clone()));
    for expr in input {
        let node: N = from_expr(&expr);
        assert_eq!(to_expr(&node), expr);
        assert_eq!(to_expr(&node.clone()), expr);
        if matches!(&expr, Expr::Atom(Atom::Integer(IntValue::Big(n))) if **n == BigInt::from(7)) {
            assert!(
                matches!(node.view(), View::Atom(AtomRef::Integer(IntRef::Big(n))) if *n == BigInt::from(7))
            );
        }
        node.release_iterative();
    }
}
#[test]
fn compact_all_variants() {
    variants::<Compact>();
}
#[test]
fn safe_all_variants() {
    variants::<Safe>();
}

fn sharing<N: Tree>() {
    // An immediate head also needs a shared identity, independent of its value.
    let head = SharedExpr::new(Expr::small_integer(3));
    let args = SharedArgs::new(vec![Expr::string("payload"), Expr::small_integer(9)]);
    let old_weak = args.downgrade();
    let repeated = Expr::call_shared(head, args);
    let original = Expr::call(Expr::symbol("g"), vec![repeated.clone(), repeated]);
    let node: N = from_expr(&original);
    let View::Call { args: outer, .. } = node.view() else {
        panic!()
    };
    let View::Call { head: h0, args: a0 } = outer[0].view() else {
        panic!()
    };
    let View::Call { head: h1, args: a1 } = outer[1].view() else {
        panic!()
    };
    assert!(h0.ptr_eq(h1));
    assert!(a0.ptr_eq(a1));
    let weak = a0.downgrade();
    assert!(weak.is_alive());
    let restored = to_expr(&node);
    let Expr::Call { args: outer, .. } = &restored else {
        panic!()
    };
    let Expr::Call { head: h0, args: a0 } = &outer[0] else {
        panic!()
    };
    let Expr::Call { head: h1, args: a1 } = &outer[1] else {
        panic!()
    };
    assert!(SharedExpr::ptr_eq(h0, h1));
    assert!(SharedArgs::ptr_eq(a0, a1));
    assert!(!old_weak.ptr_eq(a0));
    drop(original);
    assert!(!old_weak.is_alive());
    assert!(weak.is_alive());
    node.release_iterative();
    assert!(!weak.is_alive());
    assert!(restored == Expr::call(Expr::symbol("g"), vec![outer[0].clone(), outer[1].clone()]));
}
#[test]
fn compact_aliases_and_weak_namespace() {
    sharing::<Compact>();
}
#[test]
fn safe_aliases_and_weak_namespace() {
    sharing::<Safe>();
}

fn cow<N: Tree>() {
    let mut original = Args::new(vec![N::atom(Atom::String("original".into()))]);
    assert!(original.get_mut().is_some());
    let copy = original.clone();
    let weak = original.downgrade();
    assert!(original.get_mut().is_none());
    original
        .make_mut()
        .push(N::atom(Atom::Integer(IntValue::Small(2))));
    assert_eq!(original.len(), 2);
    assert_eq!(copy.len(), 1);
    assert!(!original.ptr_eq(&copy));
    assert!(weak.ptr_eq(&copy));
    assert!(weak.is_alive());
    drop(copy);
    assert!(!weak.is_alive());
    let weak_only = original.downgrade();
    assert!(original.get_mut().is_none());
    original.make_mut().clear();
    assert!(!weak_only.is_alive());
    assert!(!weak_only.ptr_eq(&original));
    assert!(original.get_mut().is_some());
    drop(original);
}
#[test]
fn compact_cow_and_weak_only_detach() {
    cow::<Compact>();
}
#[test]
fn safe_cow_and_weak_only_detach() {
    cow::<Safe>();
}

#[test]
fn consuming_parts_preserve_shared_sibling() {
    let node = Compact::call(
        Head::new(Compact::small(1)),
        Args::new(vec![Compact::small(7)]),
    );
    let sibling = node.clone();
    let (head, mut args) = node.into_parts().unwrap();
    let weak = args.downgrade();
    args.make_mut()[0] = Compact::small(9);
    let changed = Compact::call(head, args);
    assert_ne!(changed.to_expr(), sibling.to_expr());
    assert!(weak.is_alive());
    sibling.release_iterative();
    assert!(!weak.is_alive());
    changed.release_iterative();
    assert!(matches!(
        Compact::small(i64::MAX).into_parts(),
        Err(Atom::Integer(IntValue::Small(i64::MAX)))
    ));
}

#[test]
fn layout_and_threads() {
    assert_eq!(std::mem::size_of::<Compact>(), 8);
    assert_eq!(std::mem::size_of::<Safe>(), 16);
    assert_eq!(std::mem::size_of::<Head<Compact>>(), 8);
    assert_eq!(std::mem::size_of::<Args<Compact>>(), 8);
    let node = Arc::new(Compact::from_expr(&Expr::call(
        Expr::symbol("f"),
        vec![Expr::string("owned")],
    )));
    let sibling = node.clone();
    let restored = std::thread::spawn(move || sibling.to_expr())
        .join()
        .unwrap();
    assert_eq!(restored, node.to_expr());
}

fn drain_original(expr: Expr) {
    let mut pending = vec![expr];
    while let Some(expr) = pending.pop() {
        if let Expr::Call { head, args } = expr {
            pending.push(head.into_owned());
            pending.extend(args.into_vec());
        }
    }
}
#[test]
fn deep_conversion_and_iterative_release() {
    let depth = if cfg!(miri) { 24 } else { 10_000 };
    let mut expr = Expr::small_integer(7);
    let head = SharedExpr::new(Expr::small_integer(0));
    for _ in 0..depth {
        expr = Expr::call_shared(head.clone(), SharedArgs::new(vec![expr]));
    }
    let node = Compact::from_expr(&expr);
    let restored = node.to_expr();
    let mut cursor = &node;
    let mut count = 0;
    while let View::Call { args, .. } = cursor.view() {
        count += 1;
        cursor = &args[0];
    }
    assert_eq!(count, depth);
    let clone = node.clone();
    node.release_iterative();
    clone.release_iterative();
    drain_original(expr);
    drain_original(restored);
}

#[test]
fn ordinary_drop_moderate_depth_both_backends() {
    fn build<N: Tree>() -> N {
        let mut node = N::atom(Atom::Integer(IntValue::Small(0)));
        let head = Head::new(N::atom(Atom::Integer(IntValue::Small(1))));
        for _ in 0..if cfg!(miri) { 12 } else { 128 } {
            node = N::call(head.clone(), Args::new(vec![node]));
        }
        node
    }
    drop(build::<Compact>());
    drop(build::<Safe>());
}

#[test]
fn iterative_release_shared_args_and_weak() {
    let args = Args::new(vec![Compact::from_expr(&Expr::string("shared"))]);
    let weak = args.downgrade();
    let first = Compact::call(Head::new(Compact::small(1)), args.clone());
    let second = Compact::call(Head::new(Compact::small(2)), args);
    first.release_iterative();
    assert!(weak.is_alive());
    second.release_iterative();
    assert!(!weak.is_alive());
}

#[test]
fn private_owner_exactly_once_aligned_payload() {
    use std::sync::atomic::{AtomicUsize, Ordering};
    #[repr(align(128))]
    struct Count(Arc<AtomicUsize>);
    impl Drop for Count {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let drops = Arc::new(AtomicUsize::new(0));
    let owner = owner::Packed::heap(Count(drops.clone()));
    let clone = owner.clone();
    assert!(owner.take().is_none());
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    drop(clone.take());
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    let owner = owner::Packed::heap(Count(drops.clone()));
    drop(owner.clone());
    drop(owner);
    assert_eq!(drops.load(Ordering::SeqCst), 2);
}

#[test]
fn nested_dag_conversion_does_not_expand_logical_tree() {
    fn check<N: Tree>() {
        let depth = if cfg!(miri) { 8 } else { 80 };
        let head = SharedExpr::new(Expr::small_integer(0));
        let mut expr = Expr::small_integer(7);
        for _ in 0..depth {
            expr = Expr::call_shared(head.clone(), SharedArgs::new(vec![expr.clone(), expr]));
        }
        // Logical expansion has 2^80 leaves, but the shared graph has 80 levels.
        let node: N = from_expr(&expr);
        let restored = to_expr(&node);
        let mut cursor = &node;
        for level in 0..depth {
            let View::Call { args, .. } = cursor.view() else {
                panic!()
            };
            assert_eq!(args.len(), 2);
            if level + 1 < depth {
                let View::Call { args: first, .. } = args[0].view() else {
                    panic!()
                };
                let View::Call { args: second, .. } = args[1].view() else {
                    panic!()
                };
                assert!(first.ptr_eq(second));
            }
            cursor = &args[0];
        }
        assert!(matches!(
            cursor.view(),
            View::Atom(AtomRef::Integer(IntRef::Small(7)))
        ));
        node.release_iterative();
        // Dropping this moderate-depth DAG uses counts, without expanding it.
        drop(expr);
        drop(restored);
    }
    check::<Compact>();
    check::<Safe>();
}
