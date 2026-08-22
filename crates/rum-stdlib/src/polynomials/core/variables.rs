use std::collections::HashSet;

use super::*;

const INDEXED_VARIABLE_THRESHOLD: usize = 32;

impl Polynomial {
    pub(in crate::polynomials) fn variables(&self) -> Vec<Expr> {
        sorted_unique_variables(&[self])
    }
}

fn sorted_unique_variables(polynomials: &[&Polynomial]) -> Vec<Expr> {
    let mut variables = unique_variable_refs(polynomials)
        .into_iter()
        .cloned()
        .collect::<Vec<_>>();
    variables.sort_by(compare_expr_canonical);
    variables
}

fn unique_variable_refs<'a>(polynomials: &[&'a Polynomial]) -> Vec<&'a Expr> {
    let mut variables = Vec::new();
    let mut index: Option<HashSet<&'a Expr>> = None;

    for polynomial in polynomials {
        for term in &polynomial.terms {
            for factor in &term.factors {
                if let Some(index) = &mut index {
                    if index.insert(&factor.base) {
                        variables.push(&factor.base);
                    }
                    continue;
                }

                if variables.len() == INDEXED_VARIABLE_THRESHOLD {
                    let mut indexed = variables.iter().copied().collect::<HashSet<_>>();
                    if indexed.insert(&factor.base) {
                        variables.push(&factor.base);
                    }
                    index = Some(indexed);
                    continue;
                }

                if variables.iter().any(|existing| *existing == &factor.base) {
                    continue;
                }

                variables.push(&factor.base);
            }
        }
    }

    variables
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wide_variable_collection_indexes_overlaps_and_sorts_once() {
        let shared = system_expr("shared");
        let lhs = polynomial_with_shared_variable("x", &shared, 40);
        let rhs = polynomial_with_shared_variable("y", &shared, 40);

        let variables = sorted_unique_variables(&[&lhs, &rhs]);

        assert_eq!(variables.len(), 81);
        assert_eq!(
            variables
                .iter()
                .filter(|variable| *variable == &shared)
                .count(),
            1
        );
        assert!(
            variables
                .windows(2)
                .all(|pair| compare_expr_canonical(&pair[0], &pair[1]).is_lt())
        );
    }

    fn polynomial_with_shared_variable(prefix: &str, shared: &Expr, count: usize) -> Polynomial {
        Polynomial::from_normalized_terms(
            (0..count)
                .map(|index| {
                    Term {
                        coefficient: Rational::one(),
                        factors: vec![
                            Factor {
                                base: shared.clone(),
                                exponent: 1,
                            },
                            Factor {
                                base: system_expr(&format!("{prefix}{index}")),
                                exponent: 1,
                            },
                        ],
                    }
                    .normalized()
                })
                .collect(),
        )
    }
}
