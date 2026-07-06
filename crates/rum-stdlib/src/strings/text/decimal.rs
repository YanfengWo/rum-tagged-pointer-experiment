use super::*;

pub(crate) fn eval_decimal_string(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [value_expr, digits_expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("DecimalString", args, 2));
    };

    let (Some(value), Some(fraction_digits)) = (
        expr_as_exact_rational(value_expr),
        expr_as_non_negative_usize(digits_expr),
    ) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(Expr::string(decimal_string_value(&value, fraction_digits)))
}

fn decimal_string_value(value: &Rational, fraction_digits: usize) -> String {
    let zero = BigInt::from(0_u8);
    let negative = value.numerator() < &zero;
    let abs_numerator = if negative {
        -value.numerator().clone()
    } else {
        value.numerator().clone()
    };

    let scaled_numerator = abs_numerator * pow10(fraction_digits);
    let quotient = &scaled_numerator / value.denominator();
    let remainder = &scaled_numerator % value.denominator();
    let twice_remainder = remainder * BigInt::from(2_u8);

    let mut rounded_abs = quotient;
    if twice_remainder >= value.denominator().clone() {
        rounded_abs += BigInt::from(1_u8);
    }

    format_scaled_decimal(rounded_abs, fraction_digits, negative)
}

fn format_scaled_decimal(value: BigInt, fraction_digits: usize, negative: bool) -> String {
    let digits = value.to_string();
    let mut rendered = String::new();
    if negative {
        rendered.push('-');
    }

    if fraction_digits == 0 {
        rendered.push_str(&digits);
        return rendered;
    }

    if digits.len() <= fraction_digits {
        rendered.push_str("0.");
        rendered.extend(std::iter::repeat_n('0', fraction_digits - digits.len()));
        rendered.push_str(&digits);
    } else {
        let split = digits.len() - fraction_digits;
        rendered.push_str(&digits[..split]);
        rendered.push('.');
        rendered.push_str(&digits[split..]);
    }
    rendered
}

fn pow10(exponent: usize) -> BigInt {
    let mut value = BigInt::from(1_u8);
    for _ in 0..exponent {
        value *= BigInt::from(10_u8);
    }
    value
}
