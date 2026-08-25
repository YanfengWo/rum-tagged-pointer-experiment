use rum_core::{Atom, BigInt, Expr, Rational};
use rum_runtime::kernel_api::{
    EvalError, Integer, One, Roots, Session, Signed, ThreadArgSpec, ThreadingDispatch, ToPrimitive,
    Zero, expr_as_exact_integer, expr_as_exact_rational, system_expr,
};
use std::collections::BTreeMap;
use std::sync::OnceLock;

const SMALL_TRIAL_LIMIT: usize = 100_000;
const MILLER_RABIN_WITNESSES: [u64; 12] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];
const SMALL_INDEXED_PRIMES: [usize; 5] = [2, 3, 5, 7, 11];
const SIEVE_WORK_CHUNK: usize = 16;

pub(crate) fn eval_divisors(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [arg] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("Divisors", args, 1));
    };
    let Some(value) = expr_as_exact_integer(arg) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let value = value.abs();
    if value.is_zero() {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    }

    let mut budget = IntegerWorkBudget::new(session.resource_limits().integer_work_limit);
    let factors = factorize(&value, &mut budget)?;
    let divisors = positive_divisors_from_factorization(&factors, &mut budget)?;
    Ok(Expr::call(
        system_expr("List"),
        divisors.into_iter().map(Expr::integer).collect(),
    ))
}

pub(crate) fn eval_euler_phi(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [arg] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("EulerPhi", args, 1));
    };
    let Some(value) = expr_as_exact_integer(arg) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let value = value.abs();
    if value.is_zero() {
        return Ok(Expr::integer(BigInt::zero()));
    }

    let mut budget = IntegerWorkBudget::new(session.resource_limits().integer_work_limit);
    let factors = factorize(&value, &mut budget)?;
    let mut result = value;
    for (prime, _) in factors {
        budget.charge(1)?;
        result = (result / &prime) * (prime - BigInt::one());
    }

    Ok(Expr::integer(result))
}

pub(crate) fn eval_moebius_mu(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [arg] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("MoebiusMu", args, 1));
    };
    let Some(value) = expr_as_exact_integer(arg) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let value = value.abs();
    if value.is_zero() {
        return Ok(Expr::integer(BigInt::zero()));
    }

    let mut budget = IntegerWorkBudget::new(session.resource_limits().integer_work_limit);
    let factors = factorize(&value, &mut budget)?;
    if factors.iter().any(|(_, exponent)| *exponent > 1) {
        return Ok(Expr::integer(BigInt::zero()));
    }

    let result = if factors.len() % 2 == 0 {
        1_i32
    } else {
        -1_i32
    };
    Ok(Expr::integer(BigInt::from(result)))
}

pub(crate) fn eval_divisor_sigma(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [k_arg, n_arg] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("DivisorSigma", args, 2));
    };
    let (Some(k), Some(value)) = (expr_as_exact_integer(k_arg), expr_as_exact_integer(n_arg))
    else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let value = value.abs();
    if value.is_zero() {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    }

    let mut budget = IntegerWorkBudget::new(session.resource_limits().integer_work_limit);
    let factors = factorize(&value, &mut budget)?;
    let sigma = divisor_sigma_from_factorization(&k, &factors, &mut budget)?;
    Ok(Expr::rational(sigma))
}

pub(crate) fn eval_prime(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [arg] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("Prime", args, 1));
    };
    if let Some(threaded) = try_thread_number_theory_call(session, head, args)? {
        return Ok(threaded);
    }

    let Some(index) = expr_as_exact_integer(arg) else {
        if matches!(arg, Expr::Atom(Atom::Rational(_))) {
            push_prime_intpp_message(session, head, args);
        }
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    if index <= BigInt::zero() {
        push_prime_intpp_message(session, head, args);
        return Ok(Expr::call(head.clone(), args.to_vec()));
    }

    let mut budget = IntegerWorkBudget::new(session.resource_limits().integer_work_limit);
    budget.charge_bigint_units(&index)?;
    let prime = nth_positive_prime(&index, &mut budget)?;
    Ok(Expr::integer(BigInt::from(prime)))
}

pub(crate) fn eval_prime_pi(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [arg] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("PrimePi", args, 1));
    };
    if let Some(threaded) = try_thread_number_theory_call(session, head, args)? {
        return Ok(threaded);
    }

    let Some(value) = expr_as_exact_rational(arg) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let limit = floor_exact_rational_for_number_theory(&value);
    if limit < BigInt::from(2_u8) {
        return Ok(Expr::integer(BigInt::zero()));
    }

    let mut budget = IntegerWorkBudget::new(session.resource_limits().integer_work_limit);
    let primes = primes_up_to_bigint_limit(&limit, &mut budget)?;
    Ok(Expr::integer(BigInt::from(primes.len())))
}

pub(crate) fn eval_prime_nu(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [arg] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("PrimeNu", args, 1));
    };
    if let Some(threaded) = try_thread_number_theory_call(session, head, args)? {
        return Ok(threaded);
    }

    let Some(value) = expr_as_exact_integer(arg) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let value = value.abs();
    if value.is_zero() {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    }

    let mut budget = IntegerWorkBudget::new(session.resource_limits().integer_work_limit);
    let factors = factorize(&value, &mut budget)?;
    Ok(Expr::integer(BigInt::from(factors.len())))
}

pub(crate) fn eval_prime_omega(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [arg] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("PrimeOmega", args, 1));
    };
    if let Some(threaded) = try_thread_number_theory_call(session, head, args)? {
        return Ok(threaded);
    }

    let Some(value) = expr_as_exact_integer(arg) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let value = value.abs();
    if value.is_zero() {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    }

    let mut budget = IntegerWorkBudget::new(session.resource_limits().integer_work_limit);
    let factors = factorize(&value, &mut budget)?;
    let total = factors
        .iter()
        .map(|(_, exponent)| BigInt::from(*exponent))
        .sum();
    Ok(Expr::integer(total))
}

pub(crate) fn eval_next_prime(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    match args {
        [_] | [_, _] => {}
        _ => return Ok(session.invalid_call_with_arg_range("NextPrime", args, 1, 2)),
    }
    if let Some(threaded) = try_thread_number_theory_call(session, head, args)? {
        return Ok(threaded);
    }

    let Some(start) = expr_as_exact_integer(&args[0]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let step_count = match args {
        [_] => BigInt::one(),
        [_, count] => {
            let Some(count) = expr_as_exact_integer(count) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            if count.is_zero() {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            }
            count
        }
        _ => unreachable!("argument count checked above"),
    };

    let mut budget = IntegerWorkBudget::new(session.resource_limits().integer_work_limit);
    let prime = nth_next_signed_prime(&start, &step_count, &mut budget)?;
    Ok(Expr::integer(prime))
}

fn try_thread_number_theory_call(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Option<Expr>, EvalError> {
    match session.thread_call_args(
        head,
        args,
        &system_expr("List"),
        &system_expr("List"),
        &ThreadArgSpec::All,
    )? {
        ThreadingDispatch::NotApplicable => Ok(None),
        ThreadingDispatch::Threaded(threaded) => session.evaluate(threaded).map(Some),
        ThreadingDispatch::Mismatch => Ok(Some(Expr::call(head.clone(), args.to_vec()))),
    }
}

fn push_prime_intpp_message(session: &mut Session, head: &Expr, args: &[Expr]) {
    let call = Expr::call(head.clone(), args.to_vec());
    session.push_message(
        "Prime",
        "intpp",
        format!("Positive integer argument expected in {call}."),
    );
}

#[derive(Debug)]
pub(crate) struct IntegerWorkBudget {
    limit: usize,
    used: usize,
}

impl IntegerWorkBudget {
    pub(crate) fn new(limit: usize) -> Self {
        Self { limit, used: 0 }
    }

    fn charge(&mut self, units: usize) -> Result<(), EvalError> {
        self.used = self
            .used
            .checked_add(units)
            .ok_or(EvalError::IntegerWorkLimitExceeded { limit: self.limit })?;
        if self.used > self.limit {
            return Err(EvalError::IntegerWorkLimitExceeded { limit: self.limit });
        }
        Ok(())
    }

    fn charge_bigint_units(&mut self, units: &BigInt) -> Result<(), EvalError> {
        if units.is_zero() {
            return Ok(());
        }
        let Some(units) = units.to_usize() else {
            return Err(EvalError::IntegerWorkLimitExceeded { limit: self.limit });
        };
        self.charge(units)
    }

    pub(crate) fn charge_bigint_magnitude_units(
        &mut self,
        units: &BigInt,
    ) -> Result<(), EvalError> {
        let Some(units) = units.magnitude().to_usize() else {
            return Err(EvalError::IntegerWorkLimitExceeded { limit: self.limit });
        };
        self.charge(units)
    }
}

fn factorize(n: &BigInt, budget: &mut IntegerWorkBudget) -> Result<Vec<(BigInt, u32)>, EvalError> {
    let mut remaining = n.abs();
    if remaining < BigInt::from(2_u8) {
        return Ok(Vec::new());
    }

    let mut prime_factors = Vec::new();
    let mut proven_prime_after_trial = false;

    for prime in small_trial_primes() {
        budget.charge(1)?;
        if remaining == BigInt::one() {
            break;
        }

        let prime = BigInt::from(*prime);
        if prime > Roots::sqrt(&remaining) {
            proven_prime_after_trial = true;
            break;
        }

        while (&remaining % &prime).is_zero() {
            budget.charge(1)?;
            prime_factors.push(prime.clone());
            remaining /= &prime;
        }
    }

    if remaining > BigInt::one() {
        if proven_prime_after_trial {
            prime_factors.push(remaining);
        } else {
            factor_cofactor(&remaining, &mut prime_factors, budget)?;
        }
    }

    let mut counts = BTreeMap::<BigInt, u32>::new();
    for factor in prime_factors {
        *counts.entry(factor).or_insert(0) += 1;
    }
    Ok(counts.into_iter().collect())
}

fn factor_cofactor(
    n: &BigInt,
    prime_factors: &mut Vec<BigInt>,
    budget: &mut IntegerWorkBudget,
) -> Result<(), EvalError> {
    if n <= &BigInt::one() {
        return Ok(());
    }

    if is_miller_rabin_prime(n, budget)? {
        prime_factors.push(n.clone());
        return Ok(());
    }

    let divisor = pollard_rho_brent(n, budget)?;
    if divisor <= BigInt::one() || &divisor >= n {
        return Err(EvalError::IntegerWorkLimitExceeded {
            limit: budget.limit,
        });
    }
    let quotient = n / &divisor;
    factor_cofactor(&divisor, prime_factors, budget)?;
    factor_cofactor(&quotient, prime_factors, budget)
}

fn positive_divisors_from_factorization(
    factors: &[(BigInt, u32)],
    budget: &mut IntegerWorkBudget,
) -> Result<Vec<BigInt>, EvalError> {
    let mut divisors = vec![BigInt::one()];

    for (prime, exponent) in factors {
        let existing = divisors.clone();
        let mut power = BigInt::one();
        for _ in 0..*exponent {
            budget.charge(existing.len())?;
            power *= prime;
            divisors.extend(existing.iter().map(|divisor| divisor * &power));
        }
    }

    divisors.sort();
    Ok(divisors)
}

fn divisor_sigma_from_factorization(
    k: &BigInt,
    factors: &[(BigInt, u32)],
    budget: &mut IntegerWorkBudget,
) -> Result<Rational, EvalError> {
    let mut product = Rational::one();

    for (prime, exponent) in factors {
        let mut sum = Rational::zero();
        for power_index in 0..=*exponent {
            budget.charge(1)?;
            let exponent = k * BigInt::from(power_index);
            let term = rational_prime_power(prime, &exponent, budget)?;
            sum = sum.add(&term);
        }
        product = product.mul(&sum);
    }

    Ok(product)
}

fn rational_prime_power(
    prime: &BigInt,
    exponent: &BigInt,
    budget: &mut IntegerWorkBudget,
) -> Result<Rational, EvalError> {
    budget.charge_bigint_units(&exponent.abs())?;
    Rational::from_integer(prime.clone())
        .pow(exponent)
        .ok_or(EvalError::IntegerWorkLimitExceeded {
            limit: budget.limit,
        })
}

fn nth_positive_prime(index: &BigInt, budget: &mut IntegerWorkBudget) -> Result<usize, EvalError> {
    let Some(index) = index.to_usize() else {
        return Err(EvalError::IntegerWorkLimitExceeded {
            limit: budget.limit,
        });
    };
    if let Some(prime) = SMALL_INDEXED_PRIMES.get(index - 1) {
        return Ok(*prime);
    }

    let mut sieve_limit = nth_prime_upper_bound(index, budget.limit)?;
    loop {
        let primes = primes_up_to_usize_limit(sieve_limit, budget)?;
        if let Some(prime) = primes.get(index - 1) {
            return Ok(*prime);
        }

        sieve_limit = sieve_limit
            .checked_mul(2)
            .ok_or(EvalError::IntegerWorkLimitExceeded {
                limit: budget.limit,
            })?;
    }
}

fn nth_prime_upper_bound(index: usize, limit: usize) -> Result<usize, EvalError> {
    debug_assert!(index >= 6);

    let bit_len = usize::BITS as usize - index.leading_zeros() as usize;
    index
        .checked_mul(bit_len)
        .and_then(|bound| bound.checked_mul(2))
        .and_then(|bound| bound.checked_add(3))
        .ok_or(EvalError::IntegerWorkLimitExceeded { limit })
}

fn primes_up_to_bigint_limit(
    limit: &BigInt,
    budget: &mut IntegerWorkBudget,
) -> Result<Vec<usize>, EvalError> {
    let Some(limit) = limit.to_usize() else {
        return Err(EvalError::IntegerWorkLimitExceeded {
            limit: budget.limit,
        });
    };
    primes_up_to_usize_limit(limit, budget)
}

fn primes_up_to_usize_limit(
    limit: usize,
    budget: &mut IntegerWorkBudget,
) -> Result<Vec<usize>, EvalError> {
    if limit < 2 {
        return Ok(Vec::new());
    }

    budget.charge(sieve_work_units(limit))?;
    if limit <= SMALL_TRIAL_LIMIT {
        return Ok(small_trial_primes()
            .iter()
            .copied()
            .take_while(|prime| (*prime as usize) <= limit)
            .map(|prime| prime as usize)
            .collect());
    }

    let len = limit
        .checked_add(1)
        .ok_or(EvalError::IntegerWorkLimitExceeded {
            limit: budget.limit,
        })?;
    let mut composite = vec![false; len];
    let mut primes = Vec::new();

    for candidate in 2..=limit {
        if composite[candidate] {
            continue;
        }

        primes.push(candidate);
        if candidate <= limit / candidate {
            let mut multiple = candidate * candidate;
            while multiple <= limit {
                composite[multiple] = true;
                multiple += candidate;
            }
        }
    }

    Ok(primes)
}

fn sieve_work_units(limit: usize) -> usize {
    limit.saturating_add(SIEVE_WORK_CHUNK - 1) / SIEVE_WORK_CHUNK
}

fn floor_exact_rational_for_number_theory(value: &Rational) -> BigInt {
    let numerator = value.numerator();
    let denominator = value.denominator();
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;

    if remainder.is_zero() || !numerator.is_negative() {
        quotient
    } else {
        quotient - BigInt::one()
    }
}

fn nth_next_signed_prime(
    start: &BigInt,
    step_count: &BigInt,
    budget: &mut IntegerWorkBudget,
) -> Result<BigInt, EvalError> {
    budget.charge_bigint_units(&step_count.abs())?;

    let direction = if step_count.is_negative() {
        -1_i32
    } else {
        1_i32
    };
    let mut remaining = step_count.abs();
    let mut current = start.clone();

    while !remaining.is_zero() {
        current = adjacent_signed_prime(&current, direction, budget)?;
        remaining -= BigInt::one();
    }

    Ok(current)
}

fn adjacent_signed_prime(
    start: &BigInt,
    direction: i32,
    budget: &mut IntegerWorkBudget,
) -> Result<BigInt, EvalError> {
    let step = BigInt::from(direction);
    let mut candidate = start + &step;

    loop {
        budget.charge(1)?;
        let abs_candidate = candidate.abs();
        if abs_candidate >= BigInt::from(2_u8) && is_miller_rabin_prime(&abs_candidate, budget)? {
            return Ok(candidate);
        }
        candidate += &step;
    }
}

fn is_miller_rabin_prime(n: &BigInt, budget: &mut IntegerWorkBudget) -> Result<bool, EvalError> {
    let two = BigInt::from(2_u8);
    if n < &two {
        return Ok(false);
    }

    for witness in MILLER_RABIN_WITNESSES {
        let witness = BigInt::from(witness);
        if n == &witness {
            return Ok(true);
        }
        if n > &witness && (n % &witness).is_zero() {
            return Ok(false);
        }
    }

    if n.is_even() {
        return Ok(false);
    }

    let one = BigInt::one();
    let n_minus_one = n - &one;
    let mut d = n_minus_one.clone();
    let mut s = 0_usize;
    while d.is_even() {
        budget.charge(1)?;
        d /= &two;
        s += 1;
    }

    // These witnesses are deterministic for n < 3.317e24; for larger inputs
    // this remains the accepted folk-scope heuristic under the work limit.
    for witness in MILLER_RABIN_WITNESSES {
        let a = BigInt::from(witness);
        if &a >= n {
            continue;
        }

        budget.charge(1)?;
        let mut x = a.modpow(&d, n);
        if x == one || x == n_minus_one {
            continue;
        }

        let mut passed = false;
        for _ in 1..s {
            budget.charge(1)?;
            x = (&x * &x) % n;
            if x == n_minus_one {
                passed = true;
                break;
            }
        }

        if !passed {
            return Ok(false);
        }
    }

    Ok(true)
}

fn pollard_rho_brent(n: &BigInt, budget: &mut IntegerWorkBudget) -> Result<BigInt, EvalError> {
    if n.is_even() {
        return Ok(BigInt::from(2_u8));
    }

    let one = BigInt::one();
    let batch_size = 128_usize;
    let mut c = BigInt::one();
    let mut seed = BigInt::from(2_u8);

    loop {
        let mut y = seed.clone();
        let mut r = 1_usize;
        let mut g = one.clone();
        let mut x = BigInt::zero();
        let mut ys = BigInt::zero();

        while g == one {
            x = y.clone();
            for _ in 0..r {
                budget.charge(1)?;
                y = pollard_step(&y, &c, n);
            }

            let mut k = 0_usize;
            let mut q = one.clone();
            while k < r && g == one {
                ys = y.clone();
                let chunk = std::cmp::min(batch_size, r - k);
                for _ in 0..chunk {
                    budget.charge(1)?;
                    y = pollard_step(&y, &c, n);
                    let diff = (&x - &y).abs();
                    q = (q * diff) % n;
                }
                budget.charge(1)?;
                g = q.gcd(n);
                k += chunk;
            }

            r = r
                .checked_mul(2)
                .ok_or(EvalError::IntegerWorkLimitExceeded {
                    limit: budget.limit,
                })?;
        }

        if &g != n {
            return Ok(g);
        }

        loop {
            budget.charge(1)?;
            ys = pollard_step(&ys, &c, n);
            g = (&x - &ys).abs().gcd(n);
            if g > one && &g < n {
                return Ok(g);
            }
            if &g == n {
                break;
            }
        }

        budget.charge(1)?;
        c += BigInt::one();
        seed += BigInt::one();
    }
}

fn pollard_step(value: &BigInt, c: &BigInt, modulus: &BigInt) -> BigInt {
    ((value * value) + c) % modulus
}

fn small_trial_primes() -> &'static [u32] {
    static PRIMES: OnceLock<Vec<u32>> = OnceLock::new();

    PRIMES.get_or_init(|| {
        let mut composite = vec![false; SMALL_TRIAL_LIMIT + 1];
        let mut primes = Vec::new();
        for candidate in 2..=SMALL_TRIAL_LIMIT {
            if composite[candidate] {
                continue;
            }

            primes.push(candidate as u32);
            if candidate * candidate <= SMALL_TRIAL_LIMIT {
                for multiple in (candidate * candidate..=SMALL_TRIAL_LIMIT).step_by(candidate) {
                    composite[multiple] = true;
                }
            }
        }
        primes
    })
}
