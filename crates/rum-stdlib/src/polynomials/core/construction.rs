use std::collections::{HashMap, hash_map::Entry};

use super::*;

const INDEXED_TERM_MERGE_THRESHOLD: usize = 32;

impl Polynomial {
    /// Finishes a batch of individually normalized terms by merging equal
    /// monomials and establishing canonical polynomial order once.
    pub(in crate::polynomials) fn from_normalized_terms(terms: Vec<Term>) -> Self {
        let terms = if terms.len() >= INDEXED_TERM_MERGE_THRESHOLD {
            merge_terms_indexed(terms)
        } else {
            merge_terms_linear(terms)
        };
        let mut polynomial = Self { terms };

        if polynomial.terms.len() > 1 {
            let basis = VariableBasis::from_polynomials(&[&polynomial]);
            polynomial
                .terms
                .sort_by(|lhs, rhs| basis.compare_terms(lhs, rhs));
        }

        polynomial
    }

    /// Constructs a polynomial after applying the same monomial transform to
    /// every selected term of an already-canonical polynomial.
    ///
    /// Uniform coefficient scaling, multiplication or division by one
    /// monomial, and filtering by one exact monomial power preserve both term
    /// uniqueness and graded-lexicographic order. Those paths can therefore
    /// move the whole batch into place without rebuilding a variable basis and
    /// sorting after every term.
    pub(super) fn from_order_preserving_terms(terms: Vec<Term>) -> Self {
        debug_assert!(terms.iter().all(|term| !term.coefficient.is_zero()));
        Self { terms }
    }
}

fn merge_terms_linear(terms: Vec<Term>) -> Vec<Term> {
    let mut merged: Vec<Term> = Vec::with_capacity(terms.len());

    for term in terms {
        if term.coefficient.is_zero() {
            continue;
        }

        if let Some(existing) = merged
            .iter_mut()
            .find(|existing| existing.factors == term.factors)
        {
            existing.coefficient = existing.coefficient.add(&term.coefficient);
        } else {
            merged.push(term);
        }
    }

    merged.retain(|term| !term.coefficient.is_zero());
    merged
}

fn merge_terms_indexed(terms: Vec<Term>) -> Vec<Term> {
    let mut coefficients: HashMap<Vec<Factor>, Rational> = HashMap::with_capacity(terms.len());

    for term in terms {
        if term.coefficient.is_zero() {
            continue;
        }

        match coefficients.entry(term.factors) {
            Entry::Occupied(mut entry) => {
                let coefficient = entry.get().add(&term.coefficient);
                *entry.get_mut() = coefficient;
            }
            Entry::Vacant(entry) => {
                entry.insert(term.coefficient);
            }
        }
    }

    coefficients
        .into_iter()
        .filter_map(|(factors, coefficient)| {
            (!coefficient.is_zero()).then_some(Term {
                coefficient,
                factors,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indexed_batch_merge_cancels_duplicates_and_sorts_once() {
        let mut terms = (0..40)
            .map(|index| Term::indeterminate(system_expr(&format!("x{index}"))))
            .collect::<Vec<_>>();
        terms.push(Term {
            coefficient: rational_from_i64(-1),
            factors: vec![Factor {
                base: system_expr("x3"),
                exponent: 1,
            }],
        });

        let polynomial = Polynomial::from_normalized_terms(terms);

        assert_eq!(polynomial.terms.len(), 39);
        assert!(
            polynomial
                .terms
                .iter()
                .all(|term| term.factors[0].base != system_expr("x3"))
        );
        let basis = VariableBasis::from_polynomials(&[&polynomial]);
        assert!(
            polynomial
                .terms
                .windows(2)
                .all(|pair| basis.compare_terms(&pair[0], &pair[1]).is_lt())
        );
    }
}
