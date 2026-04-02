use super::*;

pub(in crate::transport) fn encode_request_content(request: &KernelRequest) -> String {
    match request {
        KernelRequest::Execute(KernelExecuteRequest {
            code,
            store_history,
            allow_stdin,
        }) => json_object(vec![
            field("code", json_string(code)),
            field("store_history", json_bool(*store_history)),
            field("allow_stdin", json_bool(*allow_stdin)),
        ]),
        KernelRequest::Inspect(KernelInspectRequest {
            code,
            cursor_offset,
        }) => json_object(vec![
            field("code", json_string(code)),
            field("cursor_pos", json_usize(*cursor_offset)),
        ]),
        KernelRequest::Complete(KernelCompleteRequest {
            code,
            cursor_offset,
        }) => json_object(vec![
            field("code", json_string(code)),
            field("cursor_pos", json_usize(*cursor_offset)),
        ]),
        KernelRequest::IsComplete(KernelIsCompleteRequest { code }) => {
            json_object(vec![field("code", json_string(code))])
        }
        KernelRequest::KernelInfo(_) | KernelRequest::Interrupt(_) => json_object(Vec::new()),
        KernelRequest::Shutdown(KernelShutdownRequest { restart }) => {
            json_object(vec![field("restart", json_bool(*restart))])
        }
    }
}

pub(in crate::transport) fn encode_input_request_content(request: &KernelInputRequest) -> String {
    json_object(vec![
        field("prompt", json_string(&request.prompt)),
        field("password", json_bool(request.password)),
    ])
}

pub(in crate::transport) fn encode_input_reply_content(reply: &KernelInputReply) -> String {
    json_object(vec![field("value", json_string(&reply.value))])
}

pub(in crate::transport) fn encode_response_content(response: &KernelResponse) -> String {
    match response {
        KernelResponse::Execute(response) => encode_execute_response(response),
        KernelResponse::Inspect(response) => encode_inspect_response(response),
        KernelResponse::Complete(response) => encode_complete_response(response),
        KernelResponse::IsComplete(response) => encode_is_complete_response(response),
        KernelResponse::KernelInfo(response) => encode_kernel_info_response(response),
        KernelResponse::Interrupt(response) => encode_interrupt_response(response),
        KernelResponse::Shutdown(response) => encode_shutdown_response(response),
    }
}

pub(in crate::transport) fn encode_iopub_content(payload: &KernelIoPubPayload) -> String {
    match payload {
        KernelIoPubPayload::Status(status) => encode_status_output(status),
        KernelIoPubPayload::ExecuteInput(input) => encode_execute_input(input),
        KernelIoPubPayload::Stream(stream) => encode_stream_output(stream),
        KernelIoPubPayload::ExecuteResult(result) => encode_execute_result(result),
        KernelIoPubPayload::DisplayData(display) => encode_display_output(display),
        KernelIoPubPayload::Error(error) => encode_error_output(error),
    }
}

pub(in crate::transport) fn encode_header(header: &KernelMessageHeader) -> String {
    json_object(vec![
        field("msg_id", json_string(&header.msg_id)),
        field("session", json_string(&header.session_id)),
        field("username", json_string(&header.username)),
        field("msg_type", json_string(&header.msg_type)),
        field("version", json_string(&header.protocol_version)),
        field("date", json_string(&header.date)),
    ])
}

pub(in crate::transport) fn encode_optional_header(header: Option<&KernelMessageHeader>) -> String {
    header
        .map(encode_header)
        .unwrap_or_else(|| "{}".to_string())
}

pub(in crate::transport) fn encode_execute_response(response: &KernelExecuteResponse) -> String {
    let mut fields = vec![
        field("status", json_string(response.status.as_str())),
        field("execution_count", json_option_u64(response.execution_count)),
        field(
            "payload",
            json_array(
                response
                    .payload
                    .iter()
                    .map(encode_execute_payload)
                    .collect(),
            ),
        ),
        field(
            "user_expressions",
            json_object(
                response
                    .user_expressions
                    .iter()
                    .map(|(name, bundle)| field(name, encode_mime_bundle(bundle)))
                    .collect(),
            ),
        ),
    ];

    if let Some(error) = &response.error {
        fields.push(field("ename", json_string(&error.error_name)));
        fields.push(field("evalue", json_string(&error.error_value)));
        fields.push(field("traceback", encode_traceback(error)));
    }

    json_object(fields)
}

pub(in crate::transport) fn encode_inspect_response(response: &KernelInspectResponse) -> String {
    json_object(vec![
        field("status", json_string(response.status.as_str())),
        field("found", json_bool(response.found)),
        field("data", encode_mime_bundle(&response.data)),
        field("metadata", encode_metadata(&response.metadata)),
    ])
}

pub(in crate::transport) fn encode_complete_response(response: &KernelCompleteResponse) -> String {
    json_object(vec![
        field("status", json_string(response.status.as_str())),
        field(
            "matches",
            json_array(
                response
                    .matches
                    .iter()
                    .map(|value| json_string(value))
                    .collect(),
            ),
        ),
        field("cursor_start", json_usize(response.replacement_start)),
        field("cursor_end", json_usize(response.replacement_end)),
        field("metadata", encode_metadata(&response.metadata)),
    ])
}

pub(in crate::transport) fn encode_is_complete_response(
    response: &KernelIsCompleteResponse,
) -> String {
    let mut fields = vec![field(
        "status",
        json_string(response.completion_status.as_str()),
    )];

    if let Some(indent) = &response.indent {
        fields.push(field("indent", json_string(indent)));
    }

    json_object(fields)
}

pub(in crate::transport) fn encode_kernel_info_response(response: &KernelInfoResponse) -> String {
    json_object(vec![
        field("status", json_string(response.status.as_str())),
        field("protocol_version", json_string(&response.protocol_version)),
        field("implementation", json_string(&response.implementation)),
        field(
            "implementation_version",
            json_string(&response.implementation_version),
        ),
        field(
            "language_info",
            encode_language_info(&response.language_info),
        ),
        field("banner", json_string(&response.banner)),
        field(
            "help_links",
            json_array(response.help_links.iter().map(encode_help_link).collect()),
        ),
    ])
}

pub(in crate::transport) fn encode_interrupt_response(
    response: &KernelInterruptResponse,
) -> String {
    json_object(vec![
        field("status", json_string(response.status.as_str())),
        field(
            "interrupt_requested",
            json_bool(response.interrupt_requested),
        ),
        field(
            "lifecycle_state",
            json_string(response.lifecycle_state.as_str()),
        ),
    ])
}

pub(in crate::transport) fn encode_shutdown_response(response: &KernelShutdownResponse) -> String {
    json_object(vec![
        field("status", json_string(response.status.as_str())),
        field("restart", json_bool(response.restart)),
        field(
            "lifecycle_state",
            json_string(response.lifecycle_state.as_str()),
        ),
    ])
}

pub(in crate::transport) fn encode_status_output(output: &KernelStatusOutput) -> String {
    json_object(vec![
        field(
            "execution_state",
            json_string(output.execution_state.as_str()),
        ),
        field(
            "lifecycle_state",
            json_string(output.lifecycle_state.as_str()),
        ),
    ])
}

pub(in crate::transport) fn encode_execute_input(input: &KernelExecuteInput) -> String {
    json_object(vec![
        field("code", json_string(&input.code)),
        field("execution_count", json_option_u64(input.execution_count)),
    ])
}

pub(in crate::transport) fn encode_stream_output(stream: &KernelStreamOutput) -> String {
    json_object(vec![
        field(
            "name",
            json_string(match stream.name {
                KernelStreamName::Stdout => "stdout",
                KernelStreamName::Stderr => "stderr",
            }),
        ),
        field("text", json_string(&stream.text)),
    ])
}

pub(in crate::transport) fn encode_execute_result(result: &KernelExecuteResult) -> String {
    json_object(vec![
        field("execution_count", json_option_u64(result.execution_count)),
        field("data", encode_mime_bundle(&result.data)),
        field("metadata", encode_metadata(&result.metadata)),
    ])
}

pub(in crate::transport) fn encode_display_output(display: &KernelDisplayOutput) -> String {
    json_object(vec![
        field("data", encode_mime_bundle(&display.data)),
        field("metadata", encode_metadata(&display.metadata)),
    ])
}

pub(in crate::transport) fn encode_error_output(error: &KernelErrorOutput) -> String {
    json_object(vec![
        field("ename", json_string(&error.error_name)),
        field("evalue", json_string(&error.error_value)),
        field("traceback", encode_traceback(error)),
    ])
}

pub(in crate::transport) fn encode_execute_payload(payload: &KernelExecutePayload) -> String {
    json_object(vec![
        field("source", json_string(&payload.source)),
        field("data", encode_mime_bundle(&payload.data)),
        field("metadata", encode_metadata(&payload.metadata)),
    ])
}

pub(in crate::transport) fn encode_language_info(info: &KernelLanguageInfo) -> String {
    json_object(vec![
        field("name", json_string(&info.name)),
        field("version", json_string(&info.version)),
        field("mimetype", json_string(&info.mimetype)),
        field("file_extension", json_string(&info.file_extension)),
        field("pygments_lexer", json_string(&info.pygments_lexer)),
        field("codemirror_mode", json_string(&info.codemirror_mode)),
    ])
}

pub(in crate::transport) fn encode_help_link(link: &KernelHelpLink) -> String {
    json_object(vec![
        field("text", json_string(&link.text)),
        field("url", json_string(&link.url)),
    ])
}

pub(in crate::transport) fn encode_metadata(metadata: &KernelMetadata) -> String {
    json_object(
        metadata
            .items
            .iter()
            .map(|(key, value)| field(key, json_string(value)))
            .collect(),
    )
}

pub(in crate::transport) fn encode_mime_bundle(bundle: &KernelMimeBundle) -> String {
    json_object(
        bundle
            .items
            .iter()
            .map(|(key, value)| field(key, json_string(value)))
            .collect(),
    )
}

pub(in crate::transport) fn encode_traceback(error: &KernelErrorOutput) -> String {
    let traceback = error
        .rendered
        .as_deref()
        .unwrap_or(error.error_value.as_str());
    json_array(vec![json_string(traceback)])
}

pub(in crate::transport) fn field(name: &str, value: String) -> (String, String) {
    (name.to_string(), value)
}

pub(in crate::transport) fn json_object(fields: Vec<(String, String)>) -> String {
    let parts = fields
        .into_iter()
        .map(|(key, value)| format!("{}:{}", json_string(&key), value))
        .collect::<Vec<_>>()
        .join(",");
    format!("{{{parts}}}")
}

pub(in crate::transport) fn json_array(items: Vec<String>) -> String {
    format!("[{}]", items.join(","))
}

pub(in crate::transport) fn json_string(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len() + 2);
    escaped.push('"');

    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            ch if ch.is_control() => escaped.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => escaped.push(ch),
        }
    }

    escaped.push('"');
    escaped
}

pub(in crate::transport) fn json_bool(value: bool) -> String {
    if value {
        "true".to_string()
    } else {
        "false".to_string()
    }
}

pub(in crate::transport) fn json_usize(value: usize) -> String {
    value.to_string()
}

pub(in crate::transport) fn json_option_u64(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "null".to_string())
}
