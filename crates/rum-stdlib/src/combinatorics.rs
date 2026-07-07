use rum_core::{BigInt, Expr, Rational};
use rum_runtime::kernel_api::{
    EvalError, Session, ToPrimitive, expr_as_exact_integer, system_expr,
};

pub(crate) fn eval_binomial(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if args.len() != 2 {
        return Ok(session.invalid_call_with_exact_arg_count("Binomial", args, 2));
    }

    let (Some(n), Some(k)) = (
        expr_as_exact_integer(&args[0]),
        expr_as_exact_integer(&args[1]),
    ) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    enforce_integer_work_limit_for_binomial(session, &n, &k)?;
    Ok(Expr::integer(binomial_integer(&n, &k)))
}

pub(crate) fn eval_fibonacci(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [arg] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("Fibonacci", args, 1));
    };

    let Some(index) = expr_as_exact_integer(arg) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(Expr::integer(fibonacci_integer(&index)))
}

pub(crate) fn eval_partitions_p(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [arg] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("PartitionsP", args, 1));
    };

    let Some(index) = expr_as_exact_integer(arg) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    if index < BigInt::from(0_u8) {
        return Ok(Expr::integer(BigInt::from(0_u8)));
    }
    let Some(index) = nonnegative_combinatorics_index(session, &index)? else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(Expr::integer(partitions_p_integer(
        index,
        session.resource_limits().integer_work_limit,
    )?))
}

pub(crate) fn eval_partitions_q(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [arg] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("PartitionsQ", args, 1));
    };

    let Some(index) = expr_as_exact_integer(arg) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    if index < BigInt::from(0_u8) {
        return Ok(Expr::integer(BigInt::from(0_u8)));
    }
    let Some(index) = nonnegative_combinatorics_index(session, &index)? else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(Expr::integer(partitions_q_integer(
        index,
        session.resource_limits().integer_work_limit,
    )?))
}

pub(crate) fn eval_bernoulli_b(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [arg] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("BernoulliB", args, 1));
    };

    let Some(index) = expr_as_exact_integer(arg) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let Some(index) = nonnegative_combinatorics_index(session, &index)? else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(Expr::rational(bernoulli_b_rational(
        index,
        session.resource_limits().integer_work_limit,
    )?))
}

pub(crate) fn eval_subfactorial(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [arg] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("Subfactorial", args, 1));
    };

    let Some(index) = expr_as_exact_integer(arg) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let Some(index) = nonnegative_combinatorics_index(session, &index)? else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let mut used = 0_usize;
    charge_combinatorics_work(
        &mut used,
        index,
        session.resource_limits().integer_work_limit,
    )?;
    Ok(Expr::integer(subfactorial_integer(index)))
}

pub(crate) fn eval_stirling_s1(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [n_arg, k_arg] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("StirlingS1", args, 2));
    };

    let (Some(n), Some(k)) = (expr_as_exact_integer(n_arg), expr_as_exact_integer(k_arg)) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let (Some(n), Some(k)) = (
        nonnegative_combinatorics_index(session, &n)?,
        nonnegative_combinatorics_index(session, &k)?,
    ) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(Expr::integer(stirling_s1_integer(
        n,
        k,
        session.resource_limits().integer_work_limit,
    )?))
}

pub(crate) fn eval_stirling_s2(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [n_arg, k_arg] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("StirlingS2", args, 2));
    };

    let (Some(n), Some(k)) = (expr_as_exact_integer(n_arg), expr_as_exact_integer(k_arg)) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let (Some(n), Some(k)) = (
        nonnegative_combinatorics_index(session, &n)?,
        nonnegative_combinatorics_index(session, &k)?,
    ) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(Expr::integer(stirling_s2_integer(
        n,
        k,
        session.resource_limits().integer_work_limit,
    )?))
}

pub(crate) fn eval_catalan_number(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [arg] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("CatalanNumber", args, 1));
    };

    let Some(index) = expr_as_exact_integer(arg) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    if index < BigInt::from(0_u8) {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    }

    let two_n = &index * BigInt::from(2_u8);
    enforce_integer_work_limit_for_binomial(session, &two_n, &index)?;
    let denominator = &index + BigInt::from(1_u8);
    Ok(Expr::integer(
        binomial_integer(&two_n, &index) / denominator,
    ))
}

pub(crate) fn eval_multinomial(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let mut values = Vec::with_capacity(args.len());
    for arg in args {
        let Some(value) = expr_as_exact_integer(arg) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        if value < BigInt::from(0_u8) {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }
        values.push(value);
    }

    Ok(Expr::integer(multinomial_integer(
        &values,
        session.resource_limits().integer_work_limit,
    )?))
}

pub(crate) fn eval_pochhammer(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [base, count] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("Pochhammer", args, 2));
    };

    let Some(count) = expr_as_exact_integer(count) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let Some(count) = nonnegative_combinatorics_index(session, &count)? else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    enforce_combinatorics_work_units(session, count)?;
    if count == 0 {
        return Ok(Expr::integer(BigInt::from(1_u8)));
    }

    let plus_head = system_expr("Plus");
    let times_head = system_expr("Times");
    let mut factors = Vec::with_capacity(count);
    for offset in 0..count {
        if offset == 0 {
            factors.push(base.clone());
        } else {
            factors.push(session.combine_additive_args(
                &plus_head,
                &[base.clone(), Expr::integer(BigInt::from(offset))],
            )?);
        }
    }

    session.combine_multiplicative_args(&times_head, &factors)
}

fn enforce_integer_work_limit_for_binomial(
    session: &Session,
    n: &BigInt,
    k: &BigInt,
) -> Result<(), EvalError> {
    let work_units = binomial_work_units(n, k);
    let limit = session.resource_limits().integer_work_limit;
    if work_units > BigInt::from(limit) {
        return Err(EvalError::IntegerWorkLimitExceeded { limit });
    }

    Ok(())
}

fn enforce_combinatorics_work_units(session: &Session, units: usize) -> Result<(), EvalError> {
    if units > session.resource_limits().integer_work_limit {
        return Err(EvalError::IntegerWorkLimitExceeded {
            limit: session.resource_limits().integer_work_limit,
        });
    }

    Ok(())
}

fn nonnegative_combinatorics_index(
    session: &Session,
    value: &BigInt,
) -> Result<Option<usize>, EvalError> {
    if value < &BigInt::from(0_u8) {
        return Ok(None);
    }

    match value.to_usize() {
        Some(value) => Ok(Some(value)),
        None => Err(EvalError::IntegerWorkLimitExceeded {
            limit: session.resource_limits().integer_work_limit,
        }),
    }
}

fn charge_combinatorics_work(
    used: &mut usize,
    units: usize,
    limit: usize,
) -> Result<(), EvalError> {
    *used = used
        .checked_add(units)
        .ok_or(EvalError::IntegerWorkLimitExceeded { limit })?;
    if *used > limit {
        return Err(EvalError::IntegerWorkLimitExceeded { limit });
    }

    Ok(())
}

fn binomial_work_units(n: &BigInt, k: &BigInt) -> BigInt {
    if k < &BigInt::from(0_u8) {
        return BigInt::from(0_u8);
    }

    if n >= &BigInt::from(0_u8) {
        if k > n {
            return BigInt::from(0_u8);
        }
        return std::cmp::min(k.clone(), n - k);
    }

    k.clone()
}

fn binomial_integer(n: &BigInt, k: &BigInt) -> BigInt {
    if k < &BigInt::from(0_u8) {
        return BigInt::from(0_u8);
    }

    if n >= &BigInt::from(0_u8) {
        if k > n {
            return BigInt::from(0_u8);
        }
        let selected_k = std::cmp::min(k.clone(), n - k);
        return binomial_nonnegative(n, &selected_k);
    }

    let top = k - n - BigInt::from(1_u8);
    let result = binomial_nonnegative(&top, k);
    if k % BigInt::from(2_u8) == BigInt::from(0_u8) {
        result
    } else {
        -result
    }
}

fn partitions_p_integer(n: usize, limit: usize) -> Result<BigInt, EvalError> {
    let mut values = vec![BigInt::from(0_u8); n + 1];
    values[0] = BigInt::from(1_u8);
    let mut used = 0_usize;

    for target in 1..=n {
        let mut total = BigInt::from(0_u8);
        let mut step = 1_usize;
        loop {
            let Some((left, right)) = generalized_pentagonal_pair(step) else {
                return Err(EvalError::IntegerWorkLimitExceeded { limit });
            };
            if left > target {
                break;
            }

            charge_combinatorics_work(&mut used, 1, limit)?;
            if step % 2 == 1 {
                total += &values[target - left];
            } else {
                total -= &values[target - left];
            }

            if right <= target {
                charge_combinatorics_work(&mut used, 1, limit)?;
                if step % 2 == 1 {
                    total += &values[target - right];
                } else {
                    total -= &values[target - right];
                }
            }

            step += 1;
        }
        values[target] = total;
    }

    Ok(values[n].clone())
}

fn generalized_pentagonal_pair(step: usize) -> Option<(usize, usize)> {
    let three_step = step.checked_mul(3)?;
    let left = step.checked_mul(three_step.checked_sub(1)?)? / 2;
    let right = step.checked_mul(three_step.checked_add(1)?)? / 2;
    Some((left, right))
}

fn partitions_q_integer(n: usize, limit: usize) -> Result<BigInt, EvalError> {
    let mut values = vec![BigInt::from(0_u8); n + 1];
    values[0] = BigInt::from(1_u8);
    let mut used = 0_usize;

    for part in 1..=n {
        for total in (part..=n).rev() {
            charge_combinatorics_work(&mut used, 1, limit)?;
            let addend = values[total - part].clone();
            values[total] += addend;
        }
    }

    Ok(values[n].clone())
}

fn bernoulli_b_rational(n: usize, limit: usize) -> Result<Rational, EvalError> {
    if n > 1 && n % 2 == 1 {
        return Ok(Rational::zero());
    }

    let mut values = Vec::with_capacity(n + 1);
    values.push(Rational::one());
    let mut used = 0_usize;

    for target in 1..=n {
        let mut sum = Rational::zero();
        for (index, bernoulli) in values.iter().enumerate() {
            charge_combinatorics_work(&mut used, 1, limit)?;
            let coefficient = binomial_nonnegative(&BigInt::from(target + 1), &BigInt::from(index));
            sum = sum.add(&bernoulli.mul(&Rational::from_integer(coefficient)));
        }

        let divisor = Rational::from_integer(BigInt::from(target + 1));
        values.push(
            sum.negated()
                .div(&divisor)
                .expect("Bernoulli recurrence divisor is positive"),
        );
    }

    Ok(values[n].clone())
}

fn subfactorial_integer(n: usize) -> BigInt {
    let mut result = BigInt::from(1_u8);
    for index in 1..=n {
        result *= BigInt::from(index);
        if index % 2 == 0 {
            result += BigInt::from(1_u8);
        } else {
            result -= BigInt::from(1_u8);
        }
    }
    result
}

fn stirling_s1_integer(n: usize, k: usize, limit: usize) -> Result<BigInt, EvalError> {
    if k > n {
        return Ok(BigInt::from(0_u8));
    }

    let mut row = vec![BigInt::from(0_u8); k + 1];
    row[0] = BigInt::from(1_u8);
    let mut used = 0_usize;

    for source_n in 1..=n {
        for column in (1..=std::cmp::min(source_n, k)).rev() {
            charge_combinatorics_work(&mut used, 1, limit)?;
            row[column] =
                row[column - 1].clone() - BigInt::from(source_n - 1) * row[column].clone();
        }
        row[0] = BigInt::from(0_u8);
    }

    Ok(row[k].clone())
}

fn stirling_s2_integer(n: usize, k: usize, limit: usize) -> Result<BigInt, EvalError> {
    if k > n {
        return Ok(BigInt::from(0_u8));
    }

    let mut row = vec![BigInt::from(0_u8); k + 1];
    row[0] = BigInt::from(1_u8);
    let mut used = 0_usize;

    for source_n in 1..=n {
        for column in (1..=std::cmp::min(source_n, k)).rev() {
            charge_combinatorics_work(&mut used, 1, limit)?;
            row[column] = BigInt::from(column) * row[column].clone() + row[column - 1].clone();
        }
        row[0] = BigInt::from(0_u8);
    }

    Ok(row[k].clone())
}

fn multinomial_integer(values: &[BigInt], limit: usize) -> Result<BigInt, EvalError> {
    let mut work_units = BigInt::from(0_u8);
    let mut total = BigInt::from(0_u8);
    for value in values {
        let next_total = &total + value;
        work_units += binomial_work_units(&next_total, value);
        if work_units > BigInt::from(limit) {
            return Err(EvalError::IntegerWorkLimitExceeded { limit });
        }
        total = next_total;
    }

    let mut result = BigInt::from(1_u8);
    let mut accumulated = BigInt::from(0_u8);
    for value in values {
        accumulated += value;
        result *= binomial_integer(&accumulated, value);
    }

    Ok(result)
}

fn binomial_nonnegative(n: &BigInt, k: &BigInt) -> BigInt {
    if k == &BigInt::from(0_u8) {
        return BigInt::from(1_u8);
    }

    let mut result = BigInt::from(1_u8);
    let mut i = BigInt::from(1_u8);
    while &i <= k {
        result *= n - k + i.clone();
        result /= i.clone();
        i += BigInt::from(1_u8);
    }
    result
}

fn fibonacci_integer(index: &BigInt) -> BigInt {
    let negative = index < &BigInt::from(0_u8);
    let magnitude = combinatorics_bigint_abs(index);
    let mut current = BigInt::from(0_u8);
    let mut next = BigInt::from(1_u8);

    for bit in magnitude.to_str_radix(2).bytes() {
        let doubled_next_minus_current = next.clone() * BigInt::from(2_u8) - current.clone();
        let even = current.clone() * doubled_next_minus_current;
        let odd = current.clone() * current.clone() + next.clone() * next.clone();
        if bit == b'0' {
            current = even;
            next = odd;
        } else {
            current = odd.clone();
            next = even + odd;
        }
    }

    if negative && magnitude % BigInt::from(2_u8) == BigInt::from(0_u8) {
        -current
    } else {
        current
    }
}

fn combinatorics_bigint_abs(value: &BigInt) -> BigInt {
    if value < &BigInt::from(0_u8) {
        -value.clone()
    } else {
        value.clone()
    }
}
