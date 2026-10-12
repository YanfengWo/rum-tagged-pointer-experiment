use super::*;
use rum_core::{BigInt, Expr, IntValue, Rational, Symbol};
fn strings<N: Tree>() {
    for len in [0, 1, 7, 8, 9, 31, 128] {
        for value in ["a".repeat(len), "界\0🙂".repeat(len)] {
            let n = N::borrowed_atom(&Atom::String(value.clone()));
            let clone = n.clone();
            let (Inspect::String(a), Inspect::String(b)) = (n.inspect(), clone.inspect()) else {
                panic!()
            };
            assert_eq!(a, value);
            assert_eq!(a.as_ptr(), b.as_ptr());
            assert_eq!(n.into_parts().err(), Some(Atom::String(value.clone())));
            assert_eq!(to_expr(&clone), Expr::string(value));
        }
    }
}
#[test]
fn typed_utf8_empty_boundaries_shared_extract() {
    strings::<Safe24>();
    strings::<Typed16>();
}
fn keys<N: Tree>() {
    let symbols = [
        Symbol::new("Global`", "x"),
        Symbol::new("Other`", "x"),
        Symbol::with_identity("Global`", "x", 1),
        Symbol::with_identity("Global`", "x", u64::MAX),
    ];
    for a in symbols {
        let n = N::atom(Atom::Symbol(a));
        let clone = n.clone();
        let Inspect::Symbol(key) = n.inspect() else {
            panic!()
        };
        assert_eq!(key, SymbolKey::from_symbol(a));
        assert_eq!(key.interned_id(), a.interned_id());
        assert_eq!(key.identity(), a.identity());
        for b in symbols {
            assert_eq!(key == SymbolKey::from_symbol(b), a == b)
        }
        drop(n);
        assert_eq!(to_expr(&clone), Expr::Atom(Atom::Symbol(a)));
    }
}
#[test]
fn cached_keys_preserve_context_and_all_identity_bits() {
    keys::<Safe24>();
    keys::<Typed16>();
}
fn borrowed<N: Tree>() {
    let atoms = [
        Atom::String("payload".into()),
        Atom::Integer(IntValue::Big(Box::new(BigInt::from(7)))),
        Atom::Integer(IntValue::Big(Box::new(BigInt::from(1) << 100))),
        Atom::Rational(Box::new(Rational::new(1.into(), 3.into()).unwrap())),
    ];
    for atom in atoms {
        let e = Expr::call(Expr::symbol("f"), vec![Expr::Atom(atom.clone()); 4]);
        let a = from_expr::<N>(&e);
        let b = from_expr_moved::<N>(&e);
        assert_eq!(to_expr(&a), e);
        assert_eq!(to_expr(&b), e);
        let n = N::borrowed_atom(&atom);
        let sibling = n.clone();
        match (n.inspect(), sibling.inspect()) {
            (Inspect::Big(a), Inspect::Big(b)) => assert!(std::ptr::eq(a, b)),
            (Inspect::Rational(a), Inspect::Rational(b)) => assert!(std::ptr::eq(a, b)),
            (Inspect::String(a), Inspect::String(b)) => assert_eq!(a.as_ptr(), b.as_ptr()),
            _ => panic!(),
        }
        assert_eq!(n.into_parts().err(), Some(atom.clone()));
        assert_eq!(sibling.into_parts().err(), Some(atom));
    }
}
#[test]
fn moved_and_borrowed_preserve_typed_leaves_and_small_big_variant() {
    borrowed::<Safe24>();
    borrowed::<Typed16>();
}
#[test]
fn typed_payloads_cross_threads_with_safe_readers() {
    let e = Expr::call(
        Expr::symbol("f"),
        vec![
            Expr::string("界\0🙂"),
            Expr::Atom(Atom::Integer(IntValue::Big(Box::new(BigInt::from(7))))),
        ],
    );
    let n = from_expr::<Typed16>(&e);
    std::thread::scope(|scope| {
        let copy = n.clone();
        let expected = e.clone();
        scope.spawn(move || {
            assert_eq!(to_expr(&copy), expected);
            drop(copy);
        });
        let View::Call { args, .. } = n.view() else {
            panic!()
        };
        scope.spawn(move || {
            assert_eq!(args.slice().len(), 2);
            drop(args.clone_handle());
        });
    });
    assert_eq!(to_expr(&n), e);
}
