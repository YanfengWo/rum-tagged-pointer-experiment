use std::str::FromStr;

use rum_core::{BigInt, Expr, Rational};
use rum_runtime::kernel_api::system_expr;

use super::TomlParseError;
use super::model::{
    TomlTable, TomlValue, add_array_table_path, bare_key_byte, define_table_path,
    insert_value_path, list_expr, table_at_path_mut, table_expr,
};
use super::number::{parse_float_token, parse_integer_token, pow10};
use super::temporal::{
    SECONDS_PER_HOUR, SECONDS_PER_MINUTE, looks_like_full_date, looks_like_local_time,
    timestamp_from_parts, validate_ymd, validate_ymd_offsets,
};

pub(super) fn from_toml(source: &str) -> Result<Expr, TomlParseError> {
    let table = Parser::new(source).parse()?;
    Ok(table_expr(table))
}

struct Parser<'a> {
    source: &'a str,
    bytes: &'a [u8],
    pos: usize,
    root: TomlTable,
    current_path: Vec<String>,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            pos: 0,
            root: TomlTable::root(),
            current_path: Vec::new(),
        }
    }

    fn parse(mut self) -> Result<TomlTable, TomlParseError> {
        self.skip_ws_comments();
        while !self.is_eof() {
            match self.peek() {
                Some(b'[') => self.parse_header()?,
                Some(_) => self.parse_key_value_line()?,
                None => break,
            }
            self.skip_ws_comments();
        }
        Ok(self.root)
    }

    fn parse_header(&mut self) -> Result<(), TomlParseError> {
        let offset = self.pos;
        self.expect_byte(b'[', "expected table header")?;
        let array = self.consume(b'[');
        self.skip_inline_ws();
        let path = self.parse_key_path()?;
        self.skip_inline_ws();

        if array {
            self.expect_byte(b']', "expected ']' to close array-of-tables header")?;
            self.expect_byte(b']', "expected second ']' to close array-of-tables header")?;
            add_array_table_path(&mut self.root, &path, offset)?;
        } else {
            self.expect_byte(b']', "expected ']' to close table header")?;
            define_table_path(&mut self.root, &path, offset)?;
        }

        self.finish_line()?;
        self.current_path = path;
        Ok(())
    }

    fn parse_key_value_line(&mut self) -> Result<(), TomlParseError> {
        let key_offset = self.pos;
        let path = self.parse_key_path()?;
        self.skip_inline_ws();
        self.expect_byte(b'=', "expected '=' after key")?;
        self.skip_inline_ws();
        let value = self.parse_value()?;
        self.finish_line()?;

        let table = table_at_path_mut(&mut self.root, &self.current_path, key_offset)?;
        insert_value_path(table, &path, value, key_offset)
    }

    fn parse_key_path(&mut self) -> Result<Vec<String>, TomlParseError> {
        let mut parts = vec![self.parse_key_segment()?];
        loop {
            self.skip_inline_ws();
            if !self.consume(b'.') {
                break;
            }
            self.skip_inline_ws();
            parts.push(self.parse_key_segment()?);
        }
        Ok(parts)
    }

    fn parse_key_segment(&mut self) -> Result<String, TomlParseError> {
        match self.peek() {
            Some(b'"') => self.parse_basic_string(false),
            Some(b'\'') => self.parse_literal_string(false),
            Some(byte) if bare_key_byte(byte) => {
                let start = self.pos;
                while self.peek().is_some_and(bare_key_byte) {
                    self.pos += 1;
                }
                Ok(self.source[start..self.pos].to_string())
            }
            _ => Err(TomlParseError::new(self.pos, "expected TOML key")),
        }
    }

    fn parse_value(&mut self) -> Result<TomlValue, TomlParseError> {
        self.skip_inline_ws();
        if let Some(value) = self.parse_temporal_value()? {
            return Ok(value);
        }

        match self.peek() {
            Some(b'"') => self
                .parse_basic_string(true)
                .map(|value| TomlValue::Scalar(Expr::string(value))),
            Some(b'\'') => self
                .parse_literal_string(true)
                .map(|value| TomlValue::Scalar(Expr::string(value))),
            Some(b'[') => self.parse_array(),
            Some(b'{') => self.parse_inline_table(),
            Some(b't') if self.starts_with("true") => {
                self.pos += 4;
                Ok(TomlValue::Scalar(system_expr("True")))
            }
            Some(b'f') if self.starts_with("false") => {
                self.pos += 5;
                Ok(TomlValue::Scalar(system_expr("False")))
            }
            Some(b'+' | b'-' | b'0'..=b'9') => self.parse_number_value(),
            Some(b'i' | b'n') => self.parse_number_value(),
            Some(_) => Err(TomlParseError::new(self.pos, "expected a TOML value")),
            None => Err(TomlParseError::new(self.pos, "expected a TOML value")),
        }
    }

    fn parse_array(&mut self) -> Result<TomlValue, TomlParseError> {
        self.expect_byte(b'[', "expected array start")?;
        self.skip_ws_comments();
        let mut values = Vec::new();

        if self.consume(b']') {
            return Ok(TomlValue::Array(values));
        }

        loop {
            values.push(self.parse_value()?);
            self.skip_ws_comments();
            if self.consume(b',') {
                self.skip_ws_comments();
                if self.consume(b']') {
                    break;
                }
                continue;
            }
            if self.consume(b']') {
                break;
            }
            return Err(TomlParseError::new(
                self.pos,
                "expected ',' or ']' after array element",
            ));
        }

        Ok(TomlValue::Array(values))
    }

    fn parse_inline_table(&mut self) -> Result<TomlValue, TomlParseError> {
        self.expect_byte(b'{', "expected inline table start")?;
        self.skip_ws_comments();
        let mut table = TomlTable::inline();

        if self.consume(b'}') {
            return Ok(TomlValue::Table(table));
        }

        loop {
            let key_offset = self.pos;
            let path = self.parse_key_path()?;
            self.skip_inline_ws();
            self.expect_byte(b'=', "expected '=' after inline table key")?;
            self.skip_inline_ws();
            let value = self.parse_value()?;
            insert_value_path(&mut table, &path, value, key_offset)?;
            self.skip_ws_comments();

            if self.consume(b',') {
                self.skip_ws_comments();
                if self.peek() == Some(b'}') {
                    return Err(TomlParseError::new(
                        self.pos,
                        "trailing comma in inline table",
                    ));
                }
                continue;
            }
            if self.consume(b'}') {
                break;
            }
            return Err(TomlParseError::new(
                self.pos,
                "expected ',' or '}' after inline table entry",
            ));
        }

        Ok(TomlValue::Table(table))
    }

    fn parse_basic_string(&mut self, allow_multiline: bool) -> Result<String, TomlParseError> {
        let multiline = self.starts_with("\"\"\"");
        if multiline {
            if !allow_multiline {
                return Err(TomlParseError::new(
                    self.pos,
                    "multiline strings are not valid keys",
                ));
            }
            self.pos += 3;
            self.consume_initial_multiline_newline();
        } else {
            self.expect_byte(b'"', "expected basic string start")?;
        }

        let mut out = String::new();
        loop {
            let Some(byte) = self.peek() else {
                return Err(TomlParseError::new(self.pos, "unterminated string"));
            };

            if multiline && self.starts_with("\"\"\"") {
                self.pos += 3;
                return Ok(out);
            }
            if !multiline && byte == b'"' {
                self.pos += 1;
                return Ok(out);
            }

            match byte {
                b'\\' => {
                    let escape_offset = self.pos;
                    self.pos += 1;
                    if multiline && self.consume_multiline_line_continuation() {
                        continue;
                    }
                    self.parse_basic_escape(escape_offset, &mut out)?;
                }
                b'\n' if multiline => {
                    self.pos += 1;
                    out.push('\n');
                }
                b'\r' if multiline && self.peek_next() == Some(b'\n') => {
                    self.pos += 2;
                    out.push('\n');
                }
                b'\n' | b'\r' => {
                    return Err(TomlParseError::new(
                        self.pos,
                        "newline in single-line basic string",
                    ));
                }
                _ => {
                    let ch = self.next_char();
                    if disallowed_string_control(ch) {
                        return Err(TomlParseError::new(
                            self.pos,
                            "unescaped control character in string",
                        ));
                    }
                    self.pos += ch.len_utf8();
                    out.push(ch);
                }
            }
        }
    }

    fn parse_literal_string(&mut self, allow_multiline: bool) -> Result<String, TomlParseError> {
        let multiline = self.starts_with("'''");
        if multiline {
            if !allow_multiline {
                return Err(TomlParseError::new(
                    self.pos,
                    "multiline strings are not valid keys",
                ));
            }
            self.pos += 3;
            self.consume_initial_multiline_newline();
        } else {
            self.expect_byte(b'\'', "expected literal string start")?;
        }

        let mut out = String::new();
        loop {
            let Some(byte) = self.peek() else {
                return Err(TomlParseError::new(self.pos, "unterminated literal string"));
            };

            if multiline && self.starts_with("'''") {
                self.pos += 3;
                return Ok(out);
            }
            if !multiline && byte == b'\'' {
                self.pos += 1;
                return Ok(out);
            }

            match byte {
                b'\n' if multiline => {
                    self.pos += 1;
                    out.push('\n');
                }
                b'\r' if multiline && self.peek_next() == Some(b'\n') => {
                    self.pos += 2;
                    out.push('\n');
                }
                b'\n' | b'\r' => {
                    return Err(TomlParseError::new(
                        self.pos,
                        "newline in single-line literal string",
                    ));
                }
                _ => {
                    let ch = self.next_char();
                    if disallowed_string_control(ch) {
                        return Err(TomlParseError::new(
                            self.pos,
                            "control character in literal string",
                        ));
                    }
                    self.pos += ch.len_utf8();
                    out.push(ch);
                }
            }
        }
    }

    fn parse_basic_escape(
        &mut self,
        escape_offset: usize,
        out: &mut String,
    ) -> Result<(), TomlParseError> {
        match self.bump() {
            Some(b'b') => out.push('\u{0008}'),
            Some(b't') => out.push('\t'),
            Some(b'n') => out.push('\n'),
            Some(b'f') => out.push('\u{000c}'),
            Some(b'r') => out.push('\r'),
            Some(b'"') => out.push('"'),
            Some(b'\\') => out.push('\\'),
            Some(b'u') => out.push(self.parse_unicode_escape(escape_offset, 4)?),
            Some(b'U') => out.push(self.parse_unicode_escape(escape_offset, 8)?),
            Some(_) => return Err(TomlParseError::new(escape_offset, "invalid string escape")),
            None => {
                return Err(TomlParseError::new(
                    escape_offset,
                    "unterminated string escape",
                ));
            }
        }
        Ok(())
    }

    fn parse_unicode_escape(
        &mut self,
        escape_offset: usize,
        digits: usize,
    ) -> Result<char, TomlParseError> {
        if self.pos + digits > self.bytes.len() {
            return Err(TomlParseError::new(
                escape_offset,
                "Unicode escape is too short",
            ));
        }

        let mut value = 0_u32;
        for _ in 0..digits {
            let byte = self.bump().expect("length checked above");
            let Some(digit) = hex_value(byte) else {
                return Err(TomlParseError::new(
                    self.pos.saturating_sub(1),
                    "Unicode escape must contain hexadecimal digits",
                ));
            };
            value = value * 16 + u32::from(digit);
        }

        char::from_u32(value).ok_or_else(|| {
            TomlParseError::new(escape_offset, "Unicode escape is not a scalar value")
        })
    }

    fn parse_number_value(&mut self) -> Result<TomlValue, TomlParseError> {
        let start = self.pos;
        let end = self.bare_value_end();
        if start == end {
            return Err(TomlParseError::new(start, "expected a TOML value"));
        }
        let token = &self.source[start..end];
        self.pos = end;

        if matches!(token, "inf" | "+inf" | "-inf" | "nan" | "+nan" | "-nan") {
            return Err(TomlParseError::new(
                start,
                "inf and nan are outside rum's exact TOML number subset",
            ));
        }

        if token.contains(['.', 'e', 'E']) {
            parse_float_token(token, start).map(|expr| TomlValue::Scalar(expr))
        } else {
            parse_integer_token(token, start).map(|expr| TomlValue::Scalar(expr))
        }
    }

    fn parse_temporal_value(&mut self) -> Result<Option<TomlValue>, TomlParseError> {
        if looks_like_full_date(self.bytes, self.pos) {
            return self.parse_date_or_datetime().map(Some);
        }
        if looks_like_local_time(self.bytes, self.pos) {
            return self.parse_local_time_value().map(Some);
        }
        Ok(None)
    }

    fn parse_date_or_datetime(&mut self) -> Result<TomlValue, TomlParseError> {
        let start = self.pos;
        let (year, month, day) = self.parse_full_date()?;
        validate_ymd(start, year, month, day)?;

        let has_time = match self.peek() {
            Some(b'T' | b't') => {
                self.pos += 1;
                true
            }
            Some(b' ') if looks_like_local_time(self.bytes, self.pos + 1) => {
                self.pos += 1;
                true
            }
            _ => false,
        };

        if !has_time {
            return Ok(TomlValue::Scalar(list_expr(vec![
                Expr::small_integer(year),
                Expr::small_integer(month),
                Expr::small_integer(day),
            ])));
        }

        let (hour, minute, second) = self.parse_local_time_parts()?;
        if let Some(offset_seconds) = self.parse_offset_seconds()? {
            let timestamp = timestamp_from_parts(year, month, day, hour, minute, &second)
                .sub(&Rational::from_integer(BigInt::from(offset_seconds)));
            return Ok(TomlValue::Scalar(Expr::rational(timestamp)));
        }

        Ok(TomlValue::Scalar(list_expr(vec![
            Expr::small_integer(year),
            Expr::small_integer(month),
            Expr::small_integer(day),
            Expr::small_integer(hour),
            Expr::small_integer(minute),
            Expr::rational(second),
        ])))
    }

    fn parse_local_time_value(&mut self) -> Result<TomlValue, TomlParseError> {
        let (hour, minute, second) = self.parse_local_time_parts()?;
        Ok(TomlValue::Scalar(list_expr(vec![
            Expr::small_integer(hour),
            Expr::small_integer(minute),
            Expr::rational(second),
        ])))
    }

    fn parse_full_date(&mut self) -> Result<(i64, i64, i64), TomlParseError> {
        let year_offset = self.pos;
        let year = self.parse_fixed_digits(4, "year")?;
        self.expect_byte(b'-', "expected '-' after year")?;
        let month_offset = self.pos;
        let month = self.parse_fixed_digits(2, "month")?;
        self.expect_byte(b'-', "expected '-' after month")?;
        let day_offset = self.pos;
        let day = self.parse_fixed_digits(2, "day")?;
        validate_ymd_offsets(year, month, month_offset, day, day_offset)?;
        if year < 0 {
            return Err(TomlParseError::new(year_offset, "year is out of range"));
        }
        Ok((year, month, day))
    }

    fn parse_local_time_parts(&mut self) -> Result<(i64, i64, Rational), TomlParseError> {
        let hour_offset = self.pos;
        let hour = self.parse_fixed_digits(2, "hour")?;
        if !(0..=23).contains(&hour) {
            return Err(TomlParseError::new(hour_offset, "hour must be in 00..23"));
        }
        self.expect_byte(b':', "expected ':' after hour")?;
        let minute_offset = self.pos;
        let minute = self.parse_fixed_digits(2, "minute")?;
        if !(0..=59).contains(&minute) {
            return Err(TomlParseError::new(
                minute_offset,
                "minute must be in 00..59",
            ));
        }
        self.expect_byte(b':', "expected ':' after minute")?;
        let second_offset = self.pos;
        let second_whole = self.parse_fixed_digits(2, "second")?;
        if !(0..=59).contains(&second_whole) {
            return Err(TomlParseError::new(
                second_offset,
                "second must be in 00..59",
            ));
        }

        let mut second = Rational::from_integer(BigInt::from(second_whole));
        if self.consume(b'.') {
            let fraction_offset = self.pos;
            while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
                self.pos += 1;
            }
            if self.pos == fraction_offset {
                return Err(TomlParseError::new(
                    fraction_offset,
                    "expected fractional second digits",
                ));
            }
            let digits = &self.source[fraction_offset..self.pos];
            let numerator =
                BigInt::from_str(digits).expect("ASCII fractional second digits should parse");
            let denominator = pow10(digits.len());
            second = second.add(
                &Rational::new(numerator, denominator)
                    .expect("decimal fractional denominator should be non-zero"),
            );
        }

        Ok((hour, minute, second))
    }

    fn parse_offset_seconds(&mut self) -> Result<Option<i64>, TomlParseError> {
        if self.consume(b'Z') {
            return Ok(Some(0));
        }

        let sign = match self.peek() {
            Some(b'+') => 1,
            Some(b'-') => -1,
            _ => return Ok(None),
        };
        self.pos += 1;

        let hour_offset = self.pos;
        let hour = self.parse_fixed_digits(2, "offset hour")?;
        if !(0..=23).contains(&hour) {
            return Err(TomlParseError::new(
                hour_offset,
                "offset hour must be in 00..23",
            ));
        }
        self.expect_byte(b':', "expected ':' in fixed offset")?;
        let minute_offset = self.pos;
        let minute = self.parse_fixed_digits(2, "offset minute")?;
        if !(0..=59).contains(&minute) {
            return Err(TomlParseError::new(
                minute_offset,
                "offset minute must be in 00..59",
            ));
        }
        Ok(Some(
            sign * (hour * SECONDS_PER_HOUR + minute * SECONDS_PER_MINUTE),
        ))
    }

    fn parse_fixed_digits(&mut self, count: usize, name: &str) -> Result<i64, TomlParseError> {
        let offset = self.pos;
        if self.pos + count > self.bytes.len() {
            return Err(TomlParseError::new(
                offset,
                format!("expected {count} digit {name}"),
            ));
        }
        for index in self.pos..self.pos + count {
            if !self.bytes[index].is_ascii_digit() {
                return Err(TomlParseError::new(
                    index,
                    format!("expected {count} digit {name}"),
                ));
            }
        }
        self.pos += count;
        self.source[offset..offset + count]
            .parse::<i64>()
            .map_err(|_| TomlParseError::new(offset, format!("{name} is out of range")))
    }

    fn consume_initial_multiline_newline(&mut self) {
        if self.peek() == Some(b'\n') {
            self.pos += 1;
        } else if self.peek() == Some(b'\r') && self.peek_next() == Some(b'\n') {
            self.pos += 2;
        }
    }

    fn consume_multiline_line_continuation(&mut self) -> bool {
        let mut cursor = self.pos;
        while matches!(self.bytes.get(cursor), Some(b' ' | b'\t')) {
            cursor += 1;
        }
        match self.bytes.get(cursor).copied() {
            Some(b'\n') => cursor += 1,
            Some(b'\r') if self.bytes.get(cursor + 1) == Some(&b'\n') => cursor += 2,
            _ => return false,
        }
        while matches!(self.bytes.get(cursor), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            if self.bytes[cursor] == b'\r' && self.bytes.get(cursor + 1) == Some(&b'\n') {
                cursor += 2;
            } else {
                cursor += 1;
            }
        }
        self.pos = cursor;
        true
    }

    fn finish_line(&mut self) -> Result<(), TomlParseError> {
        self.skip_inline_ws();
        if self.consume(b'#') {
            self.skip_comment();
        }
        match self.peek() {
            Some(b'\n') => {
                self.pos += 1;
                Ok(())
            }
            Some(b'\r') if self.peek_next() == Some(b'\n') => {
                self.pos += 2;
                Ok(())
            }
            Some(_) => Err(TomlParseError::new(
                self.pos,
                "expected end of line after TOML statement",
            )),
            None => Ok(()),
        }
    }

    fn skip_ws_comments(&mut self) {
        loop {
            while matches!(self.peek(), Some(b' ' | b'\t' | b'\n')) {
                self.pos += 1;
            }
            if self.peek() == Some(b'\r') && self.peek_next() == Some(b'\n') {
                self.pos += 2;
                continue;
            }
            if self.consume(b'#') {
                self.skip_comment();
                continue;
            }
            break;
        }
    }

    fn skip_inline_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t')) {
            self.pos += 1;
        }
    }

    fn skip_comment(&mut self) {
        while !self.is_eof() && !matches!(self.peek(), Some(b'\n' | b'\r')) {
            self.pos += 1;
        }
    }

    fn bare_value_end(&self) -> usize {
        let mut end = self.pos;
        while end < self.bytes.len()
            && !matches!(
                self.bytes[end],
                b' ' | b'\t' | b'\n' | b'\r' | b',' | b']' | b'}' | b'#'
            )
        {
            end += 1;
        }
        end
    }

    fn starts_with(&self, value: &str) -> bool {
        self.source[self.pos..].starts_with(value)
    }

    fn expect_byte(
        &mut self,
        expected: u8,
        message: impl Into<String>,
    ) -> Result<(), TomlParseError> {
        match self.bump() {
            Some(byte) if byte == expected => Ok(()),
            Some(_) => Err(TomlParseError::new(self.pos.saturating_sub(1), message)),
            None => Err(TomlParseError::new(self.pos, message)),
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

    fn bump(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.pos += 1;
        Some(byte)
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn peek_next(&self) -> Option<u8> {
        self.bytes.get(self.pos + 1).copied()
    }

    fn next_char(&self) -> char {
        self.source[self.pos..]
            .chars()
            .next()
            .expect("valid UTF-8 input should yield a char before EOF")
    }

    fn is_eof(&self) -> bool {
        self.pos >= self.bytes.len()
    }
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn disallowed_string_control(ch: char) -> bool {
    (ch <= '\u{001f}' && ch != '\t') || ch == '\u{007f}'
}
