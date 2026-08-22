use std::cmp::Ordering;

use super::*;

impl Term {
    /// Compares graded-lexicographic term order directly on the normalized,
    /// sparse factor vectors.
    pub(in crate::polynomials) fn compare_order(&self, rhs: &Self) -> Ordering {
        self.total_degree()
            .cmp(&rhs.total_degree())
            .then_with(|| compare_factor_exponents(&self.factors, &rhs.factors))
    }
}

fn compare_factor_exponents(lhs: &[Factor], rhs: &[Factor]) -> Ordering {
    compare_factor_exponents_by(lhs, rhs, compare_expr_canonical)
}

fn compare_factor_exponents_by(
    lhs: &[Factor],
    rhs: &[Factor],
    mut compare_bases: impl FnMut(&Expr, &Expr) -> Ordering,
) -> Ordering {
    let mut lhs_index = 0;
    let mut rhs_index = 0;

    loop {
        match (lhs.get(lhs_index), rhs.get(rhs_index)) {
            (Some(lhs_factor), Some(rhs_factor)) => {
                match compare_bases(&lhs_factor.base, &rhs_factor.base) {
                    Ordering::Equal => match lhs_factor.exponent.cmp(&rhs_factor.exponent) {
                        Ordering::Equal => {
                            lhs_index += 1;
                            rhs_index += 1;
                        }
                        ordering => return ordering,
                    },
                    // The first base present on only one side is also the first
                    // non-equal coordinate in the dense exponent vector.
                    Ordering::Less => return Ordering::Greater,
                    Ordering::Greater => return Ordering::Less,
                }
            }
            (Some(_), None) => return Ordering::Greater,
            (None, Some(_)) => return Ordering::Less,
            (None, None) => return Ordering::Equal,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparse_order_matches_dense_basis_order_exhaustively() {
        let mut variables = (0..4)
            .map(|index| system_expr(&format!("x{index}")))
            .collect::<Vec<_>>();
        variables.sort_by(compare_expr_canonical);
        let terms = exponent_vectors(variables.len())
            .into_iter()
            .map(|exponents| term_from_exponents(&variables, &exponents))
            .collect::<Vec<_>>();

        for lhs in &terms {
            for rhs in &terms {
                assert_eq!(
                    lhs.compare_order(rhs),
                    compare_dense(lhs, rhs, &variables).0,
                    "lhs={lhs:?}, rhs={rhs:?}"
                );
            }
        }
    }

    #[test]
    fn sparse_order_avoids_wide_zero_coordinate_probes() {
        let mut variables = (0..320)
            .map(|index| system_expr(&format!("x{index}")))
            .collect::<Vec<_>>();
        variables.sort_by(compare_expr_canonical);
        let lhs = Term::indeterminate(variables[318].clone());
        let rhs = Term::indeterminate(variables[319].clone());
        let (_, dense_probes) = compare_dense(&lhs, &rhs, &variables);
        let mut sparse_base_comparisons = 0;
        let sparse = compare_factor_exponents_by(&lhs.factors, &rhs.factors, |lhs, rhs| {
            sparse_base_comparisons += 1;
            compare_expr_canonical(lhs, rhs)
        });

        assert_eq!(dense_probes, 638);
        assert_eq!(sparse_base_comparisons, 1);
        assert_eq!(sparse, compare_dense(&lhs, &rhs, &variables).0);
    }

    fn exponent_vectors(variable_count: usize) -> Vec<Vec<usize>> {
        let count = 3_usize.pow(variable_count as u32);
        (0..count)
            .map(|mut encoded| {
                let mut exponents = Vec::with_capacity(variable_count);
                for _ in 0..variable_count {
                    exponents.push(encoded % 3);
                    encoded /= 3;
                }
                exponents
            })
            .collect()
    }

    fn term_from_exponents(variables: &[Expr], exponents: &[usize]) -> Term {
        Term {
            coefficient: Rational::one(),
            factors: variables
                .iter()
                .zip(exponents)
                .filter_map(|(base, exponent)| {
                    (*exponent > 0).then(|| Factor {
                        base: base.clone(),
                        exponent: *exponent,
                    })
                })
                .collect(),
        }
    }

    fn compare_dense(lhs: &Term, rhs: &Term, variables: &[Expr]) -> (Ordering, usize) {
        let degree_order = lhs.total_degree().cmp(&rhs.total_degree());
        if degree_order != Ordering::Equal {
            return (degree_order, 0);
        }

        let mut probes = 0;
        for variable in variables {
            probes += 2;
            match lhs
                .exponent_for_factor(variable)
                .cmp(&rhs.exponent_for_factor(variable))
            {
                Ordering::Equal => {}
                ordering => return (ordering, probes),
            }
        }

        (Ordering::Equal, probes)
    }
}
