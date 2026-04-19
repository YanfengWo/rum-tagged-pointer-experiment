use super::*;

pub(super) fn input_request_from_outcome(
    outcome: &KernelExecuteOutcome,
) -> Option<RuntimeInputRequest> {
    match outcome {
        KernelExecuteOutcome::Failure(SessionRequestFailure {
            error: SessionRequestError::InputRequired { request },
            ..
        }) => Some(request.clone()),
        _ => None,
    }
}

pub(super) fn kernel_input_request(request: &RuntimeInputRequest) -> KernelInputRequest {
    let mut input = KernelInputRequest::new(&request.prompt);
    if request.password {
        input = input.password();
    }
    input
}

pub(super) fn build_execute_response(
    execution_count: Option<u64>,
    outcome: KernelExecuteOutcome,
) -> KernelExecuteResponse {
    let outputs = build_execute_outputs(execution_count, &outcome);
    let error = match &outcome {
        KernelExecuteOutcome::Success(_) => None,
        KernelExecuteOutcome::Failure(failure) => Some(error_output(failure)),
        KernelExecuteOutcome::WireDecoded => None,
    };

    KernelExecuteResponse {
        status: if matches!(outcome, KernelExecuteOutcome::Success(_)) {
            KernelReplyStatus::Ok
        } else {
            KernelReplyStatus::Error
        },
        execution_count,
        payload: Vec::<KernelExecutePayload>::new(),
        user_expressions: Default::default(),
        error,
        outputs,
        outcome,
    }
}

fn build_execute_outputs(
    execution_count: Option<u64>,
    outcome: &KernelExecuteOutcome,
) -> Vec<KernelOutput> {
    match outcome {
        KernelExecuteOutcome::Success(response) => {
            let mut outputs = runtime_message_streams(&response.messages);
            if !response.trace.is_empty() {
                outputs.push(KernelOutput::DisplayData(KernelDisplayOutput {
                    data: trace_mime_bundle(&response.trace),
                    metadata: trace_metadata(),
                }));
            }
            outputs.push(KernelOutput::ExecuteResult(KernelExecuteResult {
                execution_count,
                data: expr_mime_bundle(&response.value.to_string()),
                metadata: execute_result_metadata(execution_count),
            }));
            outputs
        }
        KernelExecuteOutcome::Failure(failure) => {
            let mut outputs = runtime_message_streams(&failure.messages);
            if !failure.trace.is_empty() {
                outputs.push(KernelOutput::DisplayData(KernelDisplayOutput {
                    data: trace_mime_bundle(&failure.trace),
                    metadata: trace_metadata(),
                }));
            }
            outputs.push(KernelOutput::Error(error_output(failure)));
            outputs
        }
        KernelExecuteOutcome::WireDecoded => Vec::new(),
    }
}

pub(super) fn build_suspended_outputs(outcome: &KernelExecuteOutcome) -> Vec<KernelOutput> {
    match outcome {
        KernelExecuteOutcome::Failure(failure)
            if matches!(failure.error, SessionRequestError::InputRequired { .. }) =>
        {
            let mut outputs = runtime_message_streams(&failure.messages);
            if !failure.trace.is_empty() {
                outputs.push(KernelOutput::DisplayData(KernelDisplayOutput {
                    data: trace_mime_bundle(&failure.trace),
                    metadata: trace_metadata(),
                }));
            }
            outputs
        }
        _ => Vec::new(),
    }
}

fn runtime_message_streams(messages: &[RuntimeMessage]) -> Vec<KernelOutput> {
    messages
        .iter()
        .map(|message| {
            KernelOutput::Stream(KernelStreamOutput {
                name: KernelStreamName::Stderr,
                text: format!("{message}\n"),
            })
        })
        .collect()
}

fn error_output(failure: &SessionRequestFailure) -> KernelErrorOutput {
    let (error_name, rendered) = match &failure.error {
        SessionRequestError::Syntax(diagnostic) => {
            ("Syntax".to_string(), Some(diagnostic.rendered.clone()))
        }
        SessionRequestError::InvalidInteger { .. } => ("InvalidInteger".to_string(), None),
        SessionRequestError::Io { .. } => ("Io".to_string(), None),
        SessionRequestError::LoadPathNotFound { .. } => ("LoadPathNotFound".to_string(), None),
        SessionRequestError::EvaluationCycleDetected { .. } => {
            ("EvaluationCycleDetected".to_string(), None)
        }
        SessionRequestError::RecursionLimitExceeded { .. } => {
            ("RecursionLimitExceeded".to_string(), None)
        }
        SessionRequestError::TimeConstraintExceeded => ("TimeConstraintExceeded".to_string(), None),
        SessionRequestError::MemoryConstraintExceeded { .. } => {
            ("MemoryConstraintExceeded".to_string(), None)
        }
        SessionRequestError::IterationLimitExceeded { .. } => {
            ("IterationLimitExceeded".to_string(), None)
        }
        SessionRequestError::EvaluationPassLimitExceeded { .. } => {
            ("EvaluationPassLimitExceeded".to_string(), None)
        }
        SessionRequestError::InputRequired { .. } => ("InputRequired".to_string(), None),
        SessionRequestError::Other { .. } => ("Other".to_string(), None),
    };

    KernelErrorOutput {
        error_name,
        error_value: failure.error.to_string(),
        rendered,
    }
}

fn expr_mime_bundle(rendered_expr: &str) -> KernelMimeBundle {
    let mut bundle = KernelMimeBundle::text_plain(rendered_expr);
    bundle.insert("text/markdown", format!("```rum\n{rendered_expr}\n```"));
    bundle.insert("application/vnd.rum.expr", rendered_expr);
    bundle
}

fn trace_mime_bundle(trace: &[TraceEvent]) -> KernelMimeBundle {
    let rendered = trace
        .iter()
        .map(render_trace_event)
        .collect::<Vec<_>>()
        .join("\n");

    let mut bundle = KernelMimeBundle::text_plain(&rendered);
    bundle.insert("text/markdown", format!("```text\n{rendered}\n```"));
    bundle.insert("application/vnd.rum.trace", rendered);
    bundle
}

pub(super) fn inspect_mime_bundle(response: &SessionInspectResponse) -> KernelMimeBundle {
    let plain = render_inspect_text(response);
    let markdown = render_inspect_markdown(response);

    let mut bundle = KernelMimeBundle::text_plain(&plain);
    bundle.insert("text/markdown", markdown);
    bundle.insert("application/vnd.rum.inspect", plain);
    bundle
}

fn execute_result_metadata(execution_count: Option<u64>) -> KernelMetadata {
    let mut metadata = KernelMetadata::new();
    metadata.insert("output_type", "execute_result");
    metadata.insert("bundle_kind", "expr");
    metadata.insert(
        "execution_count",
        execution_count
            .map(|count| count.to_string())
            .unwrap_or_else(|| "none".to_string()),
    );
    metadata
}

fn trace_metadata() -> KernelMetadata {
    let mut metadata = KernelMetadata::new();
    metadata.insert("output_type", "display_data");
    metadata.insert("bundle_kind", "trace");
    metadata
}

#[cfg(test)]
mod tests {
    use rum_runtime::SessionRequestFailure;

    use super::*;

    #[test]
    fn error_output_covers_time_constraint_failures() {
        let outcome = KernelExecuteOutcome::Failure(SessionRequestFailure {
            error: SessionRequestError::TimeConstraintExceeded,
            messages: Vec::new(),
            prints: Vec::new(),
            trace: Vec::new(),
        });

        let response = build_execute_response(Some(1), outcome);

        assert_eq!(response.status, KernelReplyStatus::Error);
        assert_eq!(response.outputs.len(), 1);
        assert_eq!(
            response
                .error
                .as_ref()
                .map(|error| error.error_name.as_str()),
            Some("TimeConstraintExceeded")
        );

        match &response.outputs[0] {
            KernelOutput::Error(output) => {
                assert_eq!(output.error_name, "TimeConstraintExceeded");
                assert!(
                    output.error_value.contains("TimeConstrained limit"),
                    "unexpected error text: {}",
                    output.error_value
                );
            }
            other => panic!("expected error output, got {other:?}"),
        }
    }

    #[test]
    fn error_output_covers_memory_constraint_failures() {
        let outcome = KernelExecuteOutcome::Failure(SessionRequestFailure {
            error: SessionRequestError::MemoryConstraintExceeded { limit: 20 },
            messages: Vec::new(),
            prints: Vec::new(),
            trace: Vec::new(),
        });

        let response = build_execute_response(Some(1), outcome);

        assert_eq!(response.status, KernelReplyStatus::Error);
        assert_eq!(response.outputs.len(), 1);
        assert_eq!(
            response
                .error
                .as_ref()
                .map(|error| error.error_name.as_str()),
            Some("MemoryConstraintExceeded")
        );

        match &response.outputs[0] {
            KernelOutput::Error(output) => {
                assert_eq!(output.error_name, "MemoryConstraintExceeded");
                assert!(
                    output.error_value.contains("MemoryConstrained limit (20)"),
                    "unexpected error text: {}",
                    output.error_value
                );
            }
            other => panic!("expected error output, got {other:?}"),
        }
    }

    #[test]
    fn render_rule_application_source_covers_replace() {
        assert_eq!(
            render_rule_application_source(RuleApplicationSource::Replace),
            "replace"
        );
        assert_eq!(
            render_rule_application_source(RuleApplicationSource::ReplaceList),
            "replace-list"
        );
        assert_eq!(
            render_rule_application_source(RuleApplicationSource::ReplaceAt),
            "replace-at"
        );
    }
}

pub(super) fn inspect_metadata(target: Option<&str>, found: bool) -> KernelMetadata {
    let mut metadata = KernelMetadata::new();
    metadata.insert("found", if found { "true" } else { "false" });
    metadata.insert(
        "target",
        target
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| "<none>".to_string()),
    );
    metadata.insert("bundle_kind", "inspect");
    metadata
}

pub(super) fn response_msg_type(response: &KernelResponse) -> &'static str {
    match response {
        KernelResponse::Execute(_) => "execute_reply",
        KernelResponse::Inspect(_) => "inspect_reply",
        KernelResponse::Complete(_) => "complete_reply",
        KernelResponse::IsComplete(_) => "is_complete_reply",
        KernelResponse::KernelInfo(_) => "kernel_info_reply",
        KernelResponse::Interrupt(_) => "interrupt_reply",
        KernelResponse::Shutdown(_) => "shutdown_reply",
    }
}

pub(super) fn response_metadata(
    response: &KernelResponse,
    channel: KernelChannel,
    parent_header: &KernelMessageHeader,
    execution_state: KernelExecutionState,
    lifecycle_state: KernelLifecycleState,
) -> KernelMetadata {
    let mut metadata = KernelMetadata::new();
    metadata.insert("msg_type", response_msg_type(response));
    metadata.insert("channel", channel.as_str());
    metadata.insert("execution_state", execution_state.as_str());
    metadata.insert("lifecycle_state", lifecycle_state.as_str());
    metadata.insert("parent_msg_type", &parent_header.msg_type);
    metadata.insert("protocol_version", &parent_header.protocol_version);

    let status = match response {
        KernelResponse::Execute(execute) => execute.status.as_str(),
        KernelResponse::Inspect(inspect) => inspect.status.as_str(),
        KernelResponse::Complete(complete) => complete.status.as_str(),
        KernelResponse::IsComplete(response) => response.reply_status.as_str(),
        KernelResponse::KernelInfo(info) => info.status.as_str(),
        KernelResponse::Interrupt(interrupt) => interrupt.status.as_str(),
        KernelResponse::Shutdown(shutdown) => shutdown.status.as_str(),
    };

    metadata.insert("status", status);

    if let KernelResponse::IsComplete(response) = response {
        metadata.insert("completion_status", response.completion_status.as_str());
    }

    metadata
}

pub(super) fn complete_metadata(
    prefix: &str,
    replacement_start: usize,
    replacement_end: usize,
) -> KernelMetadata {
    let mut metadata = KernelMetadata::new();
    metadata.insert("cursor_start", replacement_start.to_string());
    metadata.insert("cursor_end", replacement_end.to_string());
    metadata.insert("prefix", prefix);
    metadata
}

pub(super) fn input_request_metadata(
    parent_header: &KernelMessageHeader,
    request: &KernelExecuteRequest,
    input: &KernelInputRequest,
) -> KernelMetadata {
    let mut metadata = KernelMetadata::new();
    metadata.insert("msg_type", "input_request");
    metadata.insert("channel", KernelChannel::Stdin.as_str());
    metadata.insert("parent_msg_type", &parent_header.msg_type);
    metadata.insert(
        "allow_stdin",
        if request.allow_stdin { "true" } else { "false" },
    );
    metadata.insert("password", if input.password { "true" } else { "false" });
    metadata
}

fn render_inspect_text(response: &SessionInspectResponse) -> String {
    let info = &response.info;
    let mut lines = vec![
        format!("{} ({})", info.qualified_name, info.context),
        format!(
            "attributes: {}",
            if info.attributes.is_empty() {
                "<none>".to_string()
            } else {
                info.attributes.join(", ")
            }
        ),
        format!(
            "definitions: own={} delayed={} up={} down={} sub={}",
            info.has_own_value,
            info.has_delayed_value,
            info.has_up_values,
            info.has_down_values,
            info.has_sub_values
        ),
        format!(
            "messages: {}",
            if info.message_tags.is_empty() {
                "<none>".to_string()
            } else {
                info.message_tags.join(", ")
            }
        ),
    ];

    if let Some(summary) = &info.summary {
        lines.insert(1, format!("summary: {summary}"));
    }

    lines.join("\n")
}

fn render_inspect_markdown(response: &SessionInspectResponse) -> String {
    let info = &response.info;
    let attributes = if info.attributes.is_empty() {
        "<none>".to_string()
    } else {
        info.attributes.join(", ")
    };
    let messages = if info.message_tags.is_empty() {
        "<none>".to_string()
    } else {
        info.message_tags.join(", ")
    };

    format!(
        "### `{}`\n\n- Context: `{}`\n- Attributes: {}\n- Definitions: own=`{}`, delayed=`{}`, up=`{}`, down=`{}`, sub=`{}`\n- Messages: {}",
        info.qualified_name,
        info.context,
        attributes,
        info.has_own_value,
        info.has_delayed_value,
        info.has_up_values,
        info.has_down_values,
        info.has_sub_values,
        messages
    )
}

fn render_trace_event(event: &TraceEvent) -> String {
    match event {
        TraceEvent::EvaluationPass { before, after } => {
            format!("evaluation-pass: {before} => {after}")
        }
        TraceEvent::DefinitionStored {
            symbol,
            kind,
            value,
        } => format!("definition-stored: {symbol} [{kind:?}] = {value}"),
        TraceEvent::RuleApplied {
            source,
            input,
            rule,
            result,
        } => format!(
            "rule-applied: {} | input={input} | rule={rule} | result={result}",
            render_rule_application_source(*source)
        ),
    }
}

fn render_rule_application_source(source: RuleApplicationSource) -> &'static str {
    match source {
        RuleApplicationSource::UpValue => "upvalue",
        RuleApplicationSource::DownValue => "downvalue",
        RuleApplicationSource::SubValue => "subvalue",
        RuleApplicationSource::Replace => "replace",
        RuleApplicationSource::ReplaceList => "replace-list",
        RuleApplicationSource::ReplaceAt => "replace-at",
        RuleApplicationSource::ReplaceAll => "replace-all",
        RuleApplicationSource::ReplaceRepeated => "replace-repeated",
    }
}
