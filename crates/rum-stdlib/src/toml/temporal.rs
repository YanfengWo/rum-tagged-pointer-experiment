use rum_core::{BigInt, Rational};

use super::TomlParseError;

pub(super) const SECONDS_PER_MINUTE: i64 = 60;
pub(super) const SECONDS_PER_HOUR: i64 = 60 * SECONDS_PER_MINUTE;
const SECONDS_PER_DAY: i64 = 24 * SECONDS_PER_HOUR;

pub(super) fn timestamp_from_parts(
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: &Rational,
) -> Rational {
    let days = days_from_civil(year, month, day);
    let whole_seconds = days * BigInt::from(SECONDS_PER_DAY)
        + BigInt::from(hour * SECONDS_PER_HOUR + minute * SECONDS_PER_MINUTE);
    Rational::from_integer(whole_seconds).add(second)
}

pub(super) fn validate_ymd(
    start: usize,
    year: i64,
    month: i64,
    day: i64,
) -> Result<(), TomlParseError> {
    validate_ymd_offsets(year, month, start + 5, day, start + 8)
}

pub(super) fn validate_ymd_offsets(
    year: i64,
    month: i64,
    month_offset: usize,
    day: i64,
    day_offset: usize,
) -> Result<(), TomlParseError> {
    if !(1..=12).contains(&month) {
        return Err(TomlParseError::new(month_offset, "month must be in 01..12"));
    }
    let max_day = days_in_month(year, month);
    if day < 1 || day > max_day {
        return Err(TomlParseError::new(
            day_offset,
            format!("day must be in 01..{max_day:02} for this month"),
        ));
    }
    Ok(())
}

fn days_from_civil(year: i64, month: i64, day: i64) -> BigInt {
    let adjusted_year = if month <= 2 { year - 1 } else { year };
    let era = if adjusted_year >= 0 {
        adjusted_year / 400
    } else {
        (adjusted_year - 399) / 400
    };
    let yoe = adjusted_year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * month_prime + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    BigInt::from(era) * BigInt::from(146_097_i64) + BigInt::from(doe) - BigInt::from(719_468_i64)
}

fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => unreachable!("month is validated before days_in_month"),
    }
}

fn is_leap_year(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

pub(super) fn looks_like_full_date(bytes: &[u8], pos: usize) -> bool {
    pos + 10 <= bytes.len()
        && bytes[pos..pos + 4].iter().all(u8::is_ascii_digit)
        && bytes[pos + 4] == b'-'
        && bytes[pos + 5..pos + 7].iter().all(u8::is_ascii_digit)
        && bytes[pos + 7] == b'-'
        && bytes[pos + 8..pos + 10].iter().all(u8::is_ascii_digit)
}

pub(super) fn looks_like_local_time(bytes: &[u8], pos: usize) -> bool {
    pos + 8 <= bytes.len()
        && bytes[pos..pos + 2].iter().all(u8::is_ascii_digit)
        && bytes[pos + 2] == b':'
        && bytes[pos + 3..pos + 5].iter().all(u8::is_ascii_digit)
        && bytes[pos + 5] == b':'
        && bytes[pos + 6..pos + 8].iter().all(u8::is_ascii_digit)
}
