//! Typed Dun Plugin Protocol messages over the minimal JSON codec.

use std::fmt;

use crate::json::{self, JsonValue};

/// Dun Plugin Protocol version implemented by this host.
pub const PROTOCOL_VERSION: i64 = 0;

/// Stable identifier emitted in this host's outbound envelopes.
pub const HOST_ID: &str = "rum";

const SYNTAX_HIGHLIGHT_ROLE: &str = "syntax-highlight";

/// A validated inbound message understood by the host loop.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Inbound {
    /// Initial protocol handshake.
    Hello,
    /// A syntax-highlighting request.
    Request {
        /// Correlation identifier to echo in the response.
        request_id: i64,
        /// Buffer revision to echo in the response.
        revision: i64,
        /// Language hint supplied by dun.
        language: String,
        /// Absolute line number of the first snapshot line.
        first_line: i64,
        /// Ordered snapshot lines to highlight.
        lines: Vec<String>,
    },
    /// Graceful host shutdown.
    Shutdown,
    /// Cancellation of an in-flight request.
    CancelRequest {
        /// Identifier of the request to cancel.
        request_id: i64,
    },
    /// A valid envelope whose message kind is not handled by this host.
    Unknown {
        /// Correlation identifier to echo in an error.
        request_id: i64,
        /// Unrecognized message kind.
        kind: String,
    },
}

/// A syntax-highlight span in character-column coordinates.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Span {
    /// Absolute line number.
    pub line: i64,
    /// Inclusive starting character column.
    pub start_col: i64,
    /// Exclusive ending character column.
    pub end_col: i64,
    /// Dun style vocabulary identifier.
    pub style: String,
}

/// Trust class advertised during the handshake.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Trust {
    /// The runtime exposes no file, process, network, terminal, environment,
    /// editor-state, clock, or entropy authority.
    PureSandbox,
    /// The host is an ordinary external process trusted by explicit user
    /// configuration.
    UserTrustedExternal,
}

impl Trust {
    /// Returns the trust class's wire identifier.
    pub const fn as_wire(self) -> &'static str {
        match self {
            Self::PureSandbox => "pure-sandbox",
            Self::UserTrustedExternal => "user-trusted-external",
        }
    }
}

/// A malformed or unsupported protocol envelope.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProtocolError {
    /// The frame payload is not UTF-8.
    InvalidUtf8(std::str::Utf8Error),
    /// The frame payload is malformed JSON.
    InvalidJson(json::JsonError),
    /// The top-level JSON value is not an object.
    ExpectedEnvelopeObject,
    /// A required envelope or payload field is absent.
    MissingField(&'static str),
    /// An envelope or payload field has the wrong type or value.
    InvalidField(&'static str),
    /// The protocol version is not supported.
    UnsupportedVersion(i64),
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidUtf8(error) => write!(formatter, "protocol payload is not UTF-8: {error}"),
            Self::InvalidJson(error) => write!(formatter, "{error}"),
            Self::ExpectedEnvelopeObject => {
                write!(formatter, "protocol envelope must be an object")
            }
            Self::MissingField(field) => write!(formatter, "missing protocol field '{field}'"),
            Self::InvalidField(field) => write!(formatter, "invalid protocol field '{field}'"),
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported protocol version {version}")
            }
        }
    }
}

impl std::error::Error for ProtocolError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::InvalidUtf8(error) => Some(error),
            Self::InvalidJson(error) => Some(error),
            _ => None,
        }
    }
}

/// Decodes and validates one inbound protocol envelope.
pub fn parse_envelope(payload: &[u8]) -> Result<Inbound, ProtocolError> {
    let text = std::str::from_utf8(payload).map_err(ProtocolError::InvalidUtf8)?;
    let envelope = json::decode(text).map_err(ProtocolError::InvalidJson)?;
    if envelope.as_object().is_none() {
        return Err(ProtocolError::ExpectedEnvelopeObject);
    }

    let version = required_i64(&envelope, "v")?;
    if version != PROTOCOL_VERSION {
        return Err(ProtocolError::UnsupportedVersion(version));
    }
    let kind = required_str(&envelope, "kind")?;
    let request_id = required_i64(&envelope, "request_id")?;
    let _plugin_id = required_str(&envelope, "plugin_id")?;
    let role = optional_str(&envelope, "role")?;
    let revision = optional_i64(&envelope, "revision")?;
    let payload = required_payload(&envelope)?;

    match kind {
        "hello" => Ok(Inbound::Hello),
        "request" => parse_request(request_id, role, revision, payload),
        "shutdown" => Ok(Inbound::Shutdown),
        "cancel-request" => Ok(Inbound::CancelRequest { request_id }),
        _ => Ok(Inbound::Unknown {
            request_id,
            kind: kind.to_owned(),
        }),
    }
}

/// Builds a complete `hello-ack` envelope.
pub fn hello_ack(host_id: &str, trust: Trust) -> JsonValue {
    envelope(
        "hello-ack",
        0,
        None,
        None,
        JsonValue::Object(vec![
            ("host_id".into(), JsonValue::Str(host_id.into())),
            ("trust".into(), JsonValue::Str(trust.as_wire().into())),
        ]),
    )
}

/// Builds a complete syntax-highlight `response` envelope.
pub fn response(request_id: i64, revision: i64, spans: &[Span]) -> JsonValue {
    let spans = spans
        .iter()
        .map(|span| {
            JsonValue::Object(vec![
                ("line".into(), JsonValue::Int(span.line)),
                ("start_col".into(), JsonValue::Int(span.start_col)),
                ("end_col".into(), JsonValue::Int(span.end_col)),
                ("style".into(), JsonValue::Str(span.style.clone())),
            ])
        })
        .collect();
    envelope(
        "response",
        request_id,
        Some(SYNTAX_HIGHLIGHT_ROLE),
        Some(revision),
        JsonValue::Object(vec![("spans".into(), JsonValue::Array(spans))]),
    )
}

/// Builds a complete protocol `error` envelope.
pub fn error(request_id: i64, message: &str) -> JsonValue {
    envelope(
        "error",
        request_id,
        None,
        None,
        JsonValue::Object(vec![("message".into(), JsonValue::Str(message.into()))]),
    )
}

fn parse_request(
    request_id: i64,
    role: Option<&str>,
    revision: Option<i64>,
    payload: &JsonValue,
) -> Result<Inbound, ProtocolError> {
    if role != Some(SYNTAX_HIGHLIGHT_ROLE) {
        return Err(match role {
            None => ProtocolError::MissingField("role"),
            Some(_) => ProtocolError::InvalidField("role"),
        });
    }
    let revision = revision.ok_or(ProtocolError::MissingField("revision"))?;
    if payload.as_object().is_none() {
        return Err(ProtocolError::InvalidField("payload"));
    }
    let language = required_str(payload, "language")?.to_owned();
    let first_line = required_i64(payload, "first_line")?;
    let line_values = required_field(payload, "lines")?
        .as_array()
        .ok_or(ProtocolError::InvalidField("lines"))?;
    let mut lines = Vec::with_capacity(line_values.len());
    for line in line_values {
        lines.push(
            line.as_str()
                .ok_or(ProtocolError::InvalidField("lines"))?
                .to_owned(),
        );
    }

    Ok(Inbound::Request {
        request_id,
        revision,
        language,
        first_line,
        lines,
    })
}

fn envelope(
    kind: &str,
    request_id: i64,
    role: Option<&str>,
    revision: Option<i64>,
    payload: JsonValue,
) -> JsonValue {
    let mut fields = vec![
        ("v".into(), JsonValue::Int(PROTOCOL_VERSION)),
        ("kind".into(), JsonValue::Str(kind.into())),
        ("request_id".into(), JsonValue::Int(request_id)),
        ("plugin_id".into(), JsonValue::Str(HOST_ID.into())),
    ];
    if let Some(role) = role {
        fields.push(("role".into(), JsonValue::Str(role.into())));
    }
    if let Some(revision) = revision {
        fields.push(("revision".into(), JsonValue::Int(revision)));
    }
    fields.push(("payload".into(), payload));
    JsonValue::Object(fields)
}

fn required_payload(envelope: &JsonValue) -> Result<&JsonValue, ProtocolError> {
    let payload = required_field(envelope, "payload")?;
    if matches!(payload, JsonValue::Null | JsonValue::Object(_)) {
        Ok(payload)
    } else {
        Err(ProtocolError::InvalidField("payload"))
    }
}

fn required_field<'a>(
    value: &'a JsonValue,
    field: &'static str,
) -> Result<&'a JsonValue, ProtocolError> {
    value.get(field).ok_or(ProtocolError::MissingField(field))
}

fn required_str<'a>(value: &'a JsonValue, field: &'static str) -> Result<&'a str, ProtocolError> {
    required_field(value, field)?
        .as_str()
        .ok_or(ProtocolError::InvalidField(field))
}

fn required_i64(value: &JsonValue, field: &'static str) -> Result<i64, ProtocolError> {
    required_field(value, field)?
        .as_i64()
        .ok_or(ProtocolError::InvalidField(field))
}

fn optional_str<'a>(
    value: &'a JsonValue,
    field: &'static str,
) -> Result<Option<&'a str>, ProtocolError> {
    value
        .get(field)
        .map(|field_value| {
            field_value
                .as_str()
                .ok_or(ProtocolError::InvalidField(field))
        })
        .transpose()
}

fn optional_i64(value: &JsonValue, field: &'static str) -> Result<Option<i64>, ProtocolError> {
    value
        .get(field)
        .map(|field_value| {
            field_value
                .as_i64()
                .ok_or(ProtocolError::InvalidField(field))
        })
        .transpose()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CHECKER_HELLO: &[u8] =
        br#"{"v":0,"kind":"hello","request_id":0,"plugin_id":"","payload":{"host":"dun"}}"#;
    const CHECKER_REQUEST: &[u8] = br#"{"v":0,"kind":"request","request_id":7,"plugin_id":"check","role":"syntax-highlight","revision":41,"payload":{"language":"rs","first_line":10,"lines":["fn main() { return 42; } // done",""]}}"#;

    #[test]
    fn parses_checker_hello_envelope() {
        assert_eq!(parse_envelope(CHECKER_HELLO).unwrap(), Inbound::Hello);
    }

    #[test]
    fn parses_checker_request_envelope() {
        assert_eq!(
            parse_envelope(CHECKER_REQUEST).unwrap(),
            Inbound::Request {
                request_id: 7,
                revision: 41,
                language: "rs".into(),
                first_line: 10,
                lines: vec!["fn main() { return 42; } // done".into(), String::new()],
            }
        );
    }

    #[test]
    fn parses_shutdown_cancel_and_unknown_envelopes() {
        assert_eq!(
            parse_envelope(
                br#"{"v":0,"kind":"shutdown","request_id":0,"plugin_id":"","payload":null}"#
            )
            .unwrap(),
            Inbound::Shutdown
        );
        assert_eq!(
            parse_envelope(
                br#"{"v":0,"kind":"cancel-request","request_id":7,"plugin_id":"check","payload":null}"#
            )
            .unwrap(),
            Inbound::CancelRequest { request_id: 7 }
        );
        assert_eq!(
            parse_envelope(
                br#"{"v":0,"kind":"diagnostic","request_id":9,"plugin_id":"check","payload":null}"#
            )
            .unwrap(),
            Inbound::Unknown {
                request_id: 9,
                kind: "diagnostic".into(),
            }
        );
    }

    #[test]
    fn builds_hello_ack_in_protocol_order() {
        assert_eq!(
            json::encode(&hello_ack(HOST_ID, Trust::PureSandbox)),
            r#"{"v":0,"kind":"hello-ack","request_id":0,"plugin_id":"rum","payload":{"host_id":"rum","trust":"pure-sandbox"}}"#
        );
        assert_eq!(
            Trust::UserTrustedExternal.as_wire(),
            "user-trusted-external"
        );
    }

    #[test]
    fn builds_response_with_echoed_request_and_revision() {
        let spans = [Span {
            line: 10,
            start_col: 0,
            end_col: 2,
            style: "keyword".into(),
        }];

        assert_eq!(
            json::encode(&response(7, 41, &spans)),
            r#"{"v":0,"kind":"response","request_id":7,"plugin_id":"rum","role":"syntax-highlight","revision":41,"payload":{"spans":[{"line":10,"start_col":0,"end_col":2,"style":"keyword"}]}}"#
        );
    }

    #[test]
    fn builds_error_envelope() {
        assert_eq!(
            json::encode(&error(12, "unsupported message kind")),
            r#"{"v":0,"kind":"error","request_id":12,"plugin_id":"rum","payload":{"message":"unsupported message kind"}}"#
        );
    }

    #[test]
    fn rejects_envelope_missing_kind() {
        assert_eq!(
            parse_envelope(br#"{"v":0,"request_id":0,"plugin_id":"","payload":null}"#),
            Err(ProtocolError::MissingField("kind"))
        );
    }

    #[test]
    fn rejects_mistyped_request_fields() {
        assert_eq!(
            parse_envelope(
                br#"{"v":0,"kind":"request","request_id":7,"plugin_id":"check","role":"syntax-highlight","revision":41,"payload":{"language":"rs","first_line":10,"lines":[1]}}"#
            ),
            Err(ProtocolError::InvalidField("lines"))
        );
    }
}
