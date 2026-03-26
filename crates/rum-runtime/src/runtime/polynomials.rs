use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct Polynomial {
    terms: Vec<Term>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Term {
    coefficient: Rational,
    factors: Vec<Factor>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Factor {
    base: Expr,
    exponent: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Monomial {
    factors: Vec<Factor>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RationalFunction {
    numerator: Polynomial,
    denominator: Polynomial,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct PolynomialFactorization {
    unit: Rational,
    factors: Vec<Polynomial>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct VariableBasis {
    variables: Vec<Expr>,
}

impl Session {
    pub(super) fn eval_expand(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("Expand", args, 1));
        }

        let Some(polynomial) = Polynomial::from_expr(&args[0]) else {
            return Ok(args[0].clone());
        };

        polynomial.to_expr(self)
    }

    pub(super) fn eval_coefficient(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(2..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Coefficient", args, 2, 3));
        }

        let Some(polynomial) = Polynomial::from_expr(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let Some(target) = Monomial::from_expr(&args[1]).filter(|target| !target.is_one()) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let power = match args.get(2) {
            None => 1,
            Some(arg) => {
                if let Some(value) = expr_as_non_negative_usize(arg) {
                    value
                } else if let Some(value) = expr_as_exact_integer(arg) {
                    if value < BigInt::from(0_u8) {
                        return Ok(Expr::integer(0_i32.into()));
                    }

                    return Ok(Expr::call(head.clone(), args.to_vec()));
                } else {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                }
            }
        };

        polynomial
            .coefficient_for_monomial_power(&target, power)
            .to_expr(self)
    }

    pub(super) fn eval_coefficient_list(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("CoefficientList", args, 2));
        }

        let Some(polynomial) = Polynomial::from_expr(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let Some(variables) = parse_coefficient_list_monomials(&args[1]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        polynomial.coefficient_list_expr(&variables, self)
    }

    pub(super) fn eval_together(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("Together", args, 1));
        }

        let Some(rational_function) = RationalFunction::from_expr(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        rational_function.to_expr(self)
    }

    pub(super) fn eval_factor(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("Factor", args, 1));
        }

        let Some(polynomial) = Polynomial::from_expr(&args[0]) else {
            return Ok(args[0].clone());
        };

        PolynomialFactorization::from_polynomial(polynomial).to_expr(self)
    }
}

impl Polynomial {
    fn zero() -> Self {
        Self { terms: Vec::new() }
    }

    fn one() -> Self {
        Self::constant(Rational::one())
    }

    fn constant(value: Rational) -> Self {
        if value.is_zero() {
            Self::zero()
        } else {
            Self {
                terms: vec![Term::constant(value)],
            }
        }
    }

    fn indeterminate(base: Expr) -> Self {
        Self {
            terms: vec![Term::indeterminate(base)],
        }
    }

    fn from_expr(expr: &Expr) -> Option<Self> {
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

    fn add(mut self, rhs: Self) -> Self {
        for term in rhs.terms {
            self.push_term(term);
        }

        self
    }

    fn multiply(&self, rhs: &Self) -> Self {
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

    fn pow(&self, exponent: usize) -> Self {
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

    fn scale(&self, scalar: &Rational) -> Self {
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

    fn coefficient_for_monomial_power(&self, target: &Monomial, power: usize) -> Self {
        let mut coefficient = Self::zero();

        for term in &self.terms {
            if let Some(remainder) = term.coefficient_after_exact_monomial_power(target, power) {
                coefficient.push_term(remainder);
            }
        }

        coefficient
    }

    fn max_exact_power_for_monomial(&self, target: &Monomial) -> usize {
        self.terms
            .iter()
            .map(|term| target.exact_multiplicity_in_term(term))
            .max()
            .unwrap_or(0)
    }

    fn coefficient_list_expr(
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

    fn coefficient_grid_expr(
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

    fn subtract(&self, rhs: &Self) -> Self {
        self.clone().add(rhs.scale(&rational_from_i64(-1)))
    }

    fn multiply_by_term(&self, term: &Term) -> Self {
        let mut product = Self::zero();

        for existing in &self.terms {
            product.push_term(existing.multiply(term));
        }

        product
    }

    fn divide_by_scalar(&self, scalar: &Rational) -> Option<Self> {
        Some(self.scale(&scalar.reciprocal()?))
    }

    fn common_monomial_factor(&self) -> Monomial {
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

    fn divide_by_monomial(&self, monomial: &Monomial) -> Option<Self> {
        if monomial.is_one() {
            return Some(self.clone());
        }

        let mut quotient = Self::zero();

        for term in &self.terms {
            quotient.push_term(term.divide_by_monomial(monomial)?);
        }

        Some(quotient)
    }

    fn variables(&self) -> Vec<Expr> {
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

    fn total_degree(&self) -> usize {
        self.terms.iter().map(Term::total_degree).max().unwrap_or(0)
    }

    fn leading_term<'a>(&'a self, basis: &VariableBasis) -> Option<&'a Term> {
        self.terms
            .iter()
            .max_by(|lhs, rhs| basis.compare_terms(lhs, rhs))
    }

    fn divide_exact_by(&self, divisor: &Self) -> Option<Self> {
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

    fn rational_content(&self) -> Option<Rational> {
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

    fn extract_factor_parts(&self) -> Option<(Rational, Monomial, Self)> {
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

    fn find_linear_factorization(&self) -> Option<(Self, Self)> {
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

    fn push_term(&mut self, term: Term) {
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

    fn is_zero(&self) -> bool {
        self.terms.is_empty()
    }

    fn as_constant_rational(&self) -> Option<Rational> {
        match self.terms.as_slice() {
            [] => Some(Rational::zero()),
            [term] if term.factors.is_empty() => Some(term.coefficient.clone()),
            _ => None,
        }
    }

    fn is_one(&self) -> bool {
        self.as_constant_rational() == Some(Rational::one())
    }

    fn constant_term_integer(&self) -> Option<BigInt> {
        let constant = self
            .terms
            .iter()
            .find(|term| term.factors.is_empty())
            .map(|term| term.coefficient.clone())
            .unwrap_or_else(Rational::zero);
        constant.as_integer().cloned()
    }

    fn to_expr(&self, session: &Session) -> Result<Expr, EvalError> {
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

impl RationalFunction {
    fn one() -> Self {
        Self::from_polynomial(Polynomial::one())
    }

    fn from_polynomial(polynomial: Polynomial) -> Self {
        Self {
            numerator: polynomial,
            denominator: Polynomial::one(),
        }
    }

    fn from_expr(expr: &Expr) -> Option<Self> {
        match expr {
            Expr::Atom(_) => Some(Self::from_polynomial(Polynomial::from_expr(expr)?)),
            Expr::Call { head, args } if is_system_symbol_name(head, "Plus") => {
                let mut result = Self::from_polynomial(Polynomial::zero());

                for arg in args {
                    result = result.add(Self::from_expr(arg)?)?;
                }

                Some(result)
            }
            Expr::Call { head, args } if is_system_symbol_name(head, "Times") => {
                let mut result = Self::one();

                for arg in args {
                    result = result.multiply(Self::from_expr(arg)?)?;
                }

                Some(result)
            }
            Expr::Call { head, args }
                if is_system_symbol_name(head, "Power") && args.len() == 2 =>
            {
                let exponent = expr_as_exact_integer(&args[1])?;

                if exponent == BigInt::from(0_u8) {
                    let base = Self::from_expr(&args[0])?;
                    return (!base.numerator.is_zero()).then(Self::one);
                }

                if exponent > BigInt::from(0_u8) {
                    let exponent = exponent.to_string().parse::<usize>().ok()?;
                    return Self::from_expr(&args[0])?.pow(exponent);
                }

                let magnitude = (-exponent).to_string().parse::<usize>().ok()?;
                Self::from_expr(&args[0])?.reciprocal()?.pow(magnitude)
            }
            _ => Some(Self::from_polynomial(Polynomial::from_expr(expr)?)),
        }
    }

    fn add(self, rhs: Self) -> Option<Self> {
        let numerator = self
            .numerator
            .multiply(&rhs.denominator)
            .add(rhs.numerator.multiply(&self.denominator));
        let denominator = self.denominator.multiply(&rhs.denominator);

        Self {
            numerator,
            denominator,
        }
        .normalized()
    }

    fn multiply(self, rhs: Self) -> Option<Self> {
        Self {
            numerator: self.numerator.multiply(&rhs.numerator),
            denominator: self.denominator.multiply(&rhs.denominator),
        }
        .normalized()
    }

    fn pow(self, exponent: usize) -> Option<Self> {
        if exponent == 0 {
            return (!self.numerator.is_zero()).then(Self::one);
        }

        Some(Self {
            numerator: self.numerator.pow(exponent),
            denominator: self.denominator.pow(exponent),
        })
    }

    fn reciprocal(self) -> Option<Self> {
        (!self.numerator.is_zero())
            .then_some(Self {
                numerator: self.denominator,
                denominator: self.numerator,
            })
            .and_then(Self::normalized)
    }

    fn normalized(self) -> Option<Self> {
        if self.denominator.is_zero() {
            return None;
        }

        if self.numerator.is_zero() {
            return Some(Self::from_polynomial(Polynomial::zero()));
        }

        let cancelled = self.cancel_common_factors()?;

        if let Some(scale) = cancelled.denominator.as_constant_rational() {
            return Some(Self::from_polynomial(
                cancelled.numerator.scale(&scale.reciprocal()?),
            ));
        }

        Some(cancelled)
    }

    fn to_expr(&self, session: &Session) -> Result<Expr, EvalError> {
        let numerator = self.numerator.to_expr(session)?;

        if self.denominator.as_constant_rational() == Some(Rational::one()) {
            return Ok(numerator);
        }

        let denominator = self.denominator.to_expr(session)?;
        let reciprocal = Expr::call(
            system_expr("Power"),
            vec![denominator, Expr::integer(BigInt::from(-1_i32))],
        );

        session.combine_multiplicative_args(&system_expr("Times"), &[numerator, reciprocal])
    }

    fn cancel_common_factors(self) -> Option<Self> {
        let numerator_factors = PolynomialFactorization::from_polynomial(self.numerator);
        let denominator_factors = PolynomialFactorization::from_polynomial(self.denominator);
        let mut remaining_numerator = numerator_factors.factors;
        let mut remaining_denominator = denominator_factors.factors;

        let mut index = 0;
        while index < remaining_numerator.len() {
            if let Some(position) = remaining_denominator
                .iter()
                .position(|factor| factor == &remaining_numerator[index])
            {
                remaining_numerator.remove(index);
                remaining_denominator.remove(position);
            } else {
                index += 1;
            }
        }

        let unit = numerator_factors.unit.div(&denominator_factors.unit)?;

        Some(Self {
            numerator: PolynomialFactorization {
                unit,
                factors: remaining_numerator,
            }
            .to_polynomial(),
            denominator: PolynomialFactorization {
                unit: Rational::one(),
                factors: remaining_denominator,
            }
            .to_polynomial(),
        })
    }
}

impl PolynomialFactorization {
    fn from_polynomial(polynomial: Polynomial) -> Self {
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

    fn extend(&mut self, rhs: Self) {
        self.unit = self.unit.mul(&rhs.unit);
        self.factors.extend(rhs.factors);
    }

    fn to_polynomial(&self) -> Polynomial {
        if self.unit.is_zero() {
            return Polynomial::zero();
        }

        let mut polynomial = Polynomial::constant(self.unit.clone());

        for factor in &self.factors {
            polynomial = polynomial.multiply(factor);
        }

        polynomial
    }

    fn to_expr(&self, session: &Session) -> Result<Expr, EvalError> {
        if self.unit.is_zero() {
            return Ok(Expr::integer(0_i32.into()));
        }

        let mut items = Vec::new();
        if !self.unit.is_one() || self.factors.is_empty() {
            items.push(Expr::rational(self.unit.clone()));
        }

        let mut grouped: Vec<(Polynomial, usize)> = Vec::new();
        for factor in &self.factors {
            if let Some((_, count)) = grouped.iter_mut().find(|(existing, _)| existing == factor) {
                *count += 1;
            } else {
                grouped.push((factor.clone(), 1));
            }
        }

        for (factor, count) in grouped {
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

impl VariableBasis {
    fn from_polynomials(polynomials: &[&Polynomial]) -> Self {
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

    fn compare_terms(&self, lhs: &Term, rhs: &Term) -> std::cmp::Ordering {
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
    fn constant(coefficient: Rational) -> Self {
        Self {
            coefficient,
            factors: Vec::new(),
        }
    }

    fn indeterminate(base: Expr) -> Self {
        Self {
            coefficient: Rational::one(),
            factors: vec![Factor { base, exponent: 1 }],
        }
    }

    fn multiply(&self, rhs: &Self) -> Self {
        let mut factors = self.factors.clone();
        factors.extend(rhs.factors.clone());

        Self {
            coefficient: self.coefficient.mul(&rhs.coefficient),
            factors,
        }
        .normalized()
    }

    fn total_degree(&self) -> usize {
        self.factors.iter().map(|factor| factor.exponent).sum()
    }

    fn exponent_for_factor(&self, base: &Expr) -> usize {
        self.factors
            .iter()
            .find(|factor| factor.base == *base)
            .map(|factor| factor.exponent)
            .unwrap_or(0)
    }

    fn divide_by_monomial(&self, monomial: &Monomial) -> Option<Self> {
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

    fn divide_by_term(&self, divisor: &Self) -> Option<Self> {
        let mut quotient = self.divide_by_monomial(&Monomial {
            factors: divisor.factors.clone(),
        })?;
        quotient.coefficient = quotient.coefficient.div(&divisor.coefficient)?;
        Some(quotient.normalized())
    }

    fn coefficient_after_exact_monomial_power(
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

    fn normalized(mut self) -> Self {
        self.factors = normalize_factors(self.factors);
        self
    }

    fn to_expr(&self, session: &Session) -> Result<Expr, EvalError> {
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
    fn one() -> Self {
        Self {
            factors: Vec::new(),
        }
    }

    fn from_expr(expr: &Expr) -> Option<Self> {
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

    fn from_indeterminate(expr: &Expr) -> Option<Self> {
        Some(Self {
            factors: vec![Factor {
                base: parse_indeterminate_expr(expr)?,
                exponent: 1,
            }],
        })
    }

    fn is_one(&self) -> bool {
        self.factors.is_empty()
    }

    fn exact_multiplicity_in_term(&self, term: &Term) -> usize {
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

    fn normalized(mut self) -> Self {
        self.factors = normalize_factors(self.factors);
        self
    }

    fn to_polynomial_factors(&self) -> Vec<Polynomial> {
        let mut factors = Vec::new();

        for factor in &self.factors {
            for _ in 0..factor.exponent {
                factors.push(Polynomial::indeterminate(factor.base.clone()));
            }
        }

        factors
    }
}

pub(super) fn expr_contains_pattern_language(expr: &Expr) -> bool {
    match expr {
        Expr::Call { head, args } => {
            is_pattern_language_head(head)
                || expr_contains_pattern_language(head)
                || args.iter().any(expr_contains_pattern_language)
        }
        _ => false,
    }
}

fn is_pattern_language_head(expr: &Expr) -> bool {
    matches!(
        expr_as_symbol(expr),
        Some(symbol)
            if symbol.context() == SYSTEM_CONTEXT
                && matches!(
                    symbol.name(),
                    "Alternatives"
                        | "Blank"
                        | "BlankNullSequence"
                        | "BlankSequence"
                        | "Condition"
                        | "Except"
                        | "HoldPattern"
                        | "Optional"
                        | "Pattern"
                        | "PatternTest"
                        | "Repeated"
                        | "RepeatedNull"
                        | "Verbatim"
                )
    )
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

fn parse_coefficient_list_monomials(expr: &Expr) -> Option<Vec<Monomial>> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
            args.iter().map(Monomial::from_indeterminate).collect()
        }
        _ => Some(vec![Monomial::from_indeterminate(expr)?]),
    }
}

pub(super) fn parse_indeterminate_expr(expr: &Expr) -> Option<Expr> {
    if expr_contains_pattern_language(expr) {
        return None;
    }

    match expr {
        Expr::Atom(Atom::Integer(_))
        | Expr::Atom(Atom::Rational(_))
        | Expr::Atom(Atom::String(_)) => None,
        Expr::Call { head, .. }
            if is_system_symbol_name(head, "Plus")
                || is_system_symbol_name(head, "Times")
                || is_system_symbol_name(head, "Power") =>
        {
            None
        }
        _ => Some(expr.clone()),
    }
}

fn rational_from_i64(value: i64) -> Rational {
    Rational::from_integer(BigInt::from(value))
}

fn bigint_abs(value: &BigInt) -> BigInt {
    if value < &BigInt::from(0_u8) {
        -value.clone()
    } else {
        value.clone()
    }
}

fn bigint_gcd(mut lhs: BigInt, mut rhs: BigInt) -> BigInt {
    let zero = BigInt::from(0_u8);

    while rhs != zero {
        let remainder = &lhs % &rhs;
        lhs = rhs;
        rhs = remainder;
    }

    bigint_abs(&lhs)
}

fn bigint_lcm(lhs: BigInt, rhs: BigInt) -> BigInt {
    let zero = BigInt::from(0_u8);
    if lhs == zero || rhs == zero {
        return zero;
    }

    let gcd = bigint_gcd(lhs.clone(), rhs.clone());
    bigint_abs(&(lhs / gcd * rhs))
}

fn candidate_coefficient_magnitudes(polynomial: &Polynomial) -> Vec<BigInt> {
    let mut values = vec![BigInt::from(1_u8)];
    let limit = if polynomial.variables().len() <= 3 {
        4_u64
    } else {
        2_u64
    };

    for term in &polynomial.terms {
        let Some(integer) = term.coefficient.as_integer() else {
            continue;
        };

        for divisor in small_positive_divisors(integer, limit) {
            if !values.iter().any(|existing| existing == &divisor) {
                values.push(divisor);
            }
        }
    }

    values.sort();
    values
}

fn candidate_constant_magnitudes(polynomial: &Polynomial) -> Vec<BigInt> {
    let Some(constant) = polynomial.constant_term_integer() else {
        return Vec::new();
    };

    if constant == BigInt::from(0_u8) {
        return Vec::new();
    }

    let mut values = Vec::new();
    for divisor in small_positive_divisors(&constant, 6_u64) {
        if !values.iter().any(|existing| existing == &divisor) {
            values.push(divisor);
        }
    }
    values.sort();
    values
}

fn small_positive_divisors(value: &BigInt, limit: u64) -> Vec<BigInt> {
    let absolute = bigint_abs(value);
    if absolute == BigInt::from(0_u8) {
        return Vec::new();
    }

    let mut divisors = Vec::new();
    for candidate in 1..=limit {
        let candidate = BigInt::from(candidate);
        if &absolute % &candidate == BigInt::from(0_u8) {
            divisors.push(candidate);
        }
    }

    divisors
}

fn linear_factor_candidates(polynomial: &Polynomial) -> Vec<Polynomial> {
    let variables = polynomial.variables();
    if variables.is_empty() {
        return Vec::new();
    }

    let coefficient_magnitudes = candidate_coefficient_magnitudes(polynomial);
    let constant_magnitudes = candidate_constant_magnitudes(polynomial);
    let mut choices = Vec::with_capacity(coefficient_magnitudes.len() * 2 + 1);
    choices.push(BigInt::from(0_u8));
    for magnitude in &coefficient_magnitudes {
        choices.push(magnitude.clone());
        choices.push(-magnitude.clone());
    }

    let mut constant_choices = vec![BigInt::from(0_u8)];
    for magnitude in &constant_magnitudes {
        constant_choices.push(magnitude.clone());
        constant_choices.push(-magnitude.clone());
    }

    let mut vectors = BTreeSet::new();
    let mut current = vec![BigInt::from(0_u8); variables.len() + 1];
    collect_linear_candidate_vectors(
        &choices,
        &constant_choices,
        &mut current,
        0,
        variables.len(),
        &mut vectors,
    );

    let mut normalized = vectors.into_iter().collect::<Vec<_>>();
    normalized.sort_by(|lhs, rhs| {
        let lhs_nonzero = lhs
            .iter()
            .filter(|value| **value != BigInt::from(0_u8))
            .count();
        let rhs_nonzero = rhs
            .iter()
            .filter(|value| **value != BigInt::from(0_u8))
            .count();
        lhs_nonzero.cmp(&rhs_nonzero).then_with(|| {
            let lhs_sum: BigInt = lhs.iter().map(bigint_abs).sum();
            let rhs_sum: BigInt = rhs.iter().map(bigint_abs).sum();
            lhs_sum.cmp(&rhs_sum)
        })
    });

    normalized
        .into_iter()
        .filter_map(|vector| linear_polynomial_from_vector(&variables, &vector))
        .collect()
}

fn collect_linear_candidate_vectors(
    choices: &[BigInt],
    constant_choices: &[BigInt],
    current: &mut [BigInt],
    index: usize,
    variable_count: usize,
    vectors: &mut BTreeSet<Vec<BigInt>>,
) {
    let pool = if index < variable_count {
        choices
    } else {
        constant_choices
    };

    for value in pool {
        current[index] = value.clone();

        if index + 1 == current.len() {
            if let Some(vector) = normalized_linear_vector(current, variable_count) {
                vectors.insert(vector);
            }
        } else {
            collect_linear_candidate_vectors(
                choices,
                constant_choices,
                current,
                index + 1,
                variable_count,
                vectors,
            );
        }
    }
}

fn normalized_linear_vector(values: &[BigInt], variable_count: usize) -> Option<Vec<BigInt>> {
    if values[..variable_count]
        .iter()
        .all(|value| *value == BigInt::from(0_u8))
    {
        return None;
    }

    let gcd = values
        .iter()
        .filter(|value| **value != BigInt::from(0_u8))
        .map(bigint_abs)
        .reduce(bigint_gcd)
        .unwrap_or_else(|| BigInt::from(1_u8));

    let mut normalized = values.iter().map(|value| value / &gcd).collect::<Vec<_>>();

    if normalized
        .iter()
        .find(|value| **value != BigInt::from(0_u8))
        .map(|value| value < &BigInt::from(0_u8))
        .unwrap_or(false)
    {
        for value in &mut normalized {
            *value = -value.clone();
        }
    }

    Some(normalized)
}

fn linear_polynomial_from_vector(variables: &[Expr], vector: &[BigInt]) -> Option<Polynomial> {
    let mut polynomial = Polynomial::zero();

    for (variable, coefficient) in variables.iter().zip(vector.iter()) {
        if coefficient == &BigInt::from(0_u8) {
            continue;
        }

        polynomial.push_term(Term {
            coefficient: Rational::from_integer(coefficient.clone()),
            factors: vec![Factor {
                base: variable.clone(),
                exponent: 1,
            }],
        });
    }

    let constant = &vector[variables.len()];
    if constant != &BigInt::from(0_u8) {
        polynomial.push_term(Term::constant(Rational::from_integer(constant.clone())));
    }

    (!polynomial.is_zero()).then_some(polynomial)
}
