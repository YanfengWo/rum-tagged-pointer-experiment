use std::{collections::BTreeMap, fs, path::Path};

use crate::{KernelChannel, KernelSignaturePolicy};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KernelTransportKind {
    Tcp,
    InProc,
}

impl KernelTransportKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Tcp => "tcp",
            Self::InProc => "inproc",
        }
    }

    fn parse(value: &str) -> Result<Self, KernelConnectionFileError> {
        match value {
            "tcp" => Ok(Self::Tcp),
            "inproc" => Ok(Self::InProc),
            other => Err(KernelConnectionFileError::InvalidField {
                field: "transport",
                value: other.to_string(),
            }),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KernelSignatureScheme {
    HmacSha256,
    None,
    Other(String),
}

impl KernelSignatureScheme {
    pub fn as_str(&self) -> &str {
        match self {
            Self::HmacSha256 => "hmac-sha256",
            Self::None => "",
            Self::Other(value) => value.as_str(),
        }
    }

    fn parse(value: &str) -> Self {
        match value {
            "" => Self::None,
            "hmac-sha256" => Self::HmacSha256,
            other => Self::Other(other.to_string()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelConnectionFile {
    pub transport: KernelTransportKind,
    pub ip: String,
    pub shell_port: u16,
    pub iopub_port: u16,
    pub stdin_port: u16,
    pub control_port: u16,
    pub hb_port: u16,
    pub key: String,
    pub signature_scheme: KernelSignatureScheme,
    pub kernel_name: Option<String>,
}

impl KernelConnectionFile {
    pub fn parse_json(input: &str) -> Result<Self, KernelConnectionFileError> {
        let object = parse_object(input)?;

        Ok(Self {
            transport: KernelTransportKind::parse(&string_field(&object, "transport")?)?,
            ip: string_field(&object, "ip")?,
            shell_port: u16_field(&object, "shell_port")?,
            iopub_port: u16_field(&object, "iopub_port")?,
            stdin_port: u16_field(&object, "stdin_port")?,
            control_port: u16_field(&object, "control_port")?,
            hb_port: u16_field(&object, "hb_port")?,
            key: string_field(&object, "key")?,
            signature_scheme: KernelSignatureScheme::parse(&string_field(
                &object,
                "signature_scheme",
            )?),
            kernel_name: optional_string_field(&object, "kernel_name")?,
        })
    }

    pub fn read_from_path(path: impl AsRef<Path>) -> Result<Self, KernelConnectionFileError> {
        let path = path.as_ref();
        let input = fs::read_to_string(path).map_err(|error| KernelConnectionFileError::Io {
            path: path.display().to_string(),
            message: error.to_string(),
        })?;
        Self::parse_json(&input)
    }

    pub fn channel_port(&self, channel: KernelChannel) -> u16 {
        match channel {
            KernelChannel::Shell => self.shell_port,
            KernelChannel::Control => self.control_port,
            KernelChannel::IoPub => self.iopub_port,
            KernelChannel::Stdin => self.stdin_port,
        }
    }

    pub fn channel_endpoint(&self, channel: KernelChannel) -> String {
        format!(
            "{}://{}:{}",
            self.transport.as_str(),
            self.ip,
            self.channel_port(channel)
        )
    }

    pub fn heartbeat_endpoint(&self) -> String {
        format!("{}://{}:{}", self.transport.as_str(), self.ip, self.hb_port)
    }

    pub fn signature_policy(&self) -> KernelSignaturePolicy {
        if self.key.is_empty() {
            KernelSignaturePolicy::Disabled
        } else {
            KernelSignaturePolicy::Required(self.key.clone())
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KernelConnectionFileError {
    Io { path: String, message: String },
    MissingField { field: &'static str },
    InvalidField { field: &'static str, value: String },
    InvalidJson { message: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum JsonValue {
    Null,
    Bool(bool),
    Number(String),
    String(String),
    Array(Vec<JsonValue>),
    Object(BTreeMap<String, JsonValue>),
}

fn parse_object(input: &str) -> Result<BTreeMap<String, JsonValue>, KernelConnectionFileError> {
    match JsonParser::parse(input)? {
        JsonValue::Object(object) => Ok(object),
        other => Err(KernelConnectionFileError::InvalidJson {
            message: format!("expected top-level object, found {}", other.kind_name()),
        }),
    }
}

fn string_field(
    object: &BTreeMap<String, JsonValue>,
    field: &'static str,
) -> Result<String, KernelConnectionFileError> {
    match object.get(field) {
        Some(JsonValue::String(value)) => Ok(value.clone()),
        Some(other) => Err(KernelConnectionFileError::InvalidField {
            field,
            value: other.kind_name().to_string(),
        }),
        None => Err(KernelConnectionFileError::MissingField { field }),
    }
}

fn optional_string_field(
    object: &BTreeMap<String, JsonValue>,
    field: &'static str,
) -> Result<Option<String>, KernelConnectionFileError> {
    match object.get(field) {
        Some(JsonValue::String(value)) => Ok(Some(value.clone())),
        Some(JsonValue::Null) | None => Ok(None),
        Some(other) => Err(KernelConnectionFileError::InvalidField {
            field,
            value: other.kind_name().to_string(),
        }),
    }
}

fn u16_field(
    object: &BTreeMap<String, JsonValue>,
    field: &'static str,
) -> Result<u16, KernelConnectionFileError> {
    match object.get(field) {
        Some(JsonValue::Number(value)) => {
            value
                .parse::<u16>()
                .map_err(|_| KernelConnectionFileError::InvalidField {
                    field,
                    value: value.clone(),
                })
        }
        Some(other) => Err(KernelConnectionFileError::InvalidField {
            field,
            value: other.kind_name().to_string(),
        }),
        None => Err(KernelConnectionFileError::MissingField { field }),
    }
}

impl JsonValue {
    fn kind_name(&self) -> &'static str {
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

struct JsonParser<'a> {
    input: &'a [u8],
    index: usize,
}

impl<'a> JsonParser<'a> {
    fn parse(input: &'a str) -> Result<JsonValue, KernelConnectionFileError> {
        let mut parser = Self {
            input: input.as_bytes(),
            index: 0,
        };
        let value = parser.parse_value()?;
        parser.skip_whitespace();

        if parser.index != parser.input.len() {
            return Err(KernelConnectionFileError::InvalidJson {
                message: "trailing characters after JSON value".to_string(),
            });
        }

        Ok(value)
    }

    fn parse_value(&mut self) -> Result<JsonValue, KernelConnectionFileError> {
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
            Some(other) => Err(KernelConnectionFileError::InvalidJson {
                message: format!("unexpected byte `{}`", char::from(other)),
            }),
            None => Err(KernelConnectionFileError::InvalidJson {
                message: "unexpected end of input".to_string(),
            }),
        }
    }

    fn parse_object(&mut self) -> Result<JsonValue, KernelConnectionFileError> {
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
                    return Err(KernelConnectionFileError::InvalidJson {
                        message: format!(
                            "unexpected byte `{}` while parsing object",
                            char::from(other)
                        ),
                    });
                }
                None => {
                    return Err(KernelConnectionFileError::InvalidJson {
                        message: "unexpected end of input while parsing object".to_string(),
                    });
                }
            }
        }

        Ok(JsonValue::Object(values))
    }

    fn parse_array(&mut self) -> Result<JsonValue, KernelConnectionFileError> {
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
                    return Err(KernelConnectionFileError::InvalidJson {
                        message: format!(
                            "unexpected byte `{}` while parsing array",
                            char::from(other)
                        ),
                    });
                }
                None => {
                    return Err(KernelConnectionFileError::InvalidJson {
                        message: "unexpected end of input while parsing array".to_string(),
                    });
                }
            }
        }

        Ok(JsonValue::Array(values))
    }

    fn parse_string(&mut self) -> Result<String, KernelConnectionFileError> {
        self.expect_byte(b'"')?;
        let mut output = String::new();

        loop {
            match self.next_byte() {
                Some(b'"') => break,
                Some(b'\\') => match self.next_byte() {
                    Some(b'"') => output.push('"'),
                    Some(b'\\') => output.push('\\'),
                    Some(b'n') => output.push('\n'),
                    Some(b'r') => output.push('\r'),
                    Some(b't') => output.push('\t'),
                    Some(b'u') => {
                        let mut codepoint = 0u32;
                        for _ in 0..4 {
                            let digit = self.next_byte().ok_or_else(|| {
                                KernelConnectionFileError::InvalidJson {
                                    message: "unterminated \\u escape".to_string(),
                                }
                            })?;
                            codepoint = (codepoint << 4)
                                | hex_value(digit).ok_or_else(|| {
                                    KernelConnectionFileError::InvalidJson {
                                        message: "invalid \\u escape".to_string(),
                                    }
                                })?;
                        }
                        let ch = char::from_u32(codepoint).ok_or_else(|| {
                            KernelConnectionFileError::InvalidJson {
                                message: "invalid unicode scalar value".to_string(),
                            }
                        })?;
                        output.push(ch);
                    }
                    Some(other) => {
                        return Err(KernelConnectionFileError::InvalidJson {
                            message: format!(
                                "unsupported escape sequence `\\{}`",
                                char::from(other)
                            ),
                        });
                    }
                    None => {
                        return Err(KernelConnectionFileError::InvalidJson {
                            message: "unterminated escape sequence".to_string(),
                        });
                    }
                },
                Some(other) => output.push(char::from(other)),
                None => {
                    return Err(KernelConnectionFileError::InvalidJson {
                        message: "unterminated string literal".to_string(),
                    });
                }
            }
        }

        Ok(output)
    }

    fn parse_number(&mut self) -> Result<String, KernelConnectionFileError> {
        let start = self.index;
        if self.peek_byte() == Some(b'-') {
            self.index += 1;
        }

        match self.peek_byte() {
            Some(b'0'..=b'9') => {
                self.index += 1;
            }
            _ => {
                return Err(KernelConnectionFileError::InvalidJson {
                    message: "invalid number literal".to_string(),
                });
            }
        }

        while matches!(self.peek_byte(), Some(b'0'..=b'9')) {
            self.index += 1;
        }

        String::from_utf8(self.input[start..self.index].to_vec()).map_err(|_| {
            KernelConnectionFileError::InvalidJson {
                message: "number literal was not valid UTF-8".to_string(),
            }
        })
    }

    fn consume_literal(&mut self, literal: &str) -> Result<(), KernelConnectionFileError> {
        for expected in literal.bytes() {
            let found = self
                .next_byte()
                .ok_or_else(|| KernelConnectionFileError::InvalidJson {
                    message: format!("unexpected end of input while parsing `{literal}`"),
                })?;
            if found != expected {
                return Err(KernelConnectionFileError::InvalidJson {
                    message: format!("expected literal `{literal}`"),
                });
            }
        }

        Ok(())
    }

    fn expect_byte(&mut self, expected: u8) -> Result<(), KernelConnectionFileError> {
        match self.next_byte() {
            Some(found) if found == expected => Ok(()),
            Some(found) => Err(KernelConnectionFileError::InvalidJson {
                message: format!(
                    "expected byte `{}`, found `{}`",
                    char::from(expected),
                    char::from(found)
                ),
            }),
            None => Err(KernelConnectionFileError::InvalidJson {
                message: format!(
                    "expected byte `{}`, found end of input",
                    char::from(expected)
                ),
            }),
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

fn hex_value(byte: u8) -> Option<u32> {
    match byte {
        b'0'..=b'9' => Some(u32::from(byte - b'0')),
        b'a'..=b'f' => Some(u32::from(byte - b'a' + 10)),
        b'A'..=b'F' => Some(u32::from(byte - b'A' + 10)),
        _ => None,
    }
}
