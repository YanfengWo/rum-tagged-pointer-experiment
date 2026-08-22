use rum_core::Expr;

use super::YamlParseError;
use super::quoted::{parse_quoted_value, validate_trailing};
use crate::yaml::number::parse_yaml_number;
use crate::yaml::{association_expr, list_expr, symbol_value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Chomping {
    Clip,
    Strip,
}

pub(super) fn is_sequence_marker(text: &str) -> bool {
    text == "-" || text.starts_with("- ")
}

pub(super) fn find_mapping_colon(text: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut pos = 0_usize;
    let mut quote = None;

    while pos < bytes.len() {
        match (bytes[pos], quote) {
            (b'\'', None) => {
                quote = Some(b'\'');
                pos += 1;
            }
            (b'"', None) => {
                quote = Some(b'"');
                pos += 1;
            }
            (b'\'', Some(b'\'')) => {
                if bytes.get(pos + 1) == Some(&b'\'') {
                    pos += 2;
                } else {
                    quote = None;
                    pos += 1;
                }
            }
            (b'"', Some(b'"')) => {
                quote = None;
                pos += 1;
            }
            (b'\\', Some(b'"')) => pos += 2,
            (b':', None)
                if bytes
                    .get(pos + 1)
                    .is_none_or(|byte| byte.is_ascii_whitespace()) =>
            {
                return Some(pos);
            }
            _ => pos += 1,
        }
    }

    None
}

pub(super) fn parse_key(raw: &str, offset: usize) -> Result<String, YamlParseError> {
    let leading = count_leading_spaces(raw);
    let trimmed = raw[leading..].trim_end();
    let key_offset = offset + leading;
    if trimmed.is_empty() {
        return Err(YamlParseError::new(
            key_offset,
            "mapping keys cannot be empty",
        ));
    }

    match trimmed.as_bytes()[0] {
        b'\'' | b'"' => parse_quoted_value(trimmed, key_offset, false),
        b'{' | b'[' => Err(YamlParseError::new(
            key_offset,
            "flow collection keys are outside rum's YAML subset",
        )),
        b'&' => Err(YamlParseError::new(
            key_offset,
            "YAML anchors are unsupported",
        )),
        b'*' => Err(YamlParseError::new(
            key_offset,
            "YAML aliases are unsupported",
        )),
        b'!' => Err(YamlParseError::new(key_offset, "YAML tags are unsupported")),
        b'%' => Err(YamlParseError::new(
            key_offset,
            "YAML directives are unsupported",
        )),
        _ => {
            reject_unsupported_plain_tokens(trimmed, key_offset)?;
            Ok(trimmed.to_string())
        }
    }
}

pub(super) fn count_leading_spaces(text: &str) -> usize {
    text.bytes().take_while(|byte| *byte == b' ').count()
}

pub(super) fn parse_quoted_scalar(text: &str, offset: usize) -> Result<Expr, YamlParseError> {
    parse_quoted_value(text, offset, true).map(Expr::string)
}

pub(super) fn parse_plain_scalar(text: &str, offset: usize) -> Result<Expr, YamlParseError> {
    let end = plain_scalar_end(text);
    let raw = text[..end].trim_end();
    if raw.is_empty() {
        return Ok(symbol_value("Null"));
    }

    if raw == "{}" {
        return Ok(association_expr(Vec::new()));
    }
    if raw == "[]" {
        return Ok(list_expr(Vec::new()));
    }
    if raw.starts_with('{') || raw.starts_with('[') {
        return Err(YamlParseError::new(
            offset,
            "non-empty flow collections are outside rum's YAML subset",
        ));
    }
    reject_unsupported_plain_tokens(raw, offset)?;

    match raw {
        "true" => Ok(symbol_value("True")),
        "false" => Ok(symbol_value("False")),
        "null" | "~" => Ok(symbol_value("Null")),
        _ => {
            parse_yaml_number(raw, offset).map(|number| number.unwrap_or_else(|| Expr::string(raw)))
        }
    }
}

pub(super) fn parse_literal_header(text: &str, offset: usize) -> Result<Chomping, YamlParseError> {
    let bytes = text.as_bytes();
    let mut pos = 1_usize;
    let chomping = if bytes.get(pos) == Some(&b'-') {
        pos += 1;
        Chomping::Strip
    } else if bytes.get(pos) == Some(&b'+') {
        return Err(YamlParseError::new(
            offset + pos,
            "literal block scalars support only | and |-",
        ));
    } else {
        Chomping::Clip
    };

    let rest = &text[pos..];
    validate_trailing(rest, offset + pos, 0, true)?;
    Ok(chomping)
}

fn plain_scalar_end(text: &str) -> usize {
    let bytes = text.as_bytes();
    for index in 0..bytes.len() {
        if bytes[index] == b'#' && index > 0 && bytes[index - 1].is_ascii_whitespace() {
            return index;
        }
    }
    text.len()
}

fn reject_unsupported_plain_tokens(raw: &str, offset: usize) -> Result<(), YamlParseError> {
    for (index, byte) in raw.bytes().enumerate() {
        let boundary = index == 0 || raw.as_bytes()[index - 1].is_ascii_whitespace();
        if !boundary {
            continue;
        }
        match byte {
            b'&' => {
                return Err(YamlParseError::new(
                    offset + index,
                    "YAML anchors are unsupported",
                ));
            }
            b'*' => {
                return Err(YamlParseError::new(
                    offset + index,
                    "YAML aliases are unsupported",
                ));
            }
            b'!' => {
                return Err(YamlParseError::new(
                    offset + index,
                    "YAML tags are unsupported",
                ));
            }
            b'%' => {
                return Err(YamlParseError::new(
                    offset + index,
                    "YAML directives are unsupported",
                ));
            }
            _ => {}
        }
    }
    Ok(())
}
