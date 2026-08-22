use super::*;

#[test]
fn ordered_factor_merge_matches_concat_and_normalize() {
    let lhs = term(&[("a", 2), ("c", 1), ("e", 3)]);
    let rhs = term(&[("b", 4), ("c", 5), ("d", 1)]);
    let mut legacy_factors = lhs.factors.clone();
    legacy_factors.extend(rhs.factors.clone());

    assert_eq!(
        lhs.multiply(&rhs).factors,
        normalize_factors(legacy_factors)
    );
}

#[test]
fn ordered_factor_merge_preserves_canonical_ties() {
    let integer = Expr::integer(1_i32.into());
    let rational = Expr::Atom(Atom::Rational(Box::new(
        Rational::new(1_i32.into(), 1_i32.into()).expect("non-zero denominator"),
    )));
    assert_eq!(
        compare_expr_canonical(&integer, &rational),
        std::cmp::Ordering::Equal
    );
    assert_ne!(integer, rational);

    let lhs = Term {
        coefficient: Rational::one(),
        factors: vec![
            Factor {
                base: integer.clone(),
                exponent: 1,
            },
            Factor {
                base: rational.clone(),
                exponent: 2,
            },
        ],
    };
    let rhs = Term {
        coefficient: Rational::one(),
        factors: vec![Factor {
            base: integer,
            exponent: 3,
        }],
    };
    let mut legacy_factors = lhs.factors.clone();
    legacy_factors.extend(rhs.factors.clone());

    assert_eq!(
        lhs.multiply(&rhs).factors,
        normalize_factors(legacy_factors)
    );
}

#[test]
fn batch_term_product_matches_repeated_binary_product() {
    let terms = [
        term(&[("a", 1), ("c", 2)]),
        term(&[("b", 1), ("c", 3)]),
        term(&[("a", 4), ("d", 1)]),
    ];
    let repeated = terms
        .iter()
        .fold(Term::constant(Rational::one()), |product, term| {
            product.multiply(term)
        });

    assert_eq!(Term::multiply_many(terms.iter()), repeated);
}

fn term(factors: &[(&str, usize)]) -> Term {
    Term {
        coefficient: Rational::one(),
        factors: factors
            .iter()
            .map(|(base, exponent)| Factor {
                base: system_expr(base),
                exponent: *exponent,
            })
            .collect::<Vec<_>>(),
    }
    .normalized()
}
