use super::{
    core::{Factor, Polynomial, Term},
    *,
};
pub(super) fn rational_from_i64(value: i64) -> Rational {
    Rational::from_integer(BigInt::from(value))
}

pub(super) fn bigint_abs(value: &BigInt) -> BigInt {
    if value < &BigInt::from(0_u8) {
        -value.clone()
    } else {
        value.clone()
    }
}

pub(super) fn bigint_gcd(mut lhs: BigInt, mut rhs: BigInt) -> BigInt {
    let zero = BigInt::from(0_u8);

    while rhs != zero {
        let remainder = &lhs % &rhs;
        lhs = rhs;
        rhs = remainder;
    }

    bigint_abs(&lhs)
}

pub(super) fn bigint_lcm(lhs: BigInt, rhs: BigInt) -> BigInt {
    let zero = BigInt::from(0_u8);
    if lhs == zero || rhs == zero {
        return zero;
    }

    let gcd = bigint_gcd(lhs.clone(), rhs.clone());
    bigint_abs(&(lhs / gcd * rhs))
}

pub(super) fn candidate_coefficient_magnitudes(polynomial: &Polynomial) -> Vec<BigInt> {
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

pub(super) fn candidate_constant_magnitudes(polynomial: &Polynomial) -> Vec<BigInt> {
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

pub(super) fn small_positive_divisors(value: &BigInt, limit: u64) -> Vec<BigInt> {
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

pub(super) fn linear_factor_candidates(polynomial: &Polynomial) -> Vec<Polynomial> {
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

pub(super) fn collect_linear_candidate_vectors(
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

pub(super) fn normalized_linear_vector(
    values: &[BigInt],
    variable_count: usize,
) -> Option<Vec<BigInt>> {
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

pub(super) fn linear_polynomial_from_vector(
    variables: &[Expr],
    vector: &[BigInt],
) -> Option<Polynomial> {
    let mut terms = Vec::with_capacity(variables.len() + 1);

    for (variable, coefficient) in variables.iter().zip(vector.iter()) {
        if coefficient == &BigInt::from(0_u8) {
            continue;
        }

        terms.push(Term {
            coefficient: Rational::from_integer(coefficient.clone()),
            factors: vec![Factor {
                base: variable.clone(),
                exponent: 1,
            }],
        });
    }

    let constant = &vector[variables.len()];
    if constant != &BigInt::from(0_u8) {
        terms.push(Term::constant(Rational::from_integer(constant.clone())));
    }

    let polynomial = Polynomial::from_normalized_terms(terms);
    (!polynomial.is_zero()).then_some(polynomial)
}
