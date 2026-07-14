//! A small, ordered JSON value model with a bounded decoder.

use std::fmt::{self, Write as _};

/// Maximum number of nested JSON arrays and objects accepted by [`decode`].
pub const MAX_DEPTH: usize = 64;

/// A JSON value restricted to the types used by the Dun Plugin Protocol.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum JsonValue {
    /// The JSON `null` value.
    Null,
    /// A JSON Boolean.
    Bool(bool),
    /// A signed integer JSON number.
    Int(i64),
    /// A JSON string.
    Str(String),
    /// An ordered JSON array.
    Array(Vec<JsonValue>),
    /// An insertion-ordered JSON object.
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    /// Looks up the first matching key in an object.
    pub fn get(&self, key: &str) -> Option<&JsonValue> {
        self.as_object()?
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }

    /// Returns this value's string contents, if it is a string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(value) => Some(value),
            _ => None,
        }
    }

    /// Returns this value's integer, if it is an integer.
    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Int(value) => Some(*value),
            _ => None,
        }
    }

    /// Returns this value's array items, if it is an array.
    pub fn as_array(&self) -> Option<&[JsonValue]> {
        match self {
            Self::Array(items) => Some(items),
            _ => None,
        }
    }

    /// Returns this value's ordered fields, if it is an object.
    pub fn as_object(&self) -> Option<&[(String, JsonValue)]> {
        match self {
            Self::Object(fields) => Some(fields),
            _ => None,
        }
    }
}

/// A malformed-JSON error with its byte offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JsonError {
    offset: usize,
    message: &'static str,
}

impl JsonError {
    /// Returns the byte offset at which decoding failed.
    pub const fn offset(self) -> usize {
        self.offset
    }

    /// Returns a short description of the decoding failure.
    pub const fn message(self) -> &'static str {
        self.message
    }
}

impl fmt::Display for JsonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid JSON at byte {}: {}",
            self.offset, self.message
        )
    }
}

impl std::error::Error for JsonError {}

/// Encodes a JSON value compactly while preserving object field order.
pub fn encode(value: &JsonValue) -> String {
    let mut output = String::new();
    encode_value(value, &mut output);
    output
}

/// Decodes one complete JSON value.
///
/// Only lexical integer number forms are accepted. Fractional and exponent
/// forms are rejected rather than rounded or truncated.
pub fn decode(text: &str) -> Result<JsonValue, JsonError> {
    let mut parser = Parser {
        bytes: text.as_bytes(),
        position: 0,
    };
    let value = parser.parse_value(0)?;
    parser.skip_whitespace();
    if parser.position != parser.bytes.len() {
        return Err(parser.error("trailing data"));
    }
    Ok(value)
}

fn encode_value(value: &JsonValue, output: &mut String) {
    match value {
        JsonValue::Null => output.push_str("null"),
        JsonValue::Bool(true) => output.push_str("true"),
        JsonValue::Bool(false) => output.push_str("false"),
        JsonValue::Int(value) => {
            let _ = write!(output, "{value}");
        }
        JsonValue::Str(value) => encode_string(value, output),
        JsonValue::Array(items) => {
            output.push('[');
            for (index, item) in items.iter().enumerate() {
                if index != 0 {
                    output.push(',');
                }
                encode_value(item, output);
            }
            output.push(']');
        }
        JsonValue::Object(fields) => {
            output.push('{');
            for (index, (key, value)) in fields.iter().enumerate() {
                if index != 0 {
                    output.push(',');
                }
                encode_string(key, output);
                output.push(':');
                encode_value(value, output);
            }
            output.push('}');
        }
    }
}

fn encode_string(value: &str, output: &mut String) {
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\u{0000}'..='\u{001f}' => {
                let _ = write!(output, "\\u{:04X}", character as u32);
            }
            _ => output.push(character),
        }
    }
    output.push('"');
}

struct Parser<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl Parser<'_> {
    fn error(&self, message: &'static str) -> JsonError {
        JsonError {
            offset: self.position,
            message,
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.position).copied()
    }

    fn bump(&mut self) -> Option<u8> {
        let byte = self.peek()?;
        self.position += 1;
        Some(byte)
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.position += 1;
        }
    }

    fn expect(&mut self, expected: u8, message: &'static str) -> Result<(), JsonError> {
        if self.bump() == Some(expected) {
            Ok(())
        } else {
            Err(self.error(message))
        }
    }

    fn parse_value(&mut self, depth: usize) -> Result<JsonValue, JsonError> {
        self.skip_whitespace();
        match self.peek() {
            Some(b'n') => self.parse_literal(b"null", JsonValue::Null),
            Some(b't') => self.parse_literal(b"true", JsonValue::Bool(true)),
            Some(b'f') => self.parse_literal(b"false", JsonValue::Bool(false)),
            Some(b'"') => self.parse_string().map(JsonValue::Str),
            Some(b'[') if depth < MAX_DEPTH => self.parse_array(depth + 1),
            Some(b'{') if depth < MAX_DEPTH => self.parse_object(depth + 1),
            Some(b'[' | b'{') => Err(self.error("nesting too deep")),
            Some(b'-' | b'0'..=b'9') => self.parse_number(),
            _ => Err(self.error("expected JSON value")),
        }
    }

    fn parse_literal(
        &mut self,
        literal: &'static [u8],
        value: JsonValue,
    ) -> Result<JsonValue, JsonError> {
        for expected in literal {
            if self.bump() != Some(*expected) {
                return Err(self.error("invalid literal"));
            }
        }
        Ok(value)
    }

    fn parse_number(&mut self) -> Result<JsonValue, JsonError> {
        let start = self.position;
        if self.peek() == Some(b'-') {
            self.position += 1;
        }

        match self.peek() {
            Some(b'0') => {
                self.position += 1;
                if matches!(self.peek(), Some(b'0'..=b'9')) {
                    return Err(self.error("leading zero in number"));
                }
            }
            Some(b'1'..=b'9') => {
                self.position += 1;
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.position += 1;
                }
            }
            _ => return Err(self.error("invalid number")),
        }

        let mut non_integer_form = false;
        if self.peek() == Some(b'.') {
            non_integer_form = true;
            self.position += 1;
            self.consume_digits("fraction requires a digit")?;
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            non_integer_form = true;
            self.position += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.position += 1;
            }
            self.consume_digits("exponent requires a digit")?;
        }
        if non_integer_form {
            return Err(self.error("non-integer number form"));
        }

        let number_text = std::str::from_utf8(&self.bytes[start..self.position])
            .map_err(|_| self.error("invalid number"))?;
        let value = number_text
            .parse::<i64>()
            .map_err(|_| self.error("integer out of range"))?;
        Ok(JsonValue::Int(value))
    }

    fn consume_digits(&mut self, message: &'static str) -> Result<(), JsonError> {
        if !matches!(self.peek(), Some(b'0'..=b'9')) {
            return Err(self.error(message));
        }
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.position += 1;
        }
        Ok(())
    }

    fn parse_string(&mut self) -> Result<String, JsonError> {
        self.expect(b'"', "expected string")?;
        let mut output = Vec::new();
        loop {
            let byte = self
                .bump()
                .ok_or_else(|| self.error("unterminated string"))?;
            match byte {
                b'"' => {
                    return String::from_utf8(output)
                        .map_err(|_| self.error("invalid UTF-8 in string"));
                }
                b'\\' => {
                    let character = self.parse_escape()?;
                    let mut encoded = [0u8; 4];
                    output.extend_from_slice(character.encode_utf8(&mut encoded).as_bytes());
                }
                0x00..=0x1f => return Err(self.error("control character in string")),
                _ => output.push(byte),
            }
        }
    }

    fn parse_escape(&mut self) -> Result<char, JsonError> {
        let escape = self
            .bump()
            .ok_or_else(|| self.error("unterminated escape"))?;
        match escape {
            b'"' => Ok('"'),
            b'\\' => Ok('\\'),
            b'/' => Ok('/'),
            b'b' => Ok('\u{0008}'),
            b'f' => Ok('\u{000c}'),
            b'n' => Ok('\n'),
            b'r' => Ok('\r'),
            b't' => Ok('\t'),
            b'u' => self.parse_unicode_escape(),
            _ => Err(self.error("invalid escape")),
        }
    }

    fn parse_unicode_escape(&mut self) -> Result<char, JsonError> {
        let first = self.parse_hex4()?;
        let code_point = if (0xd800..=0xdbff).contains(&first) {
            self.expect(b'\\', "missing low surrogate")?;
            self.expect(b'u', "missing low surrogate")?;
            let second = self.parse_hex4()?;
            if !(0xdc00..=0xdfff).contains(&second) {
                return Err(self.error("invalid low surrogate"));
            }
            0x10000 + ((u32::from(first) - 0xd800) << 10) + (u32::from(second) - 0xdc00)
        } else if (0xdc00..=0xdfff).contains(&first) {
            return Err(self.error("unexpected low surrogate"));
        } else {
            u32::from(first)
        };
        char::from_u32(code_point).ok_or_else(|| self.error("invalid Unicode code point"))
    }

    fn parse_hex4(&mut self) -> Result<u16, JsonError> {
        let mut value = 0u16;
        for _ in 0..4 {
            let digit = match self.bump() {
                Some(byte @ b'0'..=b'9') => u16::from(byte - b'0'),
                Some(byte @ b'a'..=b'f') => u16::from(byte - b'a') + 10,
                Some(byte @ b'A'..=b'F') => u16::from(byte - b'A') + 10,
                _ => return Err(self.error("invalid Unicode escape")),
            };
            value = value * 16 + digit;
        }
        Ok(value)
    }

    fn parse_array(&mut self, depth: usize) -> Result<JsonValue, JsonError> {
        self.expect(b'[', "expected array")?;
        let mut items = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b']') {
            self.position += 1;
            return Ok(JsonValue::Array(items));
        }
        loop {
            items.push(self.parse_value(depth)?);
            self.skip_whitespace();
            match self.bump() {
                Some(b',') => {}
                Some(b']') => return Ok(JsonValue::Array(items)),
                _ => return Err(self.error("expected ',' or ']'")),
            }
        }
    }

    fn parse_object(&mut self, depth: usize) -> Result<JsonValue, JsonError> {
        self.expect(b'{', "expected object")?;
        let mut fields = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(b'}') {
            self.position += 1;
            return Ok(JsonValue::Object(fields));
        }
        loop {
            self.skip_whitespace();
            let key = self.parse_string()?;
            self.skip_whitespace();
            self.expect(b':', "expected ':'")?;
            let value = self.parse_value(depth)?;
            fields.push((key, value));
            self.skip_whitespace();
            match self.bump() {
                Some(b',') => {}
                Some(b'}') => return Ok(JsonValue::Object(fields)),
                _ => return Err(self.error("expected ',' or '}'")),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_each_variant_compactly_and_in_order() {
        assert_eq!(encode(&JsonValue::Null), "null");
        assert_eq!(encode(&JsonValue::Bool(true)), "true");
        assert_eq!(encode(&JsonValue::Bool(false)), "false");
        assert_eq!(encode(&JsonValue::Int(-17)), "-17");
        assert_eq!(encode(&JsonValue::Str("rum".into())), r#""rum""#);
        assert_eq!(
            encode(&JsonValue::Array(vec![JsonValue::Int(1), JsonValue::Null])),
            "[1,null]"
        );
        assert_eq!(
            encode(&JsonValue::Object(vec![
                ("b".into(), JsonValue::Int(2)),
                ("a".into(), JsonValue::Int(1)),
            ])),
            r#"{"b":2,"a":1}"#
        );
    }

    #[test]
    fn round_trips_nested_request_envelope() {
        let request = JsonValue::Object(vec![
            ("v".into(), JsonValue::Int(0)),
            ("kind".into(), JsonValue::Str("request".into())),
            ("request_id".into(), JsonValue::Int(7)),
            ("plugin_id".into(), JsonValue::Str("check".into())),
            ("role".into(), JsonValue::Str("syntax-highlight".into())),
            ("revision".into(), JsonValue::Int(41)),
            (
                "payload".into(),
                JsonValue::Object(vec![
                    ("language".into(), JsonValue::Str("rs".into())),
                    ("first_line".into(), JsonValue::Int(10)),
                    (
                        "lines".into(),
                        JsonValue::Array(vec![
                            JsonValue::Str("fn main() {}".into()),
                            JsonValue::Str(String::new()),
                        ]),
                    ),
                ]),
            ),
        ]);

        assert_eq!(decode(&encode(&request)).unwrap(), request);
    }

    #[test]
    fn escapes_and_decodes_string_special_characters() {
        let value = JsonValue::Str("quote \" slash \\ line\n ctrl\u{0001}\r\t é".into());
        let encoded = encode(&value);
        assert_eq!(
            encoded,
            "\"quote \\\" slash \\\\ line\\n ctrl\\u0001\\r\\t é\""
        );
        assert_eq!(decode(&encoded).unwrap(), value);
    }

    #[test]
    fn decodes_checker_hello_literal() {
        let value = decode(
            r#"{"v":0,"kind":"hello","request_id":0,"plugin_id":"","payload":{"host":"dun"}}"#,
        )
        .unwrap();

        assert_eq!(value.get("kind").and_then(JsonValue::as_str), Some("hello"));
        assert_eq!(
            value
                .get("payload")
                .and_then(|payload| payload.get("host"))
                .and_then(JsonValue::as_str),
            Some("dun")
        );
    }

    #[test]
    fn decodes_checker_request_literal() {
        let value = decode(
            r#"{"v":0,"kind":"request","request_id":7,"plugin_id":"check","role":"syntax-highlight","revision":41,"payload":{"language":"rs","first_line":10,"lines":["fn main() { return 42; } // done",""]}}"#,
        )
        .unwrap();

        assert_eq!(value.get("revision").and_then(JsonValue::as_i64), Some(41));
        assert_eq!(
            value
                .get("payload")
                .and_then(|payload| payload.get("lines"))
                .and_then(JsonValue::as_array)
                .map(<[JsonValue]>::len),
            Some(2)
        );
    }

    #[test]
    fn rejects_nesting_over_depth_limit() {
        let at_limit = format!("{}null{}", "[".repeat(MAX_DEPTH), "]".repeat(MAX_DEPTH));
        assert!(decode(&at_limit).is_ok());

        let too_deep = format!(
            "{}null{}",
            "[".repeat(MAX_DEPTH + 1),
            "]".repeat(MAX_DEPTH + 1)
        );
        assert_eq!(decode(&too_deep).unwrap_err().message(), "nesting too deep");
    }

    #[test]
    fn rejects_unterminated_string_and_trailing_garbage() {
        assert!(decode(r#""unterminated"#).is_err());
        assert!(decode("null trailing").is_err());
    }

    #[test]
    fn decodes_bmp_and_surrogate_pair_unicode_escapes() {
        assert_eq!(decode(r#""\u03A9""#).unwrap(), JsonValue::Str("Ω".into()));
        assert_eq!(
            decode(r#""\uD83D\uDE00""#).unwrap(),
            JsonValue::Str("😀".into())
        );
    }

    #[test]
    fn rejects_fractional_and_exponent_number_forms() {
        assert!(decode("1.5").is_err());
        assert!(decode("1.0").is_err());
        assert!(decode("1e2").is_err());
    }
}
