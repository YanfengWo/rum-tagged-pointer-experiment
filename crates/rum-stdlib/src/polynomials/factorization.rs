use super::{
    core::{Polynomial, Term},
    factor_collections::stable_factor_counts,
    *,
};
pub(super) struct PolynomialFactorization {
    pub(super) unit: Rational,
    pub(super) factors: Vec<Polynomial>,
}

impl PolynomialFactorization {
    pub(super) fn from_polynomial(polynomial: Polynomial) -> Self {
        if polynomial.is_zero() {
            return Self {
                unit: Rational::zero(),
                factors: Vec::new(),
            };
        }

        let Some((unit, monomial, primitive)) = polynomial.extract_factor_parts() else {
            return Self {
                unit: Rational::one(),
                factors: vec![polynomial],
            };
        };

        let mut factorization = Self {
            unit,
            factors: monomial.to_polynomial_factors(),
        };

        if primitive.is_one() {
            return factorization;
        }

        if let Some((factor, quotient)) = primitive.find_linear_factorization() {
            factorization.extend(Self::from_polynomial(factor));
            factorization.extend(Self::from_polynomial(quotient));
        } else {
            factorization.factors.push(primitive);
        }

        factorization
    }

    pub(super) fn extend(&mut self, rhs: Self) {
        self.unit = self.unit.mul(&rhs.unit);
        self.factors.extend(rhs.factors);
    }

    pub(super) fn to_polynomial(&self) -> Polynomial {
        if self.unit.is_zero() {
            return Polynomial::zero();
        }

        if self.factors.iter().all(|factor| factor.terms.len() == 1) {
            let mut product =
                Term::multiply_many(self.factors.iter().map(|factor| &factor.terms[0]));
            product.coefficient = product.coefficient.mul(&self.unit);
            return Polynomial::from_normalized_terms(vec![product]);
        }

        let mut polynomial = Polynomial::constant(self.unit.clone());

        for factor in &self.factors {
            polynomial = polynomial.multiply(factor);
        }

        polynomial
    }

    pub(super) fn to_expr(&self, session: &Session) -> Result<Expr, EvalError> {
        if self.unit.is_zero() {
            return Ok(Expr::integer(0_i32.into()));
        }

        let mut items = Vec::new();
        if !self.unit.is_one() || self.factors.is_empty() {
            items.push(Expr::rational(self.unit.clone()));
        }

        for (factor, count) in stable_factor_counts(&self.factors) {
            let factor_expr = factor.to_expr(session)?;
            if count == 1 {
                items.push(factor_expr);
            } else {
                items.push(Expr::call(
                    system_expr("Power"),
                    vec![factor_expr, Expr::integer(BigInt::from(count as u64))],
                ));
            }
        }

        session.combine_multiplicative_args(&system_expr("Times"), &items)
    }
}
