use super::YamlParseError;

pub(super) fn parse_quoted_value(
    text: &str,
    offset: usize,
    allow_comment: bool,
) -> Result<String, YamlParseError> {
    let (value, consumed) = match text.as_bytes()[0] {
        b'\'' => parse_single_quoted(text, offset)?,
        b'"' => parse_double_quoted(text, offset)?,
        _ => unreachable!("quoted parser should only receive quoted text"),
    };
    validate_trailing(text, offset, consumed, allow_comment)?;
    Ok(value)
}

pub(super) fn validate_trailing(
    text: &str,
    offset: usize,
    consumed: usize,
    allow_comment: bool,
) -> Result<(), YamlParseError> {
    let rest = &text[consumed..];
    let spaces = count_leading_spaces(rest);
    let trailing = &rest[spaces..];
    if trailing.is_empty() {
        return Ok(());
    }
    if allow_comment && spaces > 0 && trailing.starts_with('#') {
        return Ok(());
    }
    Err(YamlParseError::new(
        offset + consumed + spaces,
        "unexpected characters after scalar",
    ))
}

fn parse_single_quoted(text: &str, offset: usize) -> Result<(String, usize), YamlParseError> {
    let bytes = text.as_bytes();
    let mut pos = 1_usize;
    let mut out = String::new();

    while pos < bytes.len() {
        if bytes[pos] == b'\'' {
            if bytes.get(pos + 1) == Some(&b'\'') {
                out.push('\'');
                pos += 2;
            } else {
                return Ok((out, pos + 1));
            }
        } else {
            let ch = text[pos..]
                .chars()
                .next()
                .expect("valid UTF-8 input should yield a char before EOF");
            pos += ch.len_utf8();
            out.push(ch);
        }
    }

    Err(YamlParseError::new(
        offset,
        "unterminated single-quoted string",
    ))
}

fn parse_double_quoted(text: &str, offset: usize) -> Result<(String, usize), YamlParseError> {
    let bytes = text.as_bytes();
    let mut pos = 1_usize;
    let mut out = String::new();

    while pos < bytes.len() {
        match bytes[pos] {
            b'"' => return Ok((out, pos + 1)),
            b'\\' => {
                let escape_offset = offset + pos;
                pos += 1;
                parse_double_escape(text, &mut pos, escape_offset, &mut out)?;
            }
            0x00..=0x1f => {
                return Err(YamlParseError::new(
                    offset + pos,
                    "unescaped control character in double-quoted string",
                ));
            }
            _ => {
                let ch = text[pos..]
                    .chars()
                    .next()
                    .expect("valid UTF-8 input should yield a char before EOF");
                pos += ch.len_utf8();
                out.push(ch);
            }
        }
    }

    Err(YamlParseError::new(
        offset,
        "unterminated double-quoted string",
    ))
}

fn parse_double_escape(
    text: &str,
    pos: &mut usize,
    escape_offset: usize,
    out: &mut String,
) -> Result<(), YamlParseError> {
    let Some(byte) = text.as_bytes().get(*pos).copied() else {
        return Err(YamlParseError::new(
            escape_offset,
            "unterminated string escape",
        ));
    };
    *pos += 1;
    match byte {
        b'"' => out.push('"'),
        b'\\' => out.push('\\'),
        b'/' => out.push('/'),
        b'b' => out.push('\u{0008}'),
        b'f' => out.push('\u{000c}'),
        b'n' => out.push('\n'),
        b'r' => out.push('\r'),
        b't' => out.push('\t'),
        b'u' => parse_unicode_escape(text, pos, escape_offset, out)?,
        _ => return Err(YamlParseError::new(escape_offset, "invalid string escape")),
    }
    Ok(())
}

fn parse_unicode_escape(
    text: &str,
    pos: &mut usize,
    escape_offset: usize,
    out: &mut String,
) -> Result<(), YamlParseError> {
    let first = read_hex_u16(text, pos, escape_offset)?;
    if is_high_surrogate(first) {
        if text.as_bytes().get(*pos) != Some(&b'\\') || text.as_bytes().get(*pos + 1) != Some(&b'u')
        {
            return Err(YamlParseError::new(
                escape_offset,
                "high surrogate must be followed by a low surrogate escape",
            ));
        }
        *pos += 2;
        let low_offset = escape_offset + 6;
        let second = read_hex_u16(text, pos, low_offset)?;
        if !is_low_surrogate(second) {
            return Err(YamlParseError::new(
                low_offset,
                "high surrogate must be followed by a low surrogate escape",
            ));
        }
        let codepoint = 0x10000 + (((first as u32 - 0xd800) << 10) | (second as u32 - 0xdc00));
        let ch = char::from_u32(codepoint)
            .ok_or_else(|| YamlParseError::new(escape_offset, "invalid Unicode scalar value"))?;
        out.push(ch);
        return Ok(());
    }
    if is_low_surrogate(first) {
        return Err(YamlParseError::new(
            escape_offset,
            "low surrogate cannot appear without a preceding high surrogate",
        ));
    }

    let ch = char::from_u32(first as u32)
        .ok_or_else(|| YamlParseError::new(escape_offset, "invalid Unicode scalar value"))?;
    out.push(ch);
    Ok(())
}

fn read_hex_u16(text: &str, pos: &mut usize, offset: usize) -> Result<u16, YamlParseError> {
    if *pos + 4 > text.len() {
        return Err(YamlParseError::new(
            offset,
            "Unicode escape must contain four hexadecimal digits",
        ));
    }

    let mut value = 0_u16;
    for _ in 0..4 {
        let byte = text.as_bytes()[*pos];
        let Some(digit) = hex_value(byte) else {
            return Err(YamlParseError::new(
                offset,
                "Unicode escape must contain four hexadecimal digits",
            ));
        };
        *pos += 1;
        value = value * 16 + u16::from(digit);
    }
    Ok(value)
}

fn count_leading_spaces(text: &str) -> usize {
    text.bytes().take_while(|byte| *byte == b' ').count()
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn is_high_surrogate(value: u16) -> bool {
    (0xd800..=0xdbff).contains(&value)
}

fn is_low_surrogate(value: u16) -> bool {
    (0xdc00..=0xdfff).contains(&value)
}
