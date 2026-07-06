use rum_core::{BigInt, Rational};
use rum_runtime::kernel_api::ToPrimitive;

use super::calendar::{DateTimeError, parts_from_timestamp};

pub(super) fn date_string_from_timestamp(timestamp: &Rational) -> Result<String, DateTimeError> {
    let parts = parts_from_timestamp(timestamp);
    Ok(format!(
        "{}-{:02}-{:02}T{:02}:{:02}:{}Z",
        format_year(&parts.year)?,
        parts.month,
        parts.day,
        parts.hour,
        parts.minute,
        format_second(&parts.second)?,
    ))
}

fn format_year(year: &BigInt) -> Result<String, DateTimeError> {
    let Some(value) = year.to_i64() else {
        return Err(DateTimeError::date(
            "DateString cannot render years outside the supported ISO decimal range.",
        ));
    };
    if (0..=9999).contains(&value) {
        Ok(format!("{value:04}"))
    } else if (-9999..=-1).contains(&value) {
        Ok(format!("-{:04}", -value))
    } else {
        Ok(value.to_string())
    }
}

fn format_second(second: &Rational) -> Result<String, DateTimeError> {
    let scale_digits = terminating_decimal_scale(second.denominator()).ok_or_else(|| {
        DateTimeError::date(
            "DateString cannot render a non-terminating fractional second; convert it explicitly before calling DateString.",
        )
    })?;
    let scale = pow10(scale_digits);
    let scaled = second.numerator() * (&scale / second.denominator());
    let whole = &scaled / &scale;
    let fraction = &scaled % &scale;
    let whole = whole
        .to_i64()
        .ok_or_else(|| DateTimeError::date("DateString second is out of range."))?;
    if fraction == BigInt::from(0_i64) {
        return Ok(format!("{whole:02}"));
    }

    let mut fraction = fraction.to_string();
    if fraction.len() < scale_digits {
        fraction = format!("{}{}", "0".repeat(scale_digits - fraction.len()), fraction);
    }
    while fraction.ends_with('0') {
        fraction.pop();
    }
    Ok(format!("{whole:02}.{fraction}"))
}

fn terminating_decimal_scale(denominator: &BigInt) -> Option<usize> {
    let mut value = denominator.clone();
    let mut twos = 0_usize;
    let mut fives = 0_usize;
    let zero = BigInt::from(0_i64);
    let one = BigInt::from(1_i64);

    while &value % 2_i64 == zero {
        value /= 2_i64;
        twos += 1;
    }
    while &value % 5_i64 == zero {
        value /= 5_i64;
        fives += 1;
    }

    (value == one).then_some(twos.max(fives))
}

fn pow10(digits: usize) -> BigInt {
    let mut value = BigInt::from(1_i64);
    for _ in 0..digits {
        value *= 10_i64;
    }
    value
}
