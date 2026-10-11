use compact_v1::CompactExpr as V1;
use compact_v1_hinted::CompactExpr as Hinted;
use rum_compact_expr::{CompactExpr as V2, MAX_INLINE, MIN_INLINE, SafeExpr};
use rum_core::{Atom, BigInt, Expr, IntValue, Rational, Symbol};

#[test]
fn all_representations_preserve_original_values() {
    let mut values = vec![
        Expr::integer(BigInt::from(1_u64) << 100),
        Expr::Atom(Atom::Integer(IntValue::Big(Box::new(BigInt::from(7))))),
        Expr::rational(Rational::new(BigInt::from(-7), BigInt::from(3)).unwrap()),
        Expr::string("shared unicode λ"),
        Expr::Atom(Atom::Symbol(Symbol::with_identity("Global`", "x", 17))),
        Expr::call(Expr::system_symbol("List"), vec![Expr::small_integer(3)]),
    ];
    values.extend(
        [
            i64::MIN,
            MIN_INLINE - 1,
            MIN_INLINE,
            -1,
            0,
            1,
            MAX_INLINE,
            MAX_INLINE + 1,
            i64::MAX,
        ]
        .map(Expr::small_integer),
    );
    for original in values {
        let old = V1::new(original.clone());
        let hinted = Hinted::new(original.clone());
        let new = V2::new(original.clone());
        let safe = SafeExpr::new(original.clone());
        assert_eq!(old.to_expr(), original);
        assert_eq!(hinted.to_expr(), original);
        assert_eq!(new.to_expr(), original);
        assert_eq!(safe.to_expr(), original);
        assert_eq!(old.clone().into_expr(), original);
        assert_eq!(hinted.clone().into_expr(), original);
        assert_eq!(new.clone().into_expr(), original);
        assert_eq!(safe.clone().into_expr(), original);
        assert_eq!(old.into_expr(), original);
        assert_eq!(hinted.into_expr(), original);
        assert_eq!(new.into_expr(), original);
        assert_eq!(safe.into_expr(), original);
    }
}

#[test]
fn projected_heap_lengths_agree() {
    let expr = Expr::string("same string");
    assert_eq!(
        rum_representation_study::inspect_v1_string_length(&V1::new(expr.clone())),
        Some(11)
    );
    assert_eq!(
        rum_representation_study::inspect_v1_hinted_string_length(&Hinted::new(expr.clone())),
        Some(11)
    );
    assert_eq!(
        rum_representation_study::inspect_v2_string_length(&V2::new(expr)),
        Some(11)
    );
    let expr = Expr::small_integer(7);
    assert_eq!(
        rum_representation_study::inspect_v1_string_length(&V1::new(expr.clone())),
        None
    );
    assert_eq!(
        rum_representation_study::inspect_v1_hinted_string_length(&Hinted::new(expr.clone())),
        None
    );
    assert_eq!(
        rum_representation_study::inspect_v2_string_length(&V2::new(expr)),
        None
    );
}
