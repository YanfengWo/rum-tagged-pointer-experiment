use super::*;

impl Session {
    pub(super) fn eval_plus(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        self.combine_additive_args(head, args)
    }

    pub(super) fn eval_times(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        self.combine_multiplicative_args(head, args)
    }

    pub(super) fn eval_power(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Power", args, 2));
        }

        let Some(base) = expr_as_exact_rational(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let Some(exponent) = expr_as_signed_integer_value(&args[1]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        if exponent == BigInt::from(0_u8) && base.is_zero() {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }

        Ok(base
            .pow(&exponent)
            .map(Expr::rational)
            .unwrap_or_else(|| Expr::call(head.clone(), args.to_vec())))
    }

    pub(super) fn eval_subtract(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Subtract", args, 2));
        }

        let negated_rhs = self.combine_multiplicative_args(
            &system_expr("Times"),
            &[Expr::integer((-1_i32).into()), args[1].clone()],
        )?;
        self.combine_additive_args(&system_expr("Plus"), &[args[0].clone(), negated_rhs])
    }

    pub(super) fn eval_divide(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Divide", args, 2));
        }

        if let (Some(lhs), Some(rhs)) = (
            expr_as_exact_rational(&args[0]),
            expr_as_exact_rational(&args[1]),
        ) {
            return Ok(lhs
                .div(&rhs)
                .map(Expr::rational)
                .unwrap_or_else(|| Expr::call(head.clone(), args.to_vec())));
        }

        let reciprocal = self.eval_power(
            &system_expr("Power"),
            &[args[1].clone(), Expr::integer((-1_i32).into())],
        )?;
        self.combine_multiplicative_args(&system_expr("Times"), &[args[0].clone(), reciprocal])
    }

    pub(super) fn eval_abs(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("Abs", args, 1));
        }

        let Some(value) = expr_as_exact_rational(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::rational(
            Rational::new(bigint_abs(value.numerator()), value.denominator().clone())
                .expect("absolute exact rational keeps a non-zero denominator"),
        ))
    }

    pub(super) fn eval_factorial(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("Factorial", args, 1));
        }

        let Some(value) = expr_as_exact_integer(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        if value < BigInt::from(0_u8) {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }
        self.enforce_integer_work_limit_for_factorial(&value)?;

        let one = BigInt::from(1_u8);
        let mut remaining = value;
        let mut result = one.clone();
        let mut checks_until_resource_poll = 0_usize;
        while remaining > one {
            result *= remaining.clone();
            remaining -= BigInt::from(1_u8);

            if checks_until_resource_poll == 0 {
                self.enforce_active_resource_limits_for_expr(&Expr::integer(result.clone()))?;
                checks_until_resource_poll = 256;
            } else {
                checks_until_resource_poll -= 1;
            }
        }

        Ok(Expr::integer(result))
    }

    pub(super) fn eval_total(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }

        let Expr::Call {
            head: list_head,
            args: items,
        } = &args[0]
        else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        if !is_system_symbol_name(list_head, "List") {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }

        self.evaluate(Expr::call(system_expr("Plus"), items.clone()))
    }

    pub(super) fn eval_denominator(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("Denominator", args, 1));
        }

        match &args[0] {
            Expr::Atom(Atom::Integer(_)) => Ok(Expr::integer(1_i32.into())),
            Expr::Atom(Atom::Rational(value)) => Ok(Expr::integer(value.denominator().clone())),
            _ => Ok(Expr::call(head.clone(), args.to_vec())),
        }
    }

    fn enforce_integer_work_limit_for_factorial(&self, value: &BigInt) -> Result<(), EvalError> {
        let one = BigInt::from(1_u8);
        let work_units = if value > &one {
            value - one
        } else {
            BigInt::from(0_u8)
        };
        let limit = self.resource_limits.integer_work_limit;
        if work_units > BigInt::from(limit) {
            return Err(EvalError::IntegerWorkLimitExceeded { limit });
        }

        Ok(())
    }

    pub(super) fn eval_mod(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Mod", args, 2));
        }

        let (Some(lhs), Some(rhs)) = (
            expr_as_exact_integer(&args[0]),
            expr_as_exact_integer(&args[1]),
        ) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(mod_integer(&lhs, &rhs)
            .map(Expr::integer)
            .unwrap_or_else(|| Expr::call(head.clone(), args.to_vec())))
    }

    pub(super) fn eval_quotient(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Quotient", args, 2));
        }

        let (Some(lhs), Some(rhs)) = (
            expr_as_exact_integer(&args[0]),
            expr_as_exact_integer(&args[1]),
        ) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(floor_quotient(&lhs, &rhs)
            .map(Expr::integer)
            .unwrap_or_else(|| Expr::call(head.clone(), args.to_vec())))
    }

    pub(super) fn eval_gcd(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.is_empty() {
            return Ok(Expr::integer(0_i32.into()));
        }

        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            let Some(value) = expr_as_exact_integer(arg) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            values.push(value);
        }

        let mut gcd = bigint_abs(&values[0]);
        for value in &values[1..] {
            gcd = bigint_gcd(gcd, bigint_abs(value));
        }

        Ok(Expr::integer(gcd))
    }

    pub(super) fn eval_lcm(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.is_empty() {
            return Ok(Expr::integer(1_i32.into()));
        }

        let mut values = Vec::with_capacity(args.len());
        for arg in args {
            let Some(value) = expr_as_exact_integer(arg) else {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            };
            values.push(value);
        }

        let zero = BigInt::from(0_u8);
        let mut lcm = bigint_abs(&values[0]);

        for value in &values[1..] {
            let normalized = bigint_abs(value);
            if lcm == zero || normalized == zero {
                lcm = zero.clone();
                break;
            }

            let divisor = bigint_gcd(lcm.clone(), normalized.clone());
            lcm = (lcm / divisor) * normalized;
        }

        Ok(Expr::integer(lcm))
    }

    pub(super) fn eval_power_mod(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 3 {
            return Ok(self.invalid_call_with_exact_arg_count("PowerMod", args, 3));
        }

        let (Some(base), Some(exponent), Some(modulus)) = (
            expr_as_exact_integer(&args[0]),
            expr_as_exact_integer(&args[1]),
            expr_as_exact_integer(&args[2]),
        ) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        if exponent < BigInt::from(0_u8) {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }

        Ok(power_mod_integer(&base, &exponent, &modulus)
            .map(Expr::integer)
            .unwrap_or_else(|| Expr::call(head.clone(), args.to_vec())))
    }

    pub(super) fn combine_additive_args(
        &self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let mut numeric = Rational::zero();
        let mut combined = Vec::new();

        for arg in args {
            if let Some(value) = expr_as_exact_rational(arg) {
                numeric = numeric.add(&value);
            } else {
                combined.push(arg.clone());
            }
        }

        if !numeric.is_zero() {
            combined.push(Expr::rational(numeric));
        }

        let normalized = self.normalize_call_args_by_attributes(
            head,
            combined,
            self.attribute_set_for_head(head),
        );

        Ok(match normalized.as_slice() {
            [] => Expr::integer(0_i32.into()),
            [only] => only.clone(),
            _ => Expr::call(head.clone(), normalized),
        })
    }

    pub(super) fn combine_multiplicative_args(
        &self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let mut numeric = Rational::one();
        let mut combined = Vec::new();

        for arg in args {
            if let Some(value) = expr_as_exact_rational(arg) {
                numeric = numeric.mul(&value);

                if numeric.is_zero() {
                    return Ok(Expr::integer(0_i32.into()));
                }
            } else {
                combined.push(arg.clone());
            }
        }

        if !numeric.is_one() || combined.is_empty() {
            combined.push(Expr::rational(numeric));
        }

        let normalized = self.normalize_call_args_by_attributes(
            head,
            combined,
            self.attribute_set_for_head(head),
        );

        Ok(match normalized.as_slice() {
            [] => Expr::integer(1_i32.into()),
            [only] => only.clone(),
            _ => Expr::call(head.clone(), normalized),
        })
    }
}

pub(super) fn expr_as_exact_integer(expr: &Expr) -> Option<BigInt> {
    match expr {
        Expr::Atom(Atom::Integer(value)) => Some(value.clone()),
        _ => None,
    }
}

pub(super) fn expr_as_exact_rational(expr: &Expr) -> Option<Rational> {
    match expr {
        Expr::Atom(Atom::Integer(value)) => Some(Rational::from_integer(value.clone())),
        Expr::Atom(Atom::Rational(value)) => Some(value.clone()),
        _ => None,
    }
}

fn expr_as_signed_integer_value(expr: &Expr) -> Option<BigInt> {
    match expr {
        Expr::Atom(Atom::Integer(value)) => Some(value.clone()),
        Expr::Call { head, args } if is_system_symbol_name(head, "Times") && args.len() == 2 => {
            if is_negative_one_expr(&args[0]) {
                expr_as_exact_integer(&args[1]).map(|value| -value)
            } else if is_negative_one_expr(&args[1]) {
                expr_as_exact_integer(&args[0]).map(|value| -value)
            } else {
                None
            }
        }
        _ => None,
    }
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

fn mod_integer(lhs: &BigInt, rhs: &BigInt) -> Option<BigInt> {
    if rhs == &BigInt::from(0_u8) {
        return None;
    }

    let quotient = floor_quotient(lhs, rhs)?;
    Some(lhs - rhs * quotient)
}

fn floor_quotient(lhs: &BigInt, rhs: &BigInt) -> Option<BigInt> {
    if rhs == &BigInt::from(0_u8) {
        return None;
    }

    let zero = BigInt::from(0_u8);
    let quotient = lhs / rhs;
    let remainder = lhs % rhs;

    if remainder != zero && ((remainder > zero && rhs < &zero) || (remainder < zero && rhs > &zero))
    {
        Some(quotient - BigInt::from(1_u8))
    } else {
        Some(quotient)
    }
}

fn power_mod_integer(base: &BigInt, exponent: &BigInt, modulus: &BigInt) -> Option<BigInt> {
    if modulus == &BigInt::from(0_u8) {
        return None;
    }

    let zero = BigInt::from(0_u8);
    let one = BigInt::from(1_u8);
    let two = BigInt::from(2_u8);
    let modulus_abs = bigint_abs(modulus);
    let mut result = mod_integer(&one, modulus)?;
    let mut factor = mod_integer(base, modulus)?;
    let mut remaining = exponent.clone();

    while remaining > zero {
        if &remaining % &two == one {
            result = mod_integer(&(result * &factor), modulus)?;
        }

        remaining /= &two;

        if remaining > zero {
            factor = mod_integer(&(&factor * &factor), modulus)?;
        }
    }

    let mut reduced = mod_integer(&result, modulus)?;
    if modulus < &zero && reduced > zero {
        reduced -= modulus_abs;
    }

    Some(reduced)
}
