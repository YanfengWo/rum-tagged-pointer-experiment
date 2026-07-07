use super::super::*;
use super::exact::expr_as_exact_integer;

impl Session {
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

        if rhs == BigInt::from(0_u8) {
            return Ok(Expr::integer(lhs));
        }

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
}

pub(super) fn bigint_abs(value: &BigInt) -> BigInt {
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

pub(super) fn floor_quotient(lhs: &BigInt, rhs: &BigInt) -> Option<BigInt> {
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
