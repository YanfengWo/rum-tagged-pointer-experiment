use std::str::FromStr;

use rum_core::{BigInt, Expr, Rational};

use super::TomlParseError;

pub(super) fn parse_integer_token(token: &str, offset: usize) -> Result<Expr, TomlParseError> {
    let (sign, rest, rest_offset) = split_sign(token, offset);
    let (radix, digits, digits_offset) =
        if let Some(digits) = rest.strip_prefix("0x").or_else(|| rest.strip_prefix("0X")) {
            (16, digits, rest_offset + 2)
        } else if let Some(digits) = rest.strip_prefix("0o").or_else(|| rest.strip_prefix("0O")) {
            (8, digits, rest_offset + 2)
        } else if let Some(digits) = rest.strip_prefix("0b").or_else(|| rest.strip_prefix("0B")) {
            (2, digits, rest_offset + 2)
        } else {
            (10, rest, rest_offset)
        };

    let cleaned = clean_digit_run(digits, digits_offset, radix, "integer")?;
    if radix == 10 && cleaned.len() > 1 && cleaned.starts_with('0') {
        return Err(TomlParseError::new(
            digits_offset,
            "leading zeros are not valid in decimal integers",
        ));
    }
    let mut value = BigInt::parse_bytes(cleaned.as_bytes(), radix)
        .ok_or_else(|| TomlParseError::new(offset, "invalid integer literal"))?;
    if sign < 0 {
        value = -value;
    }
    Ok(Expr::integer(value))
}

pub(super) fn parse_float_token(token: &str, offset: usize) -> Result<Expr, TomlParseError> {
    let (sign, rest, rest_offset) = split_sign(token, offset);
    let exponent_split = rest.find(['e', 'E']);
    let (mantissa, exponent_raw, exponent_offset) = match exponent_split {
        Some(index) => (
            &rest[..index],
            Some(&rest[index + 1..]),
            Some(rest_offset + index + 1),
        ),
        None => (rest, None, None),
    };

    let decimal_split = mantissa.find('.');
    let (integer_raw, fraction_raw, fraction_offset) = match decimal_split {
        Some(index) => (
            &mantissa[..index],
            Some(&mantissa[index + 1..]),
            rest_offset + index + 1,
        ),
        None => (mantissa, None, rest_offset + mantissa.len()),
    };

    if fraction_raw.is_none() && exponent_raw.is_none() {
        return Err(TomlParseError::new(offset, "expected float literal"));
    }

    let integer_digits = clean_digit_run(integer_raw, rest_offset, 10, "float integer part")?;
    if integer_digits.len() > 1 && integer_digits.starts_with('0') {
        return Err(TomlParseError::new(
            rest_offset,
            "leading zeros are not valid in TOML floats",
        ));
    }

    let fraction_digits = match fraction_raw {
        Some(raw) => clean_digit_run(raw, fraction_offset, 10, "float fraction")?,
        None => String::new(),
    };

    let exponent = match exponent_raw {
        Some(raw) => {
            if raw.is_empty() {
                return Err(TomlParseError::new(
                    exponent_offset.expect("offset exists with exponent"),
                    "expected exponent digits",
                ));
            }
            let exponent_offset = exponent_offset.expect("offset exists with exponent");
            let (exponent_sign, exponent_digits_raw, exponent_digits_offset) =
                split_sign(raw, exponent_offset);
            let exponent_digits =
                clean_digit_run(exponent_digits_raw, exponent_digits_offset, 10, "exponent")?;
            let parsed = exponent_digits.parse::<i64>().map_err(|_| {
                TomlParseError::new(exponent_digits_offset, "number exponent is too large")
            })?;
            exponent_sign * parsed
        }
        None => 0,
    };

    let mut digits = integer_digits;
    digits.push_str(&fraction_digits);
    let mut numerator = BigInt::from_str(&digits)
        .map_err(|_| TomlParseError::new(offset, "invalid decimal literal"))?;
    if sign < 0 {
        numerator = -numerator;
    }

    let scale = exponent as i128 - fraction_digits.len() as i128;
    if scale >= 0 {
        let scale = usize::try_from(scale)
            .map_err(|_| TomlParseError::new(offset, "number exponent is too large"))?;
        numerator *= pow10(scale);
        return Ok(Expr::integer(numerator));
    }

    let denominator_scale = usize::try_from(-scale)
        .map_err(|_| TomlParseError::new(offset, "number exponent is too large"))?;
    let denominator = pow10(denominator_scale);
    let rational =
        Rational::new(numerator, denominator).expect("decimal denominator should be non-zero");
    Ok(Expr::rational(rational))
}

fn split_sign(token: &str, offset: usize) -> (i64, &str, usize) {
    if let Some(rest) = token.strip_prefix('-') {
        (-1, rest, offset + 1)
    } else if let Some(rest) = token.strip_prefix('+') {
        (1, rest, offset + 1)
    } else {
        (1, token, offset)
    }
}

fn clean_digit_run(
    raw: &str,
    offset: usize,
    radix: u32,
    name: &str,
) -> Result<String, TomlParseError> {
    if raw.is_empty() {
        return Err(TomlParseError::new(
            offset,
            format!("expected {name} digits"),
        ));
    }

    let mut cleaned = String::new();
    let mut previous_underscore = false;
    for (relative, byte) in raw.bytes().enumerate() {
        match byte {
            b'_' => {
                if relative == 0 || previous_underscore {
                    return Err(TomlParseError::new(
                        offset + relative,
                        "underscores must separate digits",
                    ));
                }
                previous_underscore = true;
            }
            _ if digit_value(byte).is_some_and(|value| value < radix) => {
                cleaned.push(byte as char);
                previous_underscore = false;
            }
            _ => {
                return Err(TomlParseError::new(
                    offset + relative,
                    format!("invalid {name} digit"),
                ));
            }
        }
    }
    if previous_underscore {
        return Err(TomlParseError::new(
            offset + raw.len() - 1,
            "underscores must separate digits",
        ));
    }
    Ok(cleaned)
}

pub(super) fn pow10(exponent: usize) -> BigInt {
    let mut value = BigInt::from(1_u8);
    for _ in 0..exponent {
        value *= 10_u8;
    }
    value
}

fn digit_value(byte: u8) -> Option<u32> {
    match byte {
        b'0'..=b'9' => Some(u32::from(byte - b'0')),
        b'a'..=b'f' => Some(u32::from(byte - b'a' + 10)),
        b'A'..=b'F' => Some(u32::from(byte - b'A' + 10)),
        _ => None,
    }
}
