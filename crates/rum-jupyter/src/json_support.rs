use std::{collections::BTreeMap, str};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum JsonValue {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Array(Vec<JsonValue>),
    Object(BTreeMap<String, JsonValue>),
}

impl JsonValue {
    pub(crate) fn kind_name(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Bool(_) => "bool",
            Self::Number(_) => "number",
            Self::String(_) => "string",
            Self::Array(_) => "array",
            Self::Object(_) => "object",
        }
    }
}

pub(crate) fn parse_json(input: &str) -> Result<JsonValue, String> {
    JsonParser::parse(input)
}

pub(crate) fn parse_json_object(input: &str) -> Result<BTreeMap<String, JsonValue>, String> {
    match parse_json(input)? {
        JsonValue::Object(object) => Ok(object),
        other => Err(format!(
            "expected top-level object, found {}",
            other.kind_name()
        )),
    }
}

struct JsonParser<'a> {
    input: &'a [u8],
    index: usize,
}

impl<'a> JsonParser<'a> {
    fn parse(input: &'a str) -> Result<JsonValue, String> {
        let mut parser = Self {
            input: input.as_bytes(),
            index: 0,
        };
        let value = parser.parse_value()?;
        parser.skip_whitespace();

        if parser.index != parser.input.len() {
            return Err("trailing characters after JSON value".to_string());
        }

        Ok(value)
    }

    fn parse_value(&mut self) -> Result<JsonValue, String> {
        self.skip_whitespace();

        match self.peek_byte() {
            Some(b'{') => self.parse_object(),
            Some(b'[') => self.parse_array(),
            Some(b'"') => self.parse_string().map(JsonValue::String),
            Some(b't') => {
                self.consume_literal("true")?;
                Ok(JsonValue::Bool(true))
            }
            Some(b'f') => {
                self.consume_literal("false")?;
                Ok(JsonValue::Bool(false))
            }
            Some(b'n') => {
                self.consume_literal("null")?;
                Ok(JsonValue::Null)
            }
            Some(b'-' | b'0'..=b'9') => self.parse_number().map(JsonValue::Number),
            Some(other) => Err(format!("unexpected byte `{}`", char::from(other))),
            None => Err("unexpected end of input".to_string()),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, String> {
        self.expect_byte(b'{')?;
        self.skip_whitespace();

        let mut values = BTreeMap::new();
        if self.peek_byte() == Some(b'}') {
            self.index += 1;
            return Ok(JsonValue::Object(values));
        }

        loop {
            let key = self.parse_string()?;
            self.skip_whitespace();
            self.expect_byte(b':')?;
            let value = self.parse_value()?;
            values.insert(key, value);
            self.skip_whitespace();

            match self.peek_byte() {
                Some(b',') => {
                    self.index += 1;
                    self.skip_whitespace();
                }
                Some(b'}') => {
                    self.index += 1;
                    break;
                }
                Some(other) => {
                    return Err(format!(
                        "unexpected byte `{}` while parsing object",
                        char::from(other)
                    ));
                }
                None => return Err("unexpected end of input while parsing object".to_string()),
            }
        }

        Ok(JsonValue::Object(values))
    }

    fn parse_array(&mut self) -> Result<JsonValue, String> {
        self.expect_byte(b'[')?;
        self.skip_whitespace();

        let mut values = Vec::new();
        if self.peek_byte() == Some(b']') {
            self.index += 1;
            return Ok(JsonValue::Array(values));
        }

        loop {
            values.push(self.parse_value()?);
            self.skip_whitespace();

            match self.peek_byte() {
                Some(b',') => {
                    self.index += 1;
                    self.skip_whitespace();
                }
                Some(b']') => {
                    self.index += 1;
                    break;
                }
                Some(other) => {
                    return Err(format!(
                        "unexpected byte `{}` while parsing array",
                        char::from(other)
                    ));
                }
                None => return Err("unexpected end of input while parsing array".to_string()),
            }
        }

        Ok(JsonValue::Array(values))
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.expect_byte(b'"')?;
        let mut output = String::new();
        let mut chunk_start = self.index;

        loop {
            match self.next_byte() {
                Some(b'"') => {
                    self.push_utf8_chunk(chunk_start, self.index - 1, &mut output)?;
                    return Ok(output);
                }
                Some(b'\\') => {
                    let escape_start = self.index - 1;
                    self.push_utf8_chunk(chunk_start, escape_start, &mut output)?;

                    match self.next_byte() {
                        Some(b'"') => output.push('"'),
                        Some(b'\\') => output.push('\\'),
                        Some(b'/') => output.push('/'),
                        Some(b'b') => output.push('\u{0008}'),
                        Some(b'f') => output.push('\u{000c}'),
                        Some(b'n') => output.push('\n'),
                        Some(b'r') => output.push('\r'),
                        Some(b't') => output.push('\t'),
                        Some(b'u') => {
                            let first = self.parse_unicode_escape_unit()?;
                            let codepoint = if (0xd800..=0xdbff).contains(&first) {
                                self.expect_byte(b'\\')?;
                                self.expect_byte(b'u')?;
                                let second = self.parse_unicode_escape_unit()?;
                                if !(0xdc00..=0xdfff).contains(&second) {
                                    return Err("invalid low surrogate in \\u escape".to_string());
                                }

                                0x1_0000
                                    + (((u32::from(first) - 0xd800) << 10)
                                        | (u32::from(second) - 0xdc00))
                            } else if (0xdc00..=0xdfff).contains(&first) {
                                return Err("unexpected low surrogate in \\u escape".to_string());
                            } else {
                                u32::from(first)
                            };

                            let ch = char::from_u32(codepoint)
                                .ok_or_else(|| "invalid unicode scalar value".to_string())?;
                            output.push(ch);
                        }
                        Some(other) => {
                            return Err(format!(
                                "unsupported escape sequence `\\{}`",
                                char::from(other)
                            ));
                        }
                        None => return Err("unterminated escape sequence".to_string()),
                    }

                    chunk_start = self.index;
                }
                Some(other) if other < 0x20 => {
                    return Err(format!(
                        "unexpected control byte `\\u{:04x}` in string literal",
                        u32::from(other)
                    ));
                }
                Some(_) => {}
                None => return Err("unterminated string literal".to_string()),
            }
        }
    }

    fn parse_number(&mut self) -> Result<String, String> {
        let start = self.index;
        if self.peek_byte() == Some(b'-') {
            self.index += 1;
        }

        match self.peek_byte() {
            Some(b'0') => {
                self.index += 1;
            }
            Some(b'1'..=b'9') => {
                self.index += 1;
                while matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                    self.index += 1;
                }
            }
            _ => return Err("invalid number literal".to_string()),
        }

        if self.peek_byte() == Some(b'.') {
            self.index += 1;
            let mut consumed_digit = false;
            while matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                self.index += 1;
                consumed_digit = true;
            }
            if !consumed_digit {
                return Err("invalid number literal".to_string());
            }
        }

        if matches!(self.peek_byte(), Some(b'e' | b'E')) {
            self.index += 1;
            if matches!(self.peek_byte(), Some(b'+' | b'-')) {
                self.index += 1;
            }
            let mut consumed_digit = false;
            while matches!(self.peek_byte(), Some(b'0'..=b'9')) {
                self.index += 1;
                consumed_digit = true;
            }
            if !consumed_digit {
                return Err("invalid number literal".to_string());
            }
        }

        String::from_utf8(self.input[start..self.index].to_vec())
            .map_err(|_| "number literal was not valid UTF-8".to_string())
    }

    fn parse_unicode_escape_unit(&mut self) -> Result<u16, String> {
        let mut code_unit = 0u16;
        for _ in 0..4 {
            let digit = self
                .next_byte()
                .ok_or_else(|| "unterminated \\u escape".to_string())?;
            code_unit = (code_unit << 4)
                | hex_value(digit).ok_or_else(|| "invalid \\u escape".to_string())?;
        }

        Ok(code_unit)
    }

    fn push_utf8_chunk(&self, start: usize, end: usize, output: &mut String) -> Result<(), String> {
        if start == end {
            return Ok(());
        }

        let chunk = str::from_utf8(&self.input[start..end])
            .map_err(|error| format!("invalid utf-8 in string literal: {error}"))?;
        output.push_str(chunk);
        Ok(())
    }

    fn consume_literal(&mut self, literal: &str) -> Result<(), String> {
        for expected in literal.bytes() {
            let found = self
                .next_byte()
                .ok_or_else(|| format!("unexpected end of input while parsing `{literal}`"))?;
            if found != expected {
                return Err(format!("expected literal `{literal}`"));
            }
        }

        Ok(())
    }

    fn expect_byte(&mut self, expected: u8) -> Result<(), String> {
        match self.next_byte() {
            Some(found) if found == expected => Ok(()),
            Some(found) => Err(format!(
                "expected byte `{}`, found `{}`",
                char::from(expected),
                char::from(found)
            )),
            None => Err(format!(
                "expected byte `{}`, found end of input",
                char::from(expected)
            )),
        }
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek_byte(), Some(b' ' | b'\n' | b'\r' | b'\t')) {
            self.index += 1;
        }
    }

    fn peek_byte(&self) -> Option<u8> {
        self.input.get(self.index).copied()
    }

    fn next_byte(&mut self) -> Option<u8> {
        let byte = self.peek_byte()?;
        self.index += 1;
        Some(byte)
    }
}

fn hex_value(byte: u8) -> Option<u16> {
    match byte {
        b'0'..=b'9' => Some(u16::from(byte - b'0')),
        b'a'..=b'f' => Some(u16::from(byte - b'a' + 10)),
        b'A'..=b'F' => Some(u16::from(byte - b'A' + 10)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{JsonValue, parse_json, parse_json_object};

    #[test]
    fn parser_round_trips_utf8_and_surrogate_pairs_in_strings() {
        let value = parse_json(r#"{"text":"你好 \uD83D\uDE80"}"#).expect("json should parse");
        let JsonValue::Object(object) = value else {
            panic!("expected object");
        };
        assert_eq!(
            object.get("text"),
            Some(&JsonValue::String("你好 🚀".to_string()))
        );
    }

    #[test]
    fn parser_handles_nested_json_scalars_and_arrays() {
        let value =
            parse_json(r#"{"array":[null,true,false,-12.5e+2],"object":{"escaped":"a\nb\tc"}}"#)
                .expect("nested json should parse");

        let JsonValue::Object(object) = value else {
            panic!("expected object");
        };
        assert_eq!(
            object.get("array"),
            Some(&JsonValue::Array(vec![
                JsonValue::Null,
                JsonValue::Bool(true),
                JsonValue::Bool(false),
                JsonValue::Number("-12.5e+2".to_string()),
            ]))
        );

        let Some(JsonValue::Object(nested)) = object.get("object") else {
            panic!("expected nested object");
        };
        assert_eq!(
            nested.get("escaped"),
            Some(&JsonValue::String("a\nb\tc".to_string()))
        );
    }

    #[test]
    fn parser_reports_malformed_json_without_panicking() {
        assert_eq!(
            parse_json_object("[1]").expect_err("top-level arrays are not objects"),
            "expected top-level object, found array"
        );
        assert_eq!(
            parse_json("01").expect_err("leading zero should leave trailing input"),
            "trailing characters after JSON value"
        );
        assert_eq!(
            parse_json(r#""\uD83D\u0000""#).expect_err("surrogate pair should validate"),
            "invalid low surrogate in \\u escape"
        );
        assert_eq!(
            parse_json(r#""\uDE80""#).expect_err("low surrogate cannot stand alone"),
            "unexpected low surrogate in \\u escape"
        );
        assert_eq!(
            parse_json(r#""\x""#).expect_err("unknown escapes should be rejected"),
            "unsupported escape sequence `\\x`"
        );
        assert_eq!(
            parse_json("\"unterminated").expect_err("unterminated strings should be rejected"),
            "unterminated string literal"
        );
        assert_eq!(
            parse_json("[1 true]").expect_err("array items need separators"),
            "unexpected byte `t` while parsing array"
        );
    }
}
