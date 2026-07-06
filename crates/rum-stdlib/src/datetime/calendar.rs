use rum_core::{BigInt, Expr, Rational};
use rum_runtime::kernel_api::{
    ToPrimitive, expr_as_exact_integer, expr_as_exact_rational, expr_as_string, list_items,
    system_expr,
};

pub(super) const SECONDS_PER_MINUTE: i64 = 60;
pub(super) const SECONDS_PER_HOUR: i64 = 60 * SECONDS_PER_MINUTE;
const SECONDS_PER_DAY: i64 = 24 * SECONDS_PER_HOUR;
const SECONDS_PER_WEEK: i64 = 7 * SECONDS_PER_DAY;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DateTimeError {
    tag: &'static str,
    message: String,
}

impl DateTimeError {
    pub(super) fn date(message: impl Into<String>) -> Self {
        Self {
            tag: "date",
            message: message.into(),
        }
    }

    pub(super) fn parse(offset: usize, message: impl Into<String>) -> Self {
        Self::date(format!(
            "Invalid ISO 8601 date/time at byte offset {offset}: {}.",
            message.into()
        ))
    }

    pub(super) fn tag(&self) -> &'static str {
        self.tag
    }

    pub(super) fn message(self) -> String {
        self.message
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct DateTimeParts {
    pub(super) year: BigInt,
    pub(super) month: i64,
    pub(super) day: i64,
    pub(super) hour: i64,
    pub(super) minute: i64,
    pub(super) second: Rational,
}

pub(super) fn timestamp_from_timestamp_or_date_list_expr(
    expr: &Expr,
) -> Result<Rational, DateTimeError> {
    if let Some(timestamp) = expr_as_exact_rational(expr) {
        return Ok(timestamp);
    }
    if list_items(expr).is_some() {
        return timestamp_from_date_list_expr(expr);
    }

    Err(DateTimeError::date(
        "Expected an exact timestamp or a date list {y, mo, d, h, mi, s}.",
    ))
}

pub(super) fn timestamp_from_date_list_expr(expr: &Expr) -> Result<Rational, DateTimeError> {
    let items = list_items(expr).ok_or_else(|| {
        DateTimeError::date("Expected a date list with head List and one to six fields.")
    })?;
    let parts = date_list_parts(items)?;
    Ok(timestamp_from_parts(&parts))
}

pub(super) fn date_list_expr_from_timestamp(timestamp: &Rational) -> Expr {
    let parts = parts_from_timestamp(timestamp);
    Expr::call(
        system_expr("List"),
        vec![
            Expr::integer(parts.year),
            Expr::small_integer(parts.month),
            Expr::small_integer(parts.day),
            Expr::small_integer(parts.hour),
            Expr::small_integer(parts.minute),
            Expr::rational(parts.second),
        ],
    )
}

pub(super) fn date_plus(date: &Expr, spec: &Expr) -> Result<Rational, DateTimeError> {
    let timestamp = timestamp_from_timestamp_or_date_list_expr(date)?;
    let items = list_items(spec)
        .ok_or_else(|| DateTimeError::date("DatePlus expects {n, unit} as its second argument."))?;
    let [amount, unit] = items else {
        return Err(DateTimeError::date(
            "DatePlus expects {n, unit} as its second argument.",
        ));
    };
    let amount = expr_as_exact_rational(amount)
        .ok_or_else(|| DateTimeError::date("DatePlus amount must be an exact number."))?;
    let unit = expr_as_string(unit)
        .ok_or_else(|| DateTimeError::date("DatePlus unit must be a string."))?;

    Ok(timestamp.add(&amount.mul(&Rational::from_integer(unit_seconds(unit)?))))
}

pub(super) fn date_difference(
    start: &Expr,
    end: &Expr,
    unit: &Expr,
) -> Result<Rational, DateTimeError> {
    let start = timestamp_from_timestamp_or_date_list_expr(start)?;
    let end = timestamp_from_timestamp_or_date_list_expr(end)?;
    let unit = expr_as_string(unit)
        .ok_or_else(|| DateTimeError::date("DateDifference unit must be a string."))?;
    end.sub(&start)
        .div(&Rational::from_integer(unit_seconds(unit)?))
        .ok_or_else(|| DateTimeError::date("DateDifference unit must not be zero."))
}

pub(super) fn day_of_week(date: &Expr) -> Result<&'static str, DateTimeError> {
    let timestamp = timestamp_from_timestamp_or_date_list_expr(date)?;
    let days = timestamp_day_floor(&timestamp);
    let shifted = days + BigInt::from(3_i64);
    let index = mod_floor_i64(&shifted, 7);
    Ok([
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
        "Sunday",
    ][index as usize])
}

pub(super) fn leap_year_from_expr(expr: &Expr) -> Result<bool, DateTimeError> {
    let year = expr_as_exact_integer(expr)
        .ok_or_else(|| DateTimeError::date("LeapYearQ expects an exact integer year."))?;
    Ok(is_leap_year(&year))
}

pub(super) fn days_in_month_from_exprs(year: &Expr, month: &Expr) -> Result<i64, DateTimeError> {
    let year = expr_as_exact_integer(year)
        .ok_or_else(|| DateTimeError::date("DaysInMonth expects an exact integer year."))?;
    let month = exact_i64_in_range(month, "month", 1, 12)?;
    Ok(days_in_month(&year, month))
}

fn date_list_parts(items: &[Expr]) -> Result<DateTimeParts, DateTimeError> {
    if !(1..=6).contains(&items.len()) {
        return Err(DateTimeError::date(
            "Date lists must have one to six fields: {y, mo, d, h, mi, s}.",
        ));
    }

    let year = expr_as_exact_integer(&items[0])
        .ok_or_else(|| DateTimeError::date("Date list year must be an exact integer."))?;
    let month = integer_field_or_default(items, 1, "month", 1, 1, 12)?;
    let day = integer_field_or_default(items, 2, "day", 1, 1, 31)?;
    let hour = integer_field_or_default(items, 3, "hour", 0, 0, 23)?;
    let minute = integer_field_or_default(items, 4, "minute", 0, 0, 59)?;
    let second = second_field_or_default(items)?;
    let parts = DateTimeParts {
        year,
        month,
        day,
        hour,
        minute,
        second,
    };
    validate_day_in_month(&parts.year, parts.month, parts.day)?;
    Ok(parts)
}

fn integer_field_or_default(
    items: &[Expr],
    index: usize,
    name: &str,
    default: i64,
    min: i64,
    max: i64,
) -> Result<i64, DateTimeError> {
    let Some(expr) = items.get(index) else {
        return Ok(default);
    };
    exact_i64_in_range(expr, name, min, max)
}

fn exact_i64_in_range(expr: &Expr, name: &str, min: i64, max: i64) -> Result<i64, DateTimeError> {
    let value = expr_as_exact_integer(expr).ok_or_else(|| {
        DateTimeError::date(format!("Date list {name} must be an exact integer."))
    })?;
    let min_big = BigInt::from(min);
    let max_big = BigInt::from(max);
    if value < min_big || value > max_big {
        return Err(DateTimeError::date(format!(
            "Date list {name} must be in the range {min}..{max}."
        )));
    }
    value
        .to_i64()
        .ok_or_else(|| DateTimeError::date(format!("Date list {name} is out of range.")))
}

fn second_field_or_default(items: &[Expr]) -> Result<Rational, DateTimeError> {
    let Some(expr) = items.get(5) else {
        return Ok(Rational::zero());
    };
    let second = expr_as_exact_rational(expr)
        .ok_or_else(|| DateTimeError::date("Date list second must be an exact number."))?;
    validate_second(&second)?;
    Ok(second)
}

fn validate_second(second: &Rational) -> Result<(), DateTimeError> {
    let zero = BigInt::from(0_i64);
    let upper = second.denominator() * BigInt::from(60_i64);
    if second.numerator() < &zero || second.numerator() >= &upper {
        return Err(DateTimeError::date(
            "Date list second must satisfy 0 <= s < 60.",
        ));
    }
    Ok(())
}

fn validate_day_in_month(year: &BigInt, month: i64, day: i64) -> Result<(), DateTimeError> {
    let max = days_in_month(year, month);
    if day > max {
        return Err(DateTimeError::date(format!(
            "Date list day {day} is invalid for month {month} in year {year}."
        )));
    }
    Ok(())
}

pub(super) fn timestamp_from_parts(parts: &DateTimeParts) -> Rational {
    let days = days_from_civil(&parts.year, parts.month, parts.day);
    let whole_seconds = days * BigInt::from(SECONDS_PER_DAY)
        + BigInt::from(parts.hour * SECONDS_PER_HOUR + parts.minute * SECONDS_PER_MINUTE);
    Rational::from_integer(whole_seconds).add(&parts.second)
}

pub(super) fn parts_from_timestamp(timestamp: &Rational) -> DateTimeParts {
    let days = timestamp_day_floor(timestamp);
    let day_denominator = timestamp.denominator() * BigInt::from(SECONDS_PER_DAY);
    let remainder_numerator =
        timestamp.numerator().clone() - days.clone() * day_denominator.clone();
    let seconds_of_day = Rational::new(remainder_numerator, timestamp.denominator().clone())
        .expect("timestamp denominator should be non-zero");
    let whole_seconds = seconds_of_day.numerator() / seconds_of_day.denominator();
    let hour = (&whole_seconds / BigInt::from(SECONDS_PER_HOUR))
        .to_i64()
        .expect("hour should fit in i64");
    let minute = ((&whole_seconds % BigInt::from(SECONDS_PER_HOUR))
        / BigInt::from(SECONDS_PER_MINUTE))
    .to_i64()
    .expect("minute should fit in i64");
    let before_minute = BigInt::from(hour * SECONDS_PER_HOUR + minute * SECONDS_PER_MINUTE);
    let second = Rational::new(
        seconds_of_day.numerator().clone() - before_minute * seconds_of_day.denominator(),
        seconds_of_day.denominator().clone(),
    )
    .expect("timestamp denominator should be non-zero");
    let (year, month, day) = civil_from_days(&days);

    DateTimeParts {
        year,
        month,
        day,
        hour,
        minute,
        second,
    }
}

fn timestamp_day_floor(timestamp: &Rational) -> BigInt {
    let denominator = timestamp.denominator() * BigInt::from(SECONDS_PER_DAY);
    div_floor_bigint(timestamp.numerator(), &denominator)
}

fn days_from_civil(year: &BigInt, month: i64, day: i64) -> BigInt {
    let adjusted_year = if month <= 2 {
        year - BigInt::from(1_i64)
    } else {
        year.clone()
    };
    let era = if adjusted_year >= BigInt::from(0_i64) {
        &adjusted_year / BigInt::from(400_i64)
    } else {
        (&adjusted_year - BigInt::from(399_i64)) / BigInt::from(400_i64)
    };
    let yoe = adjusted_year - &era * BigInt::from(400_i64);
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * month_prime + 2) / 5 + day - 1;
    let doe = &yoe * BigInt::from(365_i64) + &yoe / 4_i64 - &yoe / 100_i64 + BigInt::from(doy);
    era * BigInt::from(146_097_i64) + doe - BigInt::from(719_468_i64)
}

fn civil_from_days(days: &BigInt) -> (BigInt, i64, i64) {
    let z = days + BigInt::from(719_468_i64);
    let era = if z >= BigInt::from(0_i64) {
        &z / BigInt::from(146_097_i64)
    } else {
        (&z - BigInt::from(146_096_i64)) / BigInt::from(146_097_i64)
    };
    let doe = (z - &era * BigInt::from(146_097_i64))
        .to_i64()
        .expect("day-of-era should fit in i64");
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = &era * BigInt::from(400_i64) + BigInt::from(yoe);
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    if month <= 2 {
        year += BigInt::from(1_i64);
    }
    (year, month, day)
}

fn is_leap_year(year: &BigInt) -> bool {
    let zero = BigInt::from(0_i64);
    year % 4_i64 == zero && (year % 100_i64 != zero || year % 400_i64 == zero)
}

pub(super) fn days_in_month(year: &BigInt, month: i64) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => unreachable!("month is validated before days_in_month"),
    }
}

fn unit_seconds(unit: &str) -> Result<BigInt, DateTimeError> {
    match unit {
        "Seconds" => Ok(BigInt::from(1_i64)),
        "Minutes" => Ok(BigInt::from(SECONDS_PER_MINUTE)),
        "Hours" => Ok(BigInt::from(SECONDS_PER_HOUR)),
        "Days" => Ok(BigInt::from(SECONDS_PER_DAY)),
        "Weeks" => Ok(BigInt::from(SECONDS_PER_WEEK)),
        "Months" | "Years" => Err(DateTimeError::date(format!(
            "Calendar unit \"{unit}\" is outside rum's UTC fixed-duration date folk subset."
        ))),
        _ => Err(DateTimeError::date(format!(
            "Unsupported date unit \"{unit}\"; expected Seconds, Minutes, Hours, Days, or Weeks."
        ))),
    }
}

fn div_floor_bigint(numerator: &BigInt, denominator: &BigInt) -> BigInt {
    let quotient = numerator / denominator;
    let remainder = numerator % denominator;
    if remainder < BigInt::from(0_i64) {
        quotient - BigInt::from(1_i64)
    } else {
        quotient
    }
}

fn mod_floor_i64(value: &BigInt, modulus: i64) -> i64 {
    let modulus = BigInt::from(modulus);
    let mut remainder = value % &modulus;
    if remainder < BigInt::from(0_i64) {
        remainder += modulus;
    }
    remainder
        .to_i64()
        .expect("small positive modulo result should fit in i64")
}
