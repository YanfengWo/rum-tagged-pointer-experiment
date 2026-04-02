use super::{
    candidates::{bigint_abs, bigint_gcd, bigint_lcm, linear_factor_candidates, rational_from_i64},
    expr_contains_pattern_language, parse_indeterminate_expr, *,
};
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Polynomial {
    pub(super) terms: Vec<Term>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Term {
    pub(super) coefficient: Rational,
    pub(super) factors: Vec<Factor>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Factor {
    pub(super) base: Expr,
    pub(super) exponent: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct Monomial {
    pub(super) factors: Vec<Factor>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct VariableBasis {
    pub(super) variables: Vec<Expr>,
}

impl Polynomial {
    pub(super) fn zero() -> Self {
        Self { terms: Vec::new() }
    }

    pub(super) fn one() -> Self {
        Self::constant(Rational::one())
    }

    pub(super) fn constant(value: Rational) -> Self {
        if value.is_zero() {
            Self::zero()
        } else {
            Self {
                terms: vec![Term::constant(value)],
            }
        }
    }

    pub(super) fn indeterminate(base: Expr) -> Self {
        Self {
            terms: vec![Term::indeterminate(base)],
        }
    }

    pub(super) fn from_expr(expr: &Expr) -> Option<Self> {
        if let Some(value) = expr_as_exact_rational(expr) {
            return Some(Self::constant(value));
        }

        match expr {
            Expr::Call { head, args } if is_system_symbol_name(head, "Plus") => {
                let mut polynomial = Self::zero();

                for arg in args {
                    polynomial = polynomial.add(Self::from_expr(arg)?);
                }

                Some(polynomial)
            }
            Expr::Call { head, args } if is_system_symbol_name(head, "Times") => {
                let mut polynomial = Self::one();

                for arg in args {
                    polynomial = polynomial.multiply(&Self::from_expr(arg)?);
                }

                Some(polynomial)
            }
            Expr::Call { head, args }
                if is_system_symbol_name(head, "Power") && args.len() == 2 =>
            {
                if let Some(exponent) = expr_as_non_negative_usize(&args[1]) {
                    return Some(Self::from_expr(&args[0])?.pow(exponent));
                }

                (!expr_contains_pattern_language(expr)).then(|| Self::indeterminate(expr.clone()))
            }
            _ => (!expr_contains_pattern_language(expr)).then(|| Self::indeterminate(expr.clone())),
        }
    }

    pub(super) fn add(mut self, rhs: Self) -> Self {
        for term in rhs.terms {
            self.push_term(term);
        }

        self
    }

    pub(super) fn multiply(&self, rhs: &Self) -> Self {
        if self.terms.is_empty() || rhs.terms.is_empty() {
            return Self::zero();
        }

        let mut product = Self::zero();

        for lhs_term in &self.terms {
            for rhs_term in &rhs.terms {
                product.push_term(lhs_term.multiply(rhs_term));
            }
        }

        product
    }

    pub(super) fn pow(&self, exponent: usize) -> Self {
        if exponent == 0 {
            return Self::one();
        }

        let mut result = Self::one();
        let mut factor = self.clone();
        let mut remaining = exponent;

        while remaining > 0 {
            if remaining % 2 == 1 {
                result = result.multiply(&factor);
            }

            remaining /= 2;

            if remaining > 0 {
                factor = factor.multiply(&factor);
            }
        }

        result
    }

    pub(super) fn scale(&self, scalar: &Rational) -> Self {
        if scalar.is_zero() || self.is_zero() {
            return Self::zero();
        }

        let mut scaled = Self::zero();

        for term in &self.terms {
            scaled.push_term(Term {
                coefficient: term.coefficient.mul(scalar),
                factors: term.factors.clone(),
            });
        }

        scaled
    }

    pub(super) fn coefficient_for_monomial_power(&self, target: &Monomial, power: usize) -> Self {
        let mut coefficient = Self::zero();

        for term in &self.terms {
            if let Some(remainder) = term.coefficient_after_exact_monomial_power(target, power) {
                coefficient.push_term(remainder);
            }
        }

        coefficient
    }

    pub(super) fn max_exact_power_for_monomial(&self, target: &Monomial) -> usize {
        self.terms
            .iter()
            .map(|term| target.exact_multiplicity_in_term(term))
            .max()
            .unwrap_or(0)
    }

    pub(super) fn coefficient_list_expr(
        &self,
        variables: &[Monomial],
        session: &Session,
    ) -> Result<Expr, EvalError> {
        let max_powers = variables
            .iter()
            .map(|variable| self.max_exact_power_for_monomial(variable))
            .collect::<Vec<_>>();

        self.coefficient_grid_expr(variables, &max_powers, session)
    }

    pub(super) fn coefficient_grid_expr(
        &self,
        variables: &[Monomial],
        max_powers: &[usize],
        session: &Session,
    ) -> Result<Expr, EvalError> {
        let Some((first, remaining_variables)) = variables.split_first() else {
            return self.to_expr(session);
        };

        let max_power = max_powers[0];
        let mut items = Vec::with_capacity(max_power + 1);

        for power in 0..=max_power {
            let coefficient = self.coefficient_for_monomial_power(first, power);
            let item = if remaining_variables.is_empty() {
                coefficient.to_expr(session)?
            } else {
                coefficient.coefficient_grid_expr(remaining_variables, &max_powers[1..], session)?
            };
            items.push(item);
        }

        Ok(Expr::call(system_expr("List"), items))
    }

    pub(super) fn subtract(&self, rhs: &Self) -> Self {
        self.clone().add(rhs.scale(&rational_from_i64(-1)))
    }

    pub(super) fn multiply_by_term(&self, term: &Term) -> Self {
        let mut product = Self::zero();

        for existing in &self.terms {
            product.push_term(existing.multiply(term));
        }

        product
    }

    pub(super) fn divide_by_scalar(&self, scalar: &Rational) -> Option<Self> {
        Some(self.scale(&scalar.reciprocal()?))
    }

    pub(super) fn common_monomial_factor(&self) -> Monomial {
        if self.is_zero() {
            return Monomial::one();
        }

        let factors = self
            .variables()
            .into_iter()
            .filter_map(|base| {
                let exponent = self
                    .terms
                    .iter()
                    .map(|term| term.exponent_for_factor(&base))
                    .min()
                    .unwrap_or(0);
                (exponent > 0).then_some(Factor { base, exponent })
            })
            .collect();

        Monomial { factors }.normalized()
    }

    pub(super) fn divide_by_monomial(&self, monomial: &Monomial) -> Option<Self> {
        if monomial.is_one() {
            return Some(self.clone());
        }

        let mut quotient = Self::zero();

        for term in &self.terms {
            quotient.push_term(term.divide_by_monomial(monomial)?);
        }

        Some(quotient)
    }

    pub(super) fn variables(&self) -> Vec<Expr> {
        let mut variables = Vec::new();

        for term in &self.terms {
            for factor in &term.factors {
                if !variables.iter().any(|existing| existing == &factor.base) {
                    variables.push(factor.base.clone());
                }
            }
        }

        variables.sort_by(compare_expr_canonical);
        variables
    }

    pub(super) fn total_degree(&self) -> usize {
        self.terms.iter().map(Term::total_degree).max().unwrap_or(0)
    }

    pub(super) fn leading_term<'a>(&'a self, basis: &VariableBasis) -> Option<&'a Term> {
        self.terms
            .iter()
            .max_by(|lhs, rhs| basis.compare_terms(lhs, rhs))
    }

    pub(super) fn divide_exact_by(&self, divisor: &Self) -> Option<Self> {
        if divisor.is_zero() {
            return None;
        }

        let basis = VariableBasis::from_polynomials(&[self, divisor]);
        let divisor_leading = divisor.leading_term(&basis)?.clone();
        let mut quotient = Self::zero();
        let mut remainder = self.clone();
        let max_steps = self.terms.len().saturating_mul(divisor.terms.len()).max(1) * 32;

        for _ in 0..max_steps {
            if remainder.is_zero() {
                return Some(quotient);
            }

            let remainder_leading = remainder.leading_term(&basis)?.clone();
            let quotient_term = remainder_leading.divide_by_term(&divisor_leading)?;
            quotient.push_term(quotient_term.clone());
            remainder = remainder.subtract(&divisor.multiply_by_term(&quotient_term));
        }

        None
    }

    pub(super) fn rational_content(&self) -> Option<Rational> {
        if self.is_zero() {
            return Some(Rational::zero());
        }

        let common_denominator = self.terms.iter().fold(BigInt::from(1_u8), |current, term| {
            bigint_lcm(current, term.coefficient.denominator().clone())
        });

        let common_numerator = self
            .terms
            .iter()
            .map(|term| {
                term.coefficient.numerator().clone()
                    * (&common_denominator / term.coefficient.denominator())
            })
            .fold(BigInt::from(0_u8), |current, value| {
                if current == BigInt::from(0_u8) {
                    bigint_abs(&value)
                } else {
                    bigint_gcd(current, bigint_abs(&value))
                }
            });

        Rational::new(common_numerator, common_denominator)
    }

    pub(super) fn extract_factor_parts(&self) -> Option<(Rational, Monomial, Self)> {
        if self.is_zero() {
            return Some((Rational::zero(), Monomial::one(), Self::zero()));
        }

        let mut unit = self.rational_content()?;
        let mut primitive = self.divide_by_scalar(&unit)?;
        let common_monomial = primitive.common_monomial_factor();
        primitive = primitive.divide_by_monomial(&common_monomial)?;

        let basis = VariableBasis::from_polynomials(&[&primitive]);
        if let Some(leading) = primitive.leading_term(&basis) {
            if leading.coefficient.numerator() < &BigInt::from(0_u8) {
                let minus_one = rational_from_i64(-1);
                unit = unit.mul(&minus_one);
                primitive = primitive.scale(&minus_one);
            }
        }

        Some((unit, common_monomial, primitive))
    }

    pub(super) fn find_linear_factorization(&self) -> Option<(Self, Self)> {
        if self.total_degree() <= 1 {
            return None;
        }

        for candidate in linear_factor_candidates(self) {
            if let Some(quotient) = self.divide_exact_by(&candidate) {
                return Some((candidate, quotient));
            }
        }

        None
    }

    pub(super) fn push_term(&mut self, term: Term) {
        let term = term.normalized();

        if term.coefficient.is_zero() {
            return;
        }

        if let Some(existing) = self
            .terms
            .iter_mut()
            .find(|existing| existing.factors == term.factors)
        {
            existing.coefficient = existing.coefficient.add(&term.coefficient);
        } else {
            self.terms.push(term);
        }

        self.terms.retain(|term| !term.coefficient.is_zero());
        let basis = VariableBasis::from_polynomials(&[self]);
        self.terms.sort_by(|lhs, rhs| basis.compare_terms(lhs, rhs));
    }

    pub(super) fn is_zero(&self) -> bool {
        self.terms.is_empty()
    }

    pub(super) fn as_constant_rational(&self) -> Option<Rational> {
        match self.terms.as_slice() {
            [] => Some(Rational::zero()),
            [term] if term.factors.is_empty() => Some(term.coefficient.clone()),
            _ => None,
        }
    }

    pub(super) fn is_one(&self) -> bool {
        self.as_constant_rational() == Some(Rational::one())
    }

    pub(super) fn constant_term_integer(&self) -> Option<BigInt> {
        let constant = self
            .terms
            .iter()
            .find(|term| term.factors.is_empty())
            .map(|term| term.coefficient.clone())
            .unwrap_or_else(Rational::zero);
        constant.as_integer().cloned()
    }

    pub(super) fn to_expr(&self, session: &Session) -> Result<Expr, EvalError> {
        if self.terms.is_empty() {
            return Ok(Expr::integer(0_i32.into()));
        }

        let mut term_exprs = Vec::with_capacity(self.terms.len());
        for term in &self.terms {
            term_exprs.push(term.to_expr(session)?);
        }

        session.combine_additive_args(&system_expr("Plus"), &term_exprs)
    }
}

impl VariableBasis {
    pub(super) fn from_polynomials(polynomials: &[&Polynomial]) -> Self {
        let mut variables = Vec::new();

        for polynomial in polynomials {
            for variable in polynomial.variables() {
                if !variables.iter().any(|existing| existing == &variable) {
                    variables.push(variable);
                }
            }
        }

        variables.sort_by(compare_expr_canonical);
        Self { variables }
    }

    pub(super) fn compare_terms(&self, lhs: &Term, rhs: &Term) -> std::cmp::Ordering {
        let lhs_degree = lhs.total_degree();
        let rhs_degree = rhs.total_degree();

        match lhs_degree.cmp(&rhs_degree) {
            std::cmp::Ordering::Equal => {
                for variable in &self.variables {
                    match lhs
                        .exponent_for_factor(variable)
                        .cmp(&rhs.exponent_for_factor(variable))
                    {
                        std::cmp::Ordering::Equal => {}
                        ordering => return ordering,
                    }
                }

                std::cmp::Ordering::Equal
            }
            ordering => ordering,
        }
    }
}

impl Term {
    pub(super) fn constant(coefficient: Rational) -> Self {
        Self {
            coefficient,
            factors: Vec::new(),
        }
    }

    pub(super) fn indeterminate(base: Expr) -> Self {
        Self {
            coefficient: Rational::one(),
            factors: vec![Factor { base, exponent: 1 }],
        }
    }

    pub(super) fn multiply(&self, rhs: &Self) -> Self {
        let mut factors = self.factors.clone();
        factors.extend(rhs.factors.clone());

        Self {
            coefficient: self.coefficient.mul(&rhs.coefficient),
            factors,
        }
        .normalized()
    }

    pub(super) fn total_degree(&self) -> usize {
        self.factors.iter().map(|factor| factor.exponent).sum()
    }

    pub(super) fn exponent_for_factor(&self, base: &Expr) -> usize {
        self.factors
            .iter()
            .find(|factor| factor.base == *base)
            .map(|factor| factor.exponent)
            .unwrap_or(0)
    }

    pub(super) fn divide_by_monomial(&self, monomial: &Monomial) -> Option<Self> {
        let mut quotient = self.clone();

        for factor in &monomial.factors {
            let existing = quotient
                .factors
                .iter_mut()
                .find(|existing| existing.base == factor.base)?;

            if existing.exponent < factor.exponent {
                return None;
            }

            existing.exponent -= factor.exponent;
        }

        Some(quotient.normalized())
    }

    pub(super) fn divide_by_term(&self, divisor: &Self) -> Option<Self> {
        let mut quotient = self.divide_by_monomial(&Monomial {
            factors: divisor.factors.clone(),
        })?;
        quotient.coefficient = quotient.coefficient.div(&divisor.coefficient)?;
        Some(quotient.normalized())
    }

    pub(super) fn coefficient_after_exact_monomial_power(
        &self,
        target: &Monomial,
        power: usize,
    ) -> Option<Self> {
        if target.is_one() || target.exact_multiplicity_in_term(self) != power {
            return None;
        }

        let mut remainder = self.clone();

        for target_factor in &target.factors {
            let exponent_to_remove = target_factor.exponent * power;

            if exponent_to_remove == 0 {
                continue;
            }

            let factor = remainder
                .factors
                .iter_mut()
                .find(|factor| factor.base == target_factor.base)?;
            factor.exponent -= exponent_to_remove;
        }

        Some(remainder.normalized())
    }

    pub(super) fn normalized(mut self) -> Self {
        self.factors = normalize_factors(self.factors);
        self
    }

    pub(super) fn to_expr(&self, session: &Session) -> Result<Expr, EvalError> {
        let mut factors = Vec::with_capacity(self.factors.len() + 1);

        if !self.coefficient.is_one() || self.factors.is_empty() {
            factors.push(Expr::rational(self.coefficient.clone()));
        }

        for factor in &self.factors {
            let factor_expr = if factor.exponent == 1 {
                factor.base.clone()
            } else {
                Expr::call(
                    system_expr("Power"),
                    vec![
                        factor.base.clone(),
                        Expr::integer(BigInt::from(u64::try_from(factor.exponent).unwrap())),
                    ],
                )
            };
            factors.push(factor_expr);
        }

        session.combine_multiplicative_args(&system_expr("Times"), &factors)
    }
}

impl Monomial {
    pub(super) fn one() -> Self {
        Self {
            factors: Vec::new(),
        }
    }

    pub(super) fn from_expr(expr: &Expr) -> Option<Self> {
        if let Some(value) = expr_as_exact_rational(expr) {
            return value.is_one().then(Self::one);
        }

        match expr {
            Expr::Call { head, args } if is_system_symbol_name(head, "Times") => {
                let mut factors = Vec::new();

                for arg in args {
                    factors.extend(Self::from_expr(arg)?.factors);
                }

                Some(Self { factors }.normalized())
            }
            Expr::Call { head, args }
                if is_system_symbol_name(head, "Power") && args.len() == 2 =>
            {
                let exponent = expr_as_non_negative_usize(&args[1])?;

                if exponent == 0 {
                    return Some(Self::one());
                }

                Some(Self {
                    factors: vec![Factor {
                        base: parse_indeterminate_expr(&args[0])?,
                        exponent,
                    }],
                })
            }
            _ => Some(Self {
                factors: vec![Factor {
                    base: parse_indeterminate_expr(expr)?,
                    exponent: 1,
                }],
            }),
        }
    }

    pub(super) fn from_indeterminate(expr: &Expr) -> Option<Self> {
        Some(Self {
            factors: vec![Factor {
                base: parse_indeterminate_expr(expr)?,
                exponent: 1,
            }],
        })
    }

    pub(super) fn is_one(&self) -> bool {
        self.factors.is_empty()
    }

    pub(super) fn exact_multiplicity_in_term(&self, term: &Term) -> usize {
        if self.factors.is_empty() {
            return 0;
        }

        self.factors
            .iter()
            .map(|factor| {
                let available = term.exponent_for_factor(&factor.base);
                if available < factor.exponent {
                    0
                } else {
                    available / factor.exponent
                }
            })
            .min()
            .unwrap_or(0)
    }

    pub(super) fn normalized(mut self) -> Self {
        self.factors = normalize_factors(self.factors);
        self
    }

    pub(super) fn to_polynomial_factors(&self) -> Vec<Polynomial> {
        let mut factors = Vec::new();

        for factor in &self.factors {
            for _ in 0..factor.exponent {
                factors.push(Polynomial::indeterminate(factor.base.clone()));
            }
        }

        factors
    }
}

fn normalize_factors(mut factors: Vec<Factor>) -> Vec<Factor> {
    factors.retain(|factor| factor.exponent > 0);
    factors.sort_by(|lhs, rhs| compare_expr_canonical(&lhs.base, &rhs.base));

    let mut merged: Vec<Factor> = Vec::with_capacity(factors.len());
    for factor in factors {
        if let Some(previous) = merged.last_mut() {
            if previous.base == factor.base {
                previous.exponent += factor.exponent;
                continue;
            }
        }

        merged.push(factor);
    }

    merged
}

pub(super) fn parse_coefficient_list_monomials(expr: &Expr) -> Option<Vec<Monomial>> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
            args.iter().map(Monomial::from_indeterminate).collect()
        }
        _ => Some(vec![Monomial::from_indeterminate(expr)?]),
    }
}
