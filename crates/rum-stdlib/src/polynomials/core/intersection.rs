use std::collections::HashMap;

use super::*;

pub(super) fn intersect_normalized_factors_into(
    lhs: &[Factor],
    rhs: &[Factor],
    intersection: &mut Vec<Factor>,
) {
    intersection.clear();
    intersection.reserve(lhs.len().min(rhs.len()));
    let mut lhs_index = 0;
    let mut rhs_index = 0;

    while let (Some(lhs_factor), Some(rhs_factor)) = (lhs.get(lhs_index), rhs.get(rhs_index)) {
        match compare_expr_canonical(&lhs_factor.base, &rhs_factor.base) {
            std::cmp::Ordering::Less => lhs_index += 1,
            std::cmp::Ordering::Greater => rhs_index += 1,
            std::cmp::Ordering::Equal => {
                let lhs_end = canonical_factor_group_end(lhs, lhs_index);
                let rhs_end = canonical_factor_group_end(rhs, rhs_index);
                let lhs_group = &lhs[lhs_index..lhs_end];
                let rhs_group = &rhs[rhs_index..rhs_end];

                if lhs_group.len() == 1 && rhs_group.len() == 1 {
                    if lhs_factor.base == rhs_factor.base {
                        intersection.push(Factor {
                            base: lhs_factor.base.clone(),
                            exponent: lhs_factor.exponent.min(rhs_factor.exponent),
                        });
                    }
                } else {
                    let mut rhs_exponents = HashMap::with_capacity(rhs_group.len());
                    for factor in rhs_group {
                        rhs_exponents.entry(&factor.base).or_insert(factor.exponent);
                    }
                    for factor in lhs_group {
                        if let Some(rhs_exponent) = rhs_exponents.get(&factor.base) {
                            intersection.push(Factor {
                                base: factor.base.clone(),
                                exponent: factor.exponent.min(*rhs_exponent),
                            });
                        }
                    }
                }

                lhs_index = lhs_end;
                rhs_index = rhs_end;
            }
        }
    }
}

fn canonical_factor_group_end(factors: &[Factor], start: usize) -> usize {
    let base = &factors[start].base;
    factors[start + 1..]
        .iter()
        .position(|factor| !compare_expr_canonical(base, &factor.base).is_eq())
        .map_or(factors.len(), |offset| start + 1 + offset)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intersection_matches_structural_bases_across_canonical_ties() {
        let integer = Expr::integer(1_i32.into());
        let rational = Expr::Atom(Atom::Rational(Box::new(
            Rational::new(1_i32.into(), 1_i32.into()).expect("non-zero denominator"),
        )));
        let lhs = vec![
            Factor {
                base: integer.clone(),
                exponent: 4,
            },
            Factor {
                base: rational.clone(),
                exponent: 2,
            },
        ];
        let rhs = vec![
            Factor {
                base: rational.clone(),
                exponent: 1,
            },
            Factor {
                base: integer.clone(),
                exponent: 3,
            },
        ];
        let mut intersection = Vec::new();

        intersect_normalized_factors_into(&lhs, &rhs, &mut intersection);

        assert_eq!(
            intersection,
            vec![
                Factor {
                    base: integer,
                    exponent: 3,
                },
                Factor {
                    base: rational,
                    exponent: 1,
                },
            ]
        );
    }
}
