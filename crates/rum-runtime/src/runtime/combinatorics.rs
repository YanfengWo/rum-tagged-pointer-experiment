use super::*;

impl Session {
    pub(super) fn eval_binomial(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Binomial", args, 2));
        }

        let (Some(n), Some(k)) = (
            expr_as_exact_integer(&args[0]),
            expr_as_exact_integer(&args[1]),
        ) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        self.enforce_integer_work_limit_for_binomial(&n, &k)?;
        Ok(Expr::integer(binomial_integer(&n, &k)))
    }

    pub(super) fn eval_fibonacci(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [arg] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let Some(index) = expr_as_exact_integer(arg) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::integer(fibonacci_integer(&index)))
    }

    fn enforce_integer_work_limit_for_binomial(
        &self,
        n: &BigInt,
        k: &BigInt,
    ) -> Result<(), EvalError> {
        let work_units = binomial_work_units(n, k);
        let limit = self.resource_limits.integer_work_limit;
        if work_units > BigInt::from(limit) {
            return Err(EvalError::IntegerWorkLimitExceeded { limit });
        }

        Ok(())
    }
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
