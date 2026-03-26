use std::{collections::BTreeMap, fs, path::Path};

use crate::json_support::{JsonValue, parse_json_object};
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
        match (&self.signature_scheme, self.key.is_empty()) {
            (_, true) | (KernelSignatureScheme::None, _) => KernelSignaturePolicy::Disabled,
            (KernelSignatureScheme::HmacSha256, false) => {
                KernelSignaturePolicy::Required(self.key.clone())
            }
            (KernelSignatureScheme::Other(_), false) => KernelSignaturePolicy::Disabled,
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

fn parse_object(input: &str) -> Result<BTreeMap<String, JsonValue>, KernelConnectionFileError> {
    parse_json_object(input).map_err(|message| KernelConnectionFileError::InvalidJson { message })
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
