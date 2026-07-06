use std::str::FromStr;

use super::*;

pub(crate) fn eval_url_encode(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [text] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("UrlEncode", args, 1));
    };
    let Some(text) = expr_as_string(text) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(Expr::string(url_encode(text)))
}

pub(crate) fn eval_url_decode(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [text] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("UrlDecode", args, 1));
    };
    let Some(text) = expr_as_string(text) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    match url_decode(text) {
        Ok(decoded) => Ok(Expr::string(decoded)),
        Err(message) => Ok(failed_with_message(session, "UrlDecode", "escape", message)),
    }
}

pub(crate) fn eval_url_parse(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [text] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("UrlParse", args, 1));
    };
    let Some(text) = expr_as_string(text) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    match parse_url(text) {
        Some(parsed) => Ok(parsed.into_expr()),
        None => Ok(failed_with_message(
            session,
            "UrlParse",
            "syntax",
            "UrlParse expects scheme://host[:port][/path][?query][#fragment].",
        )),
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ParsedUrl {
    scheme: String,
    host: String,
    port: Option<BigInt>,
    path: String,
    query: Option<String>,
    fragment: Option<String>,
}

impl ParsedUrl {
    fn into_expr(self) -> Expr {
        render_association(vec![
            (Expr::string("Scheme"), Expr::string(self.scheme)),
            (Expr::string("Host"), Expr::string(self.host)),
            (
                Expr::string("Port"),
                self.port
                    .map(Expr::integer)
                    .unwrap_or_else(|| system_expr("Null")),
            ),
            (Expr::string("Path"), Expr::string(self.path)),
            (
                Expr::string("Query"),
                self.query
                    .map(Expr::string)
                    .unwrap_or_else(|| system_expr("Null")),
            ),
            (
                Expr::string("Fragment"),
                self.fragment
                    .map(Expr::string)
                    .unwrap_or_else(|| system_expr("Null")),
            ),
        ])
    }
}

fn url_encode(text: &str) -> String {
    let mut output = String::new();
    for byte in text.as_bytes() {
        if is_unreserved(*byte) {
            output.push(*byte as char);
        } else {
            output.push_str(&format!("%{byte:02X}"));
        }
    }
    output
}

fn url_decode(text: &str) -> Result<String, String> {
    let bytes = text.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] != b'%' {
            decoded.push(bytes[index]);
            index += 1;
            continue;
        }
        let Some(hex) = bytes.get(index + 1..index + 3) else {
            return Err("UrlDecode found an incomplete percent escape.".to_string());
        };
        let value = decode_hex_byte(hex)
            .ok_or_else(|| "UrlDecode found an invalid percent escape.".to_string())?;
        decoded.push(value);
        index += 3;
    }
    String::from_utf8(decoded)
        .map_err(|_| "UrlDecode produced bytes that are not valid UTF-8.".to_string())
}

fn parse_url(text: &str) -> Option<ParsedUrl> {
    let (scheme, rest) = text.split_once("://")?;
    if !valid_scheme(scheme) {
        return None;
    }

    let host_end = rest
        .char_indices()
        .find_map(|(index, ch)| matches!(ch, ':' | '/' | '?' | '#').then_some(index))
        .unwrap_or(rest.len());
    let host = &rest[..host_end];
    if host.is_empty() {
        return None;
    }

    let mut tail = &rest[host_end..];
    let port = if let Some(after_colon) = tail.strip_prefix(':') {
        let port_end = after_colon
            .char_indices()
            .find_map(|(index, ch)| matches!(ch, '/' | '?' | '#').then_some(index))
            .unwrap_or(after_colon.len());
        let port_text = &after_colon[..port_end];
        if port_text.is_empty() || !port_text.chars().all(|ch| ch.is_ascii_digit()) {
            return None;
        }
        tail = &after_colon[port_end..];
        Some(BigInt::from_str(port_text).ok()?)
    } else {
        None
    };

    let path = if let Some(after_slash) = tail.strip_prefix('/') {
        let path_end = after_slash
            .char_indices()
            .find_map(|(index, ch)| matches!(ch, '?' | '#').then_some(index))
            .unwrap_or(after_slash.len());
        let path = format!("/{}", &after_slash[..path_end]);
        tail = &after_slash[path_end..];
        path
    } else {
        String::new()
    };

    let query = if let Some(after_question) = tail.strip_prefix('?') {
        let query_end = after_question
            .char_indices()
            .find_map(|(index, ch)| (ch == '#').then_some(index))
            .unwrap_or(after_question.len());
        let query = after_question[..query_end].to_string();
        tail = &after_question[query_end..];
        Some(query)
    } else {
        None
    };

    let fragment = if let Some(after_hash) = tail.strip_prefix('#') {
        tail = "";
        Some(after_hash.to_string())
    } else {
        None
    };

    tail.is_empty().then_some(ParsedUrl {
        scheme: scheme.to_string(),
        host: host.to_string(),
        port,
        path,
        query,
        fragment,
    })
}

fn is_unreserved(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~')
}

fn decode_hex_byte(hex: &[u8]) -> Option<u8> {
    let high = hex_value(*hex.first()?)?;
    let low = hex_value(*hex.get(1)?)?;
    Some(high * 16 + low)
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn valid_scheme(scheme: &str) -> bool {
    let mut chars = scheme.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    first.is_ascii_alphabetic()
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '+' | '-' | '.'))
}
