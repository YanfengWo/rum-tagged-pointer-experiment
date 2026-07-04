use super::super::*;
use super::exact::*;
use super::integer_ops::{bigint_abs, floor_quotient};

impl Session {
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

    pub(super) fn eval_sign(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [arg] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let Some(value) = expr_as_exact_rational(arg) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let sign = match value.numerator().cmp(&BigInt::from(0_u8)) {
            std::cmp::Ordering::Less => -1_i32,
            std::cmp::Ordering::Equal => 0_i32,
            std::cmp::Ordering::Greater => 1_i32,
        };
        Ok(Expr::integer(BigInt::from(sign)))
    }

    pub(super) fn eval_floor(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [arg] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let Some(value) = expr_as_exact_rational(arg) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::integer(floor_exact_rational_to_integer(&value)))
    }

    pub(super) fn eval_ceiling(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [arg] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let Some(value) = expr_as_exact_rational(arg) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::integer(ceiling_exact_rational_to_integer(&value)))
    }

    pub(super) fn eval_integer_part(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [arg] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let Some(value) = expr_as_exact_rational(arg) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::integer(truncate_exact_rational_to_integer(&value)))
    }

    pub(super) fn eval_fractional_part(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [arg] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let Some(value) = expr_as_exact_rational(arg) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let integer_part = Rational::from_integer(truncate_exact_rational_to_integer(&value));
        Ok(Expr::rational(value.sub(&integer_part)))
    }

    pub(super) fn eval_minus(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let [arg] = args else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        self.combine_multiplicative_args(
            &system_expr("Times"),
            &[Expr::integer((-1_i32).into()), arg.clone()],
        )
    }

    pub(super) fn eval_round(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        let (value_expr, multiple_expr) = match args {
            [value] => (value, None),
            [value, multiple] => (value, Some(multiple)),
            _ => return Ok(self.invalid_call_with_arg_range("Round", args, 1, 2)),
        };

        let Some(value) = expr_as_exact_rational(value_expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let multiple = match multiple_expr {
            None => Rational::from_integer(BigInt::from(1_u8)),
            Some(multiple_expr) => {
                let Some(multiple) = expr_as_exact_rational(multiple_expr) else {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                };
                if multiple.numerator() == &BigInt::from(0_u8) {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                }
                multiple
            }
        };

        let scaled = value
            .div(&multiple)
            .expect("zero multiple is rejected before exact division");
        let rounded = round_exact_rational_to_integer(&scaled);
        Ok(Expr::rational(
            Rational::from_integer(rounded).mul(&multiple),
        ))
    }
}

fn floor_exact_rational_to_integer(value: &Rational) -> BigInt {
    floor_quotient(value.numerator(), value.denominator())
        .expect("normalized exact rational has a non-zero denominator")
}

fn ceiling_exact_rational_to_integer(value: &Rational) -> BigInt {
    -floor_quotient(&(-value.numerator().clone()), value.denominator())
        .expect("normalized exact rational has a non-zero denominator")
}

fn truncate_exact_rational_to_integer(value: &Rational) -> BigInt {
    value.numerator() / value.denominator()
}
fn round_exact_rational_to_integer(value: &Rational) -> BigInt {
    let numerator = value.numerator();
    let denominator = value.denominator();
    let negative = numerator < &BigInt::from(0_u8);
    let abs_numerator = bigint_abs(numerator);
    let quotient = &abs_numerator / denominator;
    let remainder = &abs_numerator % denominator;
    let twice_remainder = remainder * BigInt::from(2_u8);

    let rounded_abs = if twice_remainder < *denominator {
        quotient
    } else if twice_remainder > *denominator {
        quotient + BigInt::from(1_u8)
    } else if &quotient % BigInt::from(2_u8) == BigInt::from(0_u8) {
        quotient
    } else {
        quotient + BigInt::from(1_u8)
    };

    if negative { -rounded_abs } else { rounded_abs }
}
