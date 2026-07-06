use std::str::FromStr;

use rum_core::{BigInt, Expr, Rational};

use super::YamlParseError;

pub(super) fn parse_yaml_number(raw: &str, offset: usize) -> Result<Option<Expr>, YamlParseError> {
    let bytes = raw.as_bytes();
    let mut pos = 0_usize;
    let negative = bytes.get(pos) == Some(&b'-');
    if negative {
        pos += 1;
    }

    let digits_start = pos;
    while matches!(bytes.get(pos), Some(b'0'..=b'9')) {
        pos += 1;
    }
    if pos == digits_start {
        return Ok(None);
    }

    let mut has_fraction = false;
    let mut fraction_len = 0_usize;
    if bytes.get(pos) == Some(&b'.') {
        has_fraction = true;
        pos += 1;
        let fraction_start = pos;
        while matches!(bytes.get(pos), Some(b'0'..=b'9')) {
            pos += 1;
            fraction_len += 1;
        }
        if pos == fraction_start {
            return Ok(None);
        }
    }

    let mut has_exponent = false;
    let mut exponent = 0_i64;
    if matches!(bytes.get(pos), Some(b'e' | b'E')) {
        has_exponent = true;
        pos += 1;
        let exponent_start = pos;
        if matches!(bytes.get(pos), Some(b'+' | b'-')) {
            pos += 1;
        }
        let exponent_digits = pos;
        while matches!(bytes.get(pos), Some(b'0'..=b'9')) {
            pos += 1;
        }
        if pos == exponent_digits {
            return Ok(None);
        }
        exponent = raw[exponent_start..pos].parse::<i64>().map_err(|_| {
            YamlParseError::new(offset + exponent_start, "number exponent is too large")
        })?;
    }

    if pos != bytes.len() {
        return Ok(None);
    }

    if !has_fraction && !has_exponent {
        let value = BigInt::from_str(raw)
            .map_err(|_| YamlParseError::new(offset, "invalid integer literal"))?;
        return Ok(Some(Expr::integer(value)));
    }

    let mut digits = String::new();
    for byte in raw.bytes() {
        if byte.is_ascii_digit() {
            digits.push(byte as char);
        } else if matches!(byte, b'e' | b'E') {
            break;
        }
    }

    let scale = exponent as i128 - fraction_len as i128;
    let mut numerator = BigInt::from_str(&digits)
        .map_err(|_| YamlParseError::new(offset, "invalid decimal literal"))?;
    if negative {
        numerator = -numerator;
    }

    if scale >= 0 {
        let scale = usize::try_from(scale)
            .map_err(|_| YamlParseError::new(offset, "number exponent is too large"))?;
        numerator *= pow10(scale);
        return Ok(Some(Expr::integer(numerator)));
    }

    let denominator_scale = usize::try_from(-scale)
        .map_err(|_| YamlParseError::new(offset, "number exponent is too large"))?;
    let denominator = pow10(denominator_scale);
    let rational =
        Rational::new(numerator, denominator).expect("decimal denominator should be non-zero");
    Ok(Some(Expr::rational(rational)))
}

pub(super) fn parses_as_yaml_number(raw: &str) -> bool {
    parse_yaml_number(raw, 0).is_ok_and(|value| value.is_some())
}

pub(super) fn pow10(exponent: usize) -> BigInt {
    let mut value = BigInt::from(1_u8);
    for _ in 0..exponent {
        value *= 10_u8;
    }
    value
}
