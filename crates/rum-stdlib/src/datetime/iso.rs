use std::str::FromStr;

use rum_core::{BigInt, Rational};
use rum_runtime::kernel_api::ToPrimitive;

use super::calendar::{
    DateTimeError, DateTimeParts, SECONDS_PER_HOUR, SECONDS_PER_MINUTE, days_in_month,
    timestamp_from_parts,
};

pub(super) fn timestamp_from_date_string(source: &str) -> Result<Rational, DateTimeError> {
    IsoParser::new(source).parse()
}

fn pow10(digits: usize) -> BigInt {
    let mut value = BigInt::from(1_i64);
    for _ in 0..digits {
        value *= 10_i64;
    }
    value
}

struct IsoParser<'a> {
    source: &'a str,
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> IsoParser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            pos: 0,
        }
    }

    fn parse(&mut self) -> Result<Rational, DateTimeError> {
        let year = self.parse_year()?;
        self.expect(b'-', "expected '-' after year")?;
        let month_offset = self.pos;
        let month = self.parse_two_digits("month")?;
        self.expect(b'-', "expected '-' after month")?;
        let day_offset = self.pos;
        let day = self.parse_two_digits("day")?;
        self.validate_ymd(&year, month, month_offset, day, day_offset)?;

        if self.is_eof() {
            return Ok(timestamp_from_parts(&DateTimeParts {
                year,
                month,
                day,
                hour: 0,
                minute: 0,
                second: Rational::zero(),
            }));
        }

        self.expect(b'T', "expected 'T' before time")?;
        let hour_offset = self.pos;
        let hour = self.parse_two_digits("hour")?;
        if !(0..=23).contains(&hour) {
            return Err(DateTimeError::parse(hour_offset, "hour must be in 00..23"));
        }
        self.expect(b':', "expected ':' after hour")?;
        let minute_offset = self.pos;
        let minute = self.parse_two_digits("minute")?;
        if !(0..=59).contains(&minute) {
            return Err(DateTimeError::parse(
                minute_offset,
                "minute must be in 00..59",
            ));
        }
        self.expect(b':', "expected ':' after minute")?;
        let second_offset = self.pos;
        let second_whole = self.parse_two_digits("second")?;
        if !(0..=59).contains(&second_whole) {
            return Err(DateTimeError::parse(
                second_offset,
                "second must be in 00..59",
            ));
        }
        let second = self.parse_fractional_second(second_whole)?;
        let offset_seconds = self.parse_offset_seconds()?;
        if !self.is_eof() {
            return Err(DateTimeError::parse(self.pos, "trailing characters"));
        }

        let local = timestamp_from_parts(&DateTimeParts {
            year,
            month,
            day,
            hour,
            minute,
            second,
        });
        Ok(local.sub(&Rational::from_integer(BigInt::from(offset_seconds))))
    }

    fn validate_ymd(
        &self,
        year: &BigInt,
        month: i64,
        month_offset: usize,
        day: i64,
        day_offset: usize,
    ) -> Result<(), DateTimeError> {
        if !(1..=12).contains(&month) {
            return Err(DateTimeError::parse(
                month_offset,
                "month must be in 01..12",
            ));
        }
        let max_day = days_in_month(year, month);
        if day < 1 || day > max_day {
            return Err(DateTimeError::parse(
                day_offset,
                format!("day must be in 01..{max_day:02} for this month"),
            ));
        }
        Ok(())
    }

    fn parse_year(&mut self) -> Result<BigInt, DateTimeError> {
        let negative = self.consume(b'-');
        let offset = self.pos;
        self.parse_fixed_digits(4, "year")?;
        let year = BigInt::from_str(&self.source[offset..offset + 4])
            .expect("fixed ASCII digits should parse as BigInt");
        Ok(if negative { -year } else { year })
    }

    fn parse_two_digits(&mut self, name: &str) -> Result<i64, DateTimeError> {
        let offset = self.pos;
        self.parse_fixed_digits(2, name)?
            .to_i64()
            .ok_or_else(|| DateTimeError::parse(offset, format!("{name} is out of range")))
    }

    fn parse_fixed_digits(&mut self, count: usize, name: &str) -> Result<BigInt, DateTimeError> {
        let offset = self.pos;
        if self.pos + count > self.bytes.len() {
            return Err(DateTimeError::parse(
                offset,
                format!("expected {count} digit {name}"),
            ));
        }
        for index in self.pos..self.pos + count {
            if !self.bytes[index].is_ascii_digit() {
                return Err(DateTimeError::parse(
                    index,
                    format!("expected {count} digit {name}"),
                ));
            }
        }
        self.pos += count;
        BigInt::from_str(&self.source[offset..offset + count])
            .map_err(|_| DateTimeError::parse(offset, format!("{name} is out of range")))
    }

    fn parse_fractional_second(&mut self, second_whole: i64) -> Result<Rational, DateTimeError> {
        let mut second = Rational::from_integer(BigInt::from(second_whole));
        if !self.consume(b'.') {
            return Ok(second);
        }
        let offset = self.pos;
        while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
            self.pos += 1;
        }
        if self.pos == offset {
            return Err(DateTimeError::parse(
                offset,
                "expected fractional second digits",
            ));
        }
        let digits = &self.source[offset..self.pos];
        let numerator =
            BigInt::from_str(digits).expect("ASCII fractional second digits should parse");
        let denominator = pow10(digits.len());
        second = second.add(
            &Rational::new(numerator, denominator)
                .expect("decimal fractional denominator should be non-zero"),
        );
        Ok(second)
    }

    fn parse_offset_seconds(&mut self) -> Result<i64, DateTimeError> {
        if self.is_eof() {
            return Ok(0);
        }
        if self.consume(b'Z') {
            return Ok(0);
        }

        let sign = match self.peek() {
            Some(b'+') => 1,
            Some(b'-') => -1,
            _ => {
                return Err(DateTimeError::parse(
                    self.pos,
                    "expected 'Z' or fixed offset '+HH:MM'",
                ));
            }
        };
        self.pos += 1;
        let hour_offset = self.pos;
        let hour = self.parse_two_digits("offset hour")?;
        if !(0..=23).contains(&hour) {
            return Err(DateTimeError::parse(
                hour_offset,
                "offset hour must be in 00..23",
            ));
        }
        self.expect(b':', "expected ':' in fixed offset")?;
        let minute_offset = self.pos;
        let minute = self.parse_two_digits("offset minute")?;
        if !(0..=59).contains(&minute) {
            return Err(DateTimeError::parse(
                minute_offset,
                "offset minute must be in 00..59",
            ));
        }
        let seconds = hour * SECONDS_PER_HOUR + minute * SECONDS_PER_MINUTE;
        Ok(sign * seconds)
    }

    fn expect(&mut self, byte: u8, message: &str) -> Result<(), DateTimeError> {
        if self.consume(byte) {
            Ok(())
        } else {
            Err(DateTimeError::parse(self.pos, message))
        }
    }

    fn consume(&mut self, byte: u8) -> bool {
        if self.peek() == Some(byte) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.bytes.len()
    }
}
