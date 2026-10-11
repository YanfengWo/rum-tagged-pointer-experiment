use super::*;
use rum_core::{BigInt, Rational, SharedArgs, Symbol};

#[test]
fn borrowed_integer_variants_include_heap_small_and_manual_big() {
    for number in [i64::MIN, MIN_INLINE, -1, 0, MAX_INLINE, i64::MAX] {
        let value = CompactExpr::new(Expr::small_integer(number));
        let safe = SafeExpr::new(Expr::small_integer(number));
        for view in [value.borrow(), safe.borrow()] {
            let ExprRef::Atom(AtomRef::Integer(integer)) = view else {
                panic!()
            };
            assert_eq!(integer.as_i64(), Some(number));
        }
    }
    let expr = Expr::Atom(Atom::Integer(IntValue::Big(Box::new(BigInt::from(7)))));
    let value = CompactExpr::new(expr);
    let ExprRef::Atom(AtomRef::Integer(integer @ IntRef::Big(number))) = value.borrow() else {
        panic!()
    };
    assert_eq!(number, &BigInt::from(7));
    assert_eq!(integer.as_i64(), None);
}

#[test]
fn borrowed_atoms_keep_content_and_generated_identity() {
    let value = CompactExpr::new(Expr::string("unicode λ"));
    let ExprRef::Atom(AtomRef::String(text)) = value.borrow() else {
        panic!()
    };
    assert_eq!(text, "unicode λ");
    let ExprView::Heap(Expr::Atom(Atom::String(original))) = value.view() else {
        panic!()
    };
    assert_eq!(text.as_ptr(), original.as_ptr());
    let symbol = Symbol::with_identity("Global`", "x", 17);
    let value = CompactExpr::new(Expr::Atom(Atom::Symbol(symbol)));
    let ExprRef::Atom(AtomRef::Symbol(actual)) = value.borrow() else {
        panic!()
    };
    assert_eq!(actual, symbol);
    let rational = Rational::new(BigInt::from(-7), BigInt::from(3)).unwrap();
    let value = CompactExpr::new(Expr::rational(rational.clone()));
    let ExprRef::Atom(AtomRef::Rational(actual)) = value.borrow() else {
        panic!()
    };
    assert_eq!(actual, &rational);
}

#[test]
fn borrowed_calls_retain_handle_identity_and_weak_liveness() {
    let args = SharedArgs::new(vec![Expr::small_integer(7)]);
    let weak = args.downgrade();
    let value = CompactExpr::new(Expr::call_with_shared_args(
        Expr::system_symbol("List"),
        args.clone(),
    ));
    let ExprRef::Call {
        head,
        args: borrowed,
    } = value.borrow()
    else {
        panic!()
    };
    assert_eq!(head.as_ref(), &Expr::system_symbol("List"));
    assert!(SharedArgs::ptr_eq(borrowed, &args));
    assert_eq!(borrowed.len(), 1);
    drop(args);
    assert!(weak.is_alive());
    drop(value);
    assert!(!weak.is_alive());
}
