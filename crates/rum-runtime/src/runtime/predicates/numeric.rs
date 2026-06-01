use super::*;

impl Session {
    pub(in crate::runtime) fn eval_integer_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("IntegerQ", args, 1));
        };

        Ok(predicate_bool_expr(matches!(
            expr,
            Expr::Atom(Atom::Integer(_))
        )))
    }

    pub(in crate::runtime) fn eval_number_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("NumberQ", args, 1));
        };

        Ok(predicate_bool_expr(matches!(
            expr,
            Expr::Atom(Atom::Integer(_) | Atom::Rational(_))
        )))
    }

    pub(in crate::runtime) fn eval_numeric_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("NumericQ", args, 1));
        };

        Ok(predicate_bool_expr(exact_numeric_value(expr).is_some()))
    }

    pub(in crate::runtime) fn eval_even_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("EvenQ", args, 1));
        };

        let is_even = match expr {
            Expr::Atom(Atom::Integer(value)) => value % BigInt::from(2_u8) == BigInt::from(0_u8),
            _ => false,
        };

        Ok(predicate_bool_expr(is_even))
    }

    pub(in crate::runtime) fn eval_odd_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("OddQ", args, 1));
        };

        let is_odd = match expr {
            Expr::Atom(Atom::Integer(value)) => value % BigInt::from(2_u8) != BigInt::from(0_u8),
            _ => false,
        };

        Ok(predicate_bool_expr(is_odd))
    }

    pub(in crate::runtime) fn eval_positive(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_exact_numeric_sign_predicate_or_inert("Positive", args, |value| {
            value.numerator() > &BigInt::from(0_u8)
        })
    }

    pub(in crate::runtime) fn eval_negative(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_exact_numeric_sign_predicate_or_inert("Negative", args, |value| {
            value.numerator() < &BigInt::from(0_u8)
        })
    }

    pub(in crate::runtime) fn eval_non_positive(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_exact_numeric_sign_predicate_or_inert("NonPositive", args, |value| {
            value.numerator() <= &BigInt::from(0_u8)
        })
    }

    pub(in crate::runtime) fn eval_non_negative(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_exact_numeric_sign_predicate_or_inert("NonNegative", args, |value| {
            value.numerator() >= &BigInt::from(0_u8)
        })
    }

    pub(in crate::runtime) fn eval_possible_zero_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_exact_numeric_sign_predicate("PossibleZeroQ", args, |value| value.is_zero())
    }

    pub(in crate::runtime) fn eval_machine_number_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [_expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("MachineNumberQ", args, 1));
        };

        Ok(predicate_bool_expr(false))
    }

    pub(in crate::runtime) fn eval_real_valued_number_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("RealValuedNumberQ", args, 1));
        };

        Ok(predicate_bool_expr(exact_numeric_value(expr).is_some()))
    }

    pub(in crate::runtime) fn eval_real_valued_numeric_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("RealValuedNumericQ", args, 1));
        };

        Ok(predicate_bool_expr(exact_numeric_value(expr).is_some()))
    }

    pub(in crate::runtime) fn eval_prime_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("PrimeQ", args, 1));
        };

        Ok(predicate_bool_expr(
            exact_integer_value(expr).is_some_and(|value| is_prime_bigint(&value)),
        ))
    }

    pub(in crate::runtime) fn eval_algebraic_integer_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("AlgebraicIntegerQ", args, 1));
        };

        Ok(predicate_bool_expr(exact_integer_value(expr).is_some()))
    }

    pub(in crate::runtime) fn eval_coprime_q(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let Some(values) = args
            .iter()
            .map(exact_integer_value)
            .collect::<Option<Vec<_>>>()
        else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let one = BigInt::from(1_u8);
        for (index, lhs) in values.iter().enumerate() {
            for rhs in &values[index + 1..] {
                if bigint_gcd(bigint_abs(lhs), bigint_abs(rhs)) != one {
                    return Ok(predicate_bool_expr(false));
                }
            }
        }

        Ok(predicate_bool_expr(true))
    }

    fn eval_exact_numeric_sign_predicate<F>(
        &mut self,
        name: &str,
        args: &[Expr],
        predicate: F,
    ) -> Result<Expr, EvalError>
    where
        F: FnOnce(&Rational) -> bool,
    {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count(name, args, 1));
        };

        Ok(predicate_bool_expr(
            exact_numeric_value(expr).is_some_and(|value| predicate(&value)),
        ))
    }

    fn eval_exact_numeric_sign_predicate_or_inert<F>(
        &mut self,
        name: &str,
        args: &[Expr],
        predicate: F,
    ) -> Result<Expr, EvalError>
    where
        F: FnOnce(&Rational) -> bool,
    {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count(name, args, 1));
        };

        let Some(value) = exact_numeric_value(expr) else {
            return Ok(Expr::call(system_expr(name), args.to_vec()));
        };

        Ok(predicate_bool_expr(predicate(&value)))
    }
}

fn is_prime_bigint(value: &BigInt) -> bool {
    let zero = BigInt::from(0_u8);
    let one = BigInt::from(1_u8);
    let two = BigInt::from(2_u8);

    if value <= &one {
        return false;
    }

    if value == &two {
        return true;
    }

    if value < &zero || value % &two == zero {
        return false;
    }

    let mut divisor = BigInt::from(3_u8);
    while &divisor * &divisor <= *value {
        if value % &divisor == zero {
            return false;
        }

        divisor += &two;
    }

    true
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
        let remainder = lhs % rhs.clone();
        lhs = rhs;
        rhs = remainder;
    }

    lhs
}
