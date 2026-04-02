use super::*;

pub(in crate::transport) fn parse_header(
    input: &str,
) -> Result<KernelMessageHeader, KernelWireError> {
    let object = parse_object(input, "header")?;

    Ok(KernelMessageHeader {
        msg_id: string_field(&object, "header", "msg_id")?,
        session_id: string_field(&object, "header", "session")?,
        username: string_field(&object, "header", "username")?,
        msg_type: string_field(&object, "header", "msg_type")?,
        protocol_version: string_field(&object, "header", "version")?,
        date: optional_string_field(&object, "header", "date")?
            .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string()),
    })
}

pub(in crate::transport) fn parse_optional_header(
    input: &str,
) -> Result<Option<KernelMessageHeader>, KernelWireError> {
    let object = parse_object(input, "parent_header")?;
    if object.is_empty() {
        Ok(None)
    } else {
        Ok(Some(KernelMessageHeader {
            msg_id: string_field(&object, "parent_header", "msg_id")?,
            session_id: string_field(&object, "parent_header", "session")?,
            username: string_field(&object, "parent_header", "username")?,
            msg_type: string_field(&object, "parent_header", "msg_type")?,
            protocol_version: string_field(&object, "parent_header", "version")?,
            date: optional_string_field(&object, "parent_header", "date")?
                .unwrap_or_else(|| "1970-01-01T00:00:00Z".to_string()),
        }))
    }
}

pub(in crate::transport) fn parse_metadata(input: &str) -> Result<KernelMetadata, KernelWireError> {
    let object = parse_object(input, "metadata")?;
    let mut metadata = KernelMetadata::new();

    for (key, value) in object {
        metadata.insert(key, scalar_to_string("metadata", value)?);
    }

    Ok(metadata)
}

pub(in crate::transport) fn parse_request_content(
    msg_type: &str,
    input: &str,
) -> Result<KernelRequest, KernelWireError> {
    let object = parse_object(input, "content")?;

    match msg_type {
        "execute_request" => Ok(KernelRequest::Execute(KernelExecuteRequest {
            code: string_field(&object, "content", "code")?,
            store_history: bool_field_with_default(&object, "content", "store_history", true)?,
            allow_stdin: bool_field_with_default(&object, "content", "allow_stdin", true)?,
        })),
        "inspect_request" => Ok(KernelRequest::Inspect(KernelInspectRequest {
            code: string_field(&object, "content", "code")?,
            cursor_offset: usize_field(&object, "content", "cursor_pos")?,
        })),
        "complete_request" => Ok(KernelRequest::Complete(KernelCompleteRequest {
            code: string_field(&object, "content", "code")?,
            cursor_offset: usize_field(&object, "content", "cursor_pos")?,
        })),
        "is_complete_request" => Ok(KernelRequest::IsComplete(KernelIsCompleteRequest {
            code: string_field(&object, "content", "code")?,
        })),
        "kernel_info_request" => Ok(KernelRequest::KernelInfo(crate::KernelInfoRequest)),
        "interrupt_request" => Ok(KernelRequest::Interrupt(KernelInterruptRequest)),
        "shutdown_request" => Ok(KernelRequest::Shutdown(KernelShutdownRequest {
            restart: bool_field_with_default(&object, "content", "restart", false)?,
        })),
        _ => Err(KernelWireError::UnsupportedMessageType {
            msg_type: msg_type.to_string(),
        }),
    }
}

pub(in crate::transport) fn parse_input_request_content(
    msg_type: &str,
    input: &str,
) -> Result<KernelInputRequest, KernelWireError> {
    let object = parse_object(input, "content")?;

    match msg_type {
        "input_request" => Ok(KernelInputRequest {
            prompt: string_field(&object, "content", "prompt")?,
            password: bool_field_with_default(&object, "content", "password", false)?,
        }),
        _ => Err(KernelWireError::UnsupportedMessageType {
            msg_type: msg_type.to_string(),
        }),
    }
}

pub(in crate::transport) fn parse_input_reply_content(
    msg_type: &str,
    input: &str,
) -> Result<KernelInputReply, KernelWireError> {
    let object = parse_object(input, "content")?;

    match msg_type {
        "input_reply" => Ok(KernelInputReply {
            value: string_field(&object, "content", "value")?,
        }),
        _ => Err(KernelWireError::UnsupportedMessageType {
            msg_type: msg_type.to_string(),
        }),
    }
}

pub(in crate::transport) fn parse_response_content(
    msg_type: &str,
    input: &str,
    metadata: &KernelMetadata,
) -> Result<KernelResponse, KernelWireError> {
    let object = parse_object(input, "content")?;

    match msg_type {
        "execute_reply" => {
            let status = parse_reply_status(&string_field(&object, "content", "status")?)?;
            let execution_count = optional_u64_field(&object, "content", "execution_count")?;
            let payload = execute_payloads_field(&object, "content", "payload")?;
            let user_expressions = mime_bundle_map_field(&object, "content", "user_expressions")?;
            let error = if object.contains_key("ename") {
                Some(parse_error_fields(&object, "content")?)
            } else {
                None
            };

            Ok(KernelResponse::Execute(KernelExecuteResponse {
                status,
                execution_count,
                payload,
                user_expressions,
                error,
                outputs: Vec::new(),
                outcome: KernelExecuteOutcome::WireDecoded,
            }))
        }
        "inspect_reply" => Ok(KernelResponse::Inspect(KernelInspectResponse {
            status: parse_reply_status(&string_field(&object, "content", "status")?)?,
            target: metadata.get("target").map(ToOwned::to_owned),
            found: bool_field_with_default(&object, "content", "found", false)?,
            data: mime_bundle_field(&object, "content", "data")?,
            metadata: metadata_field(&object, "content", "metadata")?,
            result: None,
        })),
        "complete_reply" => Ok(KernelResponse::Complete(KernelCompleteResponse {
            status: parse_reply_status(&string_field(&object, "content", "status")?)?,
            replacement_start: usize_field(&object, "content", "cursor_start")?,
            replacement_end: usize_field(&object, "content", "cursor_end")?,
            matches: string_array_field(&object, "content", "matches")?,
            metadata: metadata_field(&object, "content", "metadata")?,
            prefix: metadata.get("prefix").unwrap_or_default().to_string(),
        })),
        "is_complete_reply" => Ok(KernelResponse::IsComplete(KernelIsCompleteResponse {
            reply_status: parse_reply_status(
                metadata
                    .get("status")
                    .unwrap_or(KernelReplyStatus::Ok.as_str()),
            )?,
            completion_status: parse_is_complete_status(&string_field(
                &object, "content", "status",
            )?)?,
            indent: optional_string_field(&object, "content", "indent")?,
        })),
        "kernel_info_reply" => Ok(KernelResponse::KernelInfo(KernelInfoResponse {
            status: parse_reply_status(&string_field(&object, "content", "status")?)?,
            protocol_version: string_field(&object, "content", "protocol_version")?,
            implementation: string_field(&object, "content", "implementation")?,
            implementation_version: string_field(&object, "content", "implementation_version")?,
            banner: string_field(&object, "content", "banner")?,
            language_info: language_info_field(&object, "content", "language_info")?,
            help_links: help_links_field(&object, "content", "help_links")?,
        })),
        "interrupt_reply" => Ok(KernelResponse::Interrupt(KernelInterruptResponse {
            status: parse_reply_status(&string_field(&object, "content", "status")?)?,
            interrupt_requested: bool_field_with_default(
                &object,
                "content",
                "interrupt_requested",
                false,
            )?,
            lifecycle_state: parse_lifecycle_state(&string_field(
                &object,
                "content",
                "lifecycle_state",
            )?)?,
        })),
        "shutdown_reply" => Ok(KernelResponse::Shutdown(KernelShutdownResponse {
            status: parse_reply_status(&string_field(&object, "content", "status")?)?,
            restart: bool_field_with_default(&object, "content", "restart", false)?,
            lifecycle_state: parse_lifecycle_state(&string_field(
                &object,
                "content",
                "lifecycle_state",
            )?)?,
        })),
        _ => Err(KernelWireError::UnsupportedMessageType {
            msg_type: msg_type.to_string(),
        }),
    }
}

pub(in crate::transport) fn parse_iopub_content(
    msg_type: &str,
    input: &str,
    _metadata: &KernelMetadata,
) -> Result<KernelIoPubPayload, KernelWireError> {
    let object = parse_object(input, "content")?;

    match msg_type {
        "status" => Ok(KernelIoPubPayload::Status(KernelStatusOutput {
            execution_state: parse_execution_state(&string_field(
                &object,
                "content",
                "execution_state",
            )?)?,
            lifecycle_state: parse_lifecycle_state(&string_field(
                &object,
                "content",
                "lifecycle_state",
            )?)?,
        })),
        "execute_input" => Ok(KernelIoPubPayload::ExecuteInput(KernelExecuteInput {
            execution_count: optional_u64_field(&object, "content", "execution_count")?,
            code: string_field(&object, "content", "code")?,
        })),
        "stream" => Ok(KernelIoPubPayload::Stream(KernelStreamOutput {
            name: parse_stream_name(&string_field(&object, "content", "name")?)?,
            text: string_field(&object, "content", "text")?,
        })),
        "execute_result" => Ok(KernelIoPubPayload::ExecuteResult(KernelExecuteResult {
            execution_count: optional_u64_field(&object, "content", "execution_count")?,
            data: mime_bundle_field(&object, "content", "data")?,
            metadata: metadata_field(&object, "content", "metadata")?,
        })),
        "display_data" => Ok(KernelIoPubPayload::DisplayData(KernelDisplayOutput {
            data: mime_bundle_field(&object, "content", "data")?,
            metadata: metadata_field(&object, "content", "metadata")?,
        })),
        "error" => Ok(KernelIoPubPayload::Error(parse_error_fields(
            &object, "content",
        )?)),
        _ => Err(KernelWireError::UnsupportedMessageType {
            msg_type: msg_type.to_string(),
        }),
    }
}

pub(in crate::transport) fn parse_object(
    input: &str,
    context: &'static str,
) -> Result<BTreeMap<String, JsonValue>, KernelWireError> {
    parse_json_object(input).map_err(|message| {
        if let Some(found) = message.strip_prefix("expected top-level object, found ") {
            KernelWireError::InvalidField {
                context,
                field: "<root>",
                value: found.to_string(),
            }
        } else {
            KernelWireError::InvalidJson { message }
        }
    })
}

pub(in crate::transport) fn string_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<String, KernelWireError> {
    match object.get(field) {
        Some(JsonValue::String(value)) => Ok(value.clone()),
        Some(other) => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
        None => Err(KernelWireError::MissingField { context, field }),
    }
}

pub(in crate::transport) fn usize_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<usize, KernelWireError> {
    match object.get(field) {
        Some(JsonValue::Number(value)) => {
            value
                .parse::<usize>()
                .map_err(|_| KernelWireError::InvalidField {
                    context,
                    field,
                    value: value.clone(),
                })
        }
        Some(other) => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
        None => Err(KernelWireError::MissingField { context, field }),
    }
}

pub(in crate::transport) fn bool_field_with_default(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
    default: bool,
) -> Result<bool, KernelWireError> {
    match object.get(field) {
        Some(JsonValue::Bool(value)) => Ok(*value),
        Some(other) => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
        None => Ok(default),
    }
}

pub(in crate::transport) fn scalar_to_string(
    context: &'static str,
    value: JsonValue,
) -> Result<String, KernelWireError> {
    match value {
        JsonValue::String(value) => Ok(value),
        JsonValue::Number(value) => Ok(value),
        JsonValue::Bool(value) => Ok(if value { "true" } else { "false" }.to_string()),
        JsonValue::Null => Ok("null".to_string()),
        JsonValue::Array(_) | JsonValue::Object(_) => Err(KernelWireError::InvalidField {
            context,
            field: "<scalar>",
            value: value.kind_name().to_string(),
        }),
    }
}

pub(in crate::transport) fn optional_string_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<Option<String>, KernelWireError> {
    match object.get(field) {
        Some(JsonValue::String(value)) => Ok(Some(value.clone())),
        Some(JsonValue::Null) => Ok(None),
        Some(other) => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
        None => Ok(None),
    }
}

pub(in crate::transport) fn optional_u64_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<Option<u64>, KernelWireError> {
    match object.get(field) {
        Some(JsonValue::Number(value)) => {
            value
                .parse::<u64>()
                .map(Some)
                .map_err(|_| KernelWireError::InvalidField {
                    context,
                    field,
                    value: value.clone(),
                })
        }
        Some(JsonValue::Null) => Ok(None),
        Some(other) => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
        None => Ok(None),
    }
}

pub(in crate::transport) fn string_array_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<Vec<String>, KernelWireError> {
    match object.get(field) {
        Some(JsonValue::Array(values)) => values
            .iter()
            .map(|value| match value {
                JsonValue::String(value) => Ok(value.clone()),
                other => Err(KernelWireError::InvalidField {
                    context,
                    field,
                    value: other.kind_name().to_string(),
                }),
            })
            .collect(),
        Some(other) => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
        None => Err(KernelWireError::MissingField { context, field }),
    }
}

pub(in crate::transport) fn metadata_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<KernelMetadata, KernelWireError> {
    let nested = nested_object_field(object, context, field)?;
    let mut metadata = KernelMetadata::new();
    for (key, value) in nested {
        metadata.insert(key, scalar_to_string(context, value)?);
    }
    Ok(metadata)
}

pub(in crate::transport) fn mime_bundle_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<KernelMimeBundle, KernelWireError> {
    let nested = nested_object_field(object, context, field)?;
    let mut bundle = KernelMimeBundle::new();
    for (key, value) in nested {
        bundle.insert(key, scalar_to_string(context, value)?);
    }
    Ok(bundle)
}

pub(in crate::transport) fn mime_bundle_map_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<BTreeMap<String, KernelMimeBundle>, KernelWireError> {
    let nested = nested_object_field(object, context, field)?;
    let mut result = BTreeMap::new();
    for (key, value) in nested {
        let inner = match value {
            JsonValue::Object(map) => map,
            other => {
                return Err(KernelWireError::InvalidField {
                    context,
                    field,
                    value: other.kind_name().to_string(),
                });
            }
        };
        let mut bundle = KernelMimeBundle::new();
        for (mime, value) in inner {
            bundle.insert(mime, scalar_to_string(context, value)?);
        }
        result.insert(key, bundle);
    }
    Ok(result)
}

pub(in crate::transport) fn execute_payloads_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<Vec<KernelExecutePayload>, KernelWireError> {
    match object.get(field) {
        Some(JsonValue::Array(values)) => values
            .iter()
            .map(|value| {
                let item = match value {
                    JsonValue::Object(map) => map,
                    other => {
                        return Err(KernelWireError::InvalidField {
                            context,
                            field,
                            value: other.kind_name().to_string(),
                        });
                    }
                };

                Ok(KernelExecutePayload {
                    source: string_field(item, context, "source")?,
                    data: mime_bundle_field(item, context, "data")?,
                    metadata: metadata_field(item, context, "metadata")?,
                })
            })
            .collect(),
        Some(other) => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
        None => Ok(Vec::new()),
    }
}

pub(in crate::transport) fn language_info_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<KernelLanguageInfo, KernelWireError> {
    let nested = nested_object_field(object, context, field)?;
    Ok(KernelLanguageInfo {
        name: string_field(&nested, context, "name")?,
        version: string_field(&nested, context, "version")?,
        mimetype: string_field(&nested, context, "mimetype")?,
        file_extension: string_field(&nested, context, "file_extension")?,
        pygments_lexer: string_field(&nested, context, "pygments_lexer")?,
        codemirror_mode: string_field(&nested, context, "codemirror_mode")?,
    })
}

pub(in crate::transport) fn help_links_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<Vec<KernelHelpLink>, KernelWireError> {
    match object.get(field) {
        Some(JsonValue::Array(values)) => values
            .iter()
            .map(|value| {
                let item = match value {
                    JsonValue::Object(map) => map,
                    other => {
                        return Err(KernelWireError::InvalidField {
                            context,
                            field,
                            value: other.kind_name().to_string(),
                        });
                    }
                };

                Ok(KernelHelpLink {
                    text: string_field(item, context, "text")?,
                    url: string_field(item, context, "url")?,
                })
            })
            .collect(),
        Some(other) => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
        None => Ok(Vec::new()),
    }
}

pub(in crate::transport) fn parse_error_fields(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
) -> Result<KernelErrorOutput, KernelWireError> {
    let rendered = match object.get("traceback") {
        Some(JsonValue::Array(values)) => values.first().and_then(|value| match value {
            JsonValue::String(value) => Some(value.clone()),
            _ => None,
        }),
        _ => None,
    };

    Ok(KernelErrorOutput {
        error_name: string_field(object, context, "ename")?,
        error_value: string_field(object, context, "evalue")?,
        rendered,
    })
}

pub(in crate::transport) fn nested_object_field(
    object: &BTreeMap<String, JsonValue>,
    context: &'static str,
    field: &'static str,
) -> Result<BTreeMap<String, JsonValue>, KernelWireError> {
    match object.get(field) {
        Some(JsonValue::Object(value)) => Ok(value.clone()),
        Some(other) => Err(KernelWireError::InvalidField {
            context,
            field,
            value: other.kind_name().to_string(),
        }),
        None => Err(KernelWireError::MissingField { context, field }),
    }
}

pub(in crate::transport) fn parse_reply_status(
    value: &str,
) -> Result<KernelReplyStatus, KernelWireError> {
    match value {
        "ok" => Ok(KernelReplyStatus::Ok),
        "error" => Ok(KernelReplyStatus::Error),
        "abort" => Ok(KernelReplyStatus::Abort),
        other => Err(KernelWireError::InvalidField {
            context: "content",
            field: "status",
            value: other.to_string(),
        }),
    }
}

pub(in crate::transport) fn parse_is_complete_status(
    value: &str,
) -> Result<KernelIsCompleteStatus, KernelWireError> {
    match value {
        "complete" => Ok(KernelIsCompleteStatus::Complete),
        "incomplete" => Ok(KernelIsCompleteStatus::Incomplete),
        "invalid" => Ok(KernelIsCompleteStatus::Invalid),
        "unknown" => Ok(KernelIsCompleteStatus::Unknown),
        other => Err(KernelWireError::InvalidField {
            context: "content",
            field: "status",
            value: other.to_string(),
        }),
    }
}

pub(in crate::transport) fn parse_execution_state(
    value: &str,
) -> Result<crate::KernelExecutionState, KernelWireError> {
    match value {
        "starting" => Ok(crate::KernelExecutionState::Starting),
        "busy" => Ok(crate::KernelExecutionState::Busy),
        "idle" => Ok(crate::KernelExecutionState::Idle),
        other => Err(KernelWireError::InvalidField {
            context: "content",
            field: "execution_state",
            value: other.to_string(),
        }),
    }
}

pub(in crate::transport) fn parse_lifecycle_state(
    value: &str,
) -> Result<crate::KernelLifecycleState, KernelWireError> {
    match value {
        "running" => Ok(crate::KernelLifecycleState::Running),
        "interrupt_requested" => Ok(crate::KernelLifecycleState::InterruptRequested),
        "shutdown_requested" => Ok(crate::KernelLifecycleState::ShutdownRequested),
        "shutdown" => Ok(crate::KernelLifecycleState::Shutdown),
        other => Err(KernelWireError::InvalidField {
            context: "content",
            field: "lifecycle_state",
            value: other.to_string(),
        }),
    }
}

pub(in crate::transport) fn parse_stream_name(
    value: &str,
) -> Result<KernelStreamName, KernelWireError> {
    match value {
        "stdout" => Ok(KernelStreamName::Stdout),
        "stderr" => Ok(KernelStreamName::Stderr),
        other => Err(KernelWireError::InvalidField {
            context: "content",
            field: "name",
            value: other.to_string(),
        }),
    }
}
