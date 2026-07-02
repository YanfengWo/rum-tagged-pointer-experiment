use super::*;

mod fields;
mod messages;
mod status;

pub(in crate::transport) use messages::*;

#[cfg(test)]
mod tests {
    use super::{fields::*, messages::*, status::*, *};
    use crate::{KernelExecutionState, KernelLifecycleState, json_support::JsonValue};
    use std::collections::BTreeMap;

    fn object(fields: Vec<(&str, JsonValue)>) -> BTreeMap<String, JsonValue> {
        fields
            .into_iter()
            .map(|(key, value)| (key.to_string(), value))
            .collect()
    }

    fn string(value: &str) -> JsonValue {
        JsonValue::String(value.to_string())
    }

    fn number(value: &str) -> JsonValue {
        JsonValue::Number(value.to_string())
    }

    #[test]
    fn field_helpers_accept_supported_shapes_and_report_typed_errors() {
        let values = object(vec![
            ("name", string("kernel")),
            ("count", number("7")),
            ("flag", JsonValue::Bool(true)),
            ("maybe_text", JsonValue::Null),
            ("bad_count", number("-1")),
            (
                "strings",
                JsonValue::Array(vec![string("one"), string("two")]),
            ),
            (
                "metadata",
                JsonValue::Object(object(vec![
                    ("trusted", JsonValue::Bool(true)),
                    (
                        "tags",
                        JsonValue::Array(vec![string("parameters"), JsonValue::Null]),
                    ),
                ])),
            ),
        ]);

        assert_eq!(
            string_field(&values, "ctx", "name"),
            Ok("kernel".to_string())
        );
        assert_eq!(usize_field(&values, "ctx", "count"), Ok(7));
        assert_eq!(
            bool_field_with_default(&values, "ctx", "flag", false),
            Ok(true)
        );
        assert_eq!(
            bool_field_with_default(&values, "ctx", "missing_flag", false),
            Ok(false)
        );
        assert_eq!(
            optional_string_field(&values, "ctx", "maybe_text"),
            Ok(None)
        );
        assert_eq!(
            optional_u64_field(&values, "ctx", "missing_count"),
            Ok(None)
        );
        assert_eq!(
            string_array_field(&values, "ctx", "strings"),
            Ok(vec!["one".to_string(), "two".to_string()])
        );

        let metadata = metadata_field(&values, "ctx", "metadata")
            .expect("structured metadata should convert scalar values to strings");
        assert_eq!(metadata.get("trusted"), Some("true"));
        assert_eq!(metadata.get("tags"), Some(r#"["parameters",null]"#));

        assert_eq!(
            string_field(&values, "ctx", "missing"),
            Err(KernelWireError::MissingField {
                context: "ctx",
                field: "missing",
            })
        );
        assert_eq!(
            string_field(&values, "ctx", "count"),
            Err(KernelWireError::InvalidField {
                context: "ctx",
                field: "count",
                value: "number".to_string(),
            })
        );
        assert_eq!(
            usize_field(&values, "ctx", "bad_count"),
            Err(KernelWireError::InvalidField {
                context: "ctx",
                field: "bad_count",
                value: "-1".to_string(),
            })
        );
        assert_eq!(
            bool_field_with_default(&values, "ctx", "name", true),
            Err(KernelWireError::InvalidField {
                context: "ctx",
                field: "name",
                value: "string".to_string(),
            })
        );
        assert_eq!(
            optional_u64_field(&values, "ctx", "bad_count"),
            Err(KernelWireError::InvalidField {
                context: "ctx",
                field: "bad_count",
                value: "-1".to_string(),
            })
        );
    }

    #[test]
    fn aggregate_field_helpers_parse_payloads_history_language_and_help_links() {
        let content = parse_object(
            r#"{
                "data":{"text/plain":"42","application/json":{"ok":true}},
                "user_expressions":{"expr":{"text/plain":"value"}},
                "payload":[{"source":"page","data":{"text/plain":"payload"},"metadata":{"n":2}}],
                "history":[[1,2,"In[2]"],[1,3,["In[3]","Out[3]"]]],
                "language_info":{
                    "name":"rum","version":"0.1","mimetype":"text/x-rum",
                    "file_extension":".wl","pygments_lexer":"mma","codemirror_mode":"mathematica"
                },
                "help_links":[{"text":"Docs","url":"https://example.invalid"}],
                "ename":"Failure","evalue":"bad","traceback":["rendered"]
            }"#,
            "content",
        )
        .expect("content object should parse");

        let bundle = mime_bundle_field(&content, "content", "data").expect("bundle should parse");
        assert_eq!(bundle.get("text/plain"), Some("42"));
        assert_eq!(bundle.get("application/json"), Some(r#"{"ok":true}"#));

        let expressions = mime_bundle_map_field(&content, "content", "user_expressions")
            .expect("user expressions should parse");
        assert_eq!(expressions["expr"].get("text/plain"), Some("value"));

        let payloads =
            execute_payloads_field(&content, "content", "payload").expect("payload should parse");
        assert_eq!(payloads.len(), 1);
        assert_eq!(payloads[0].source, "page");
        assert_eq!(payloads[0].metadata.get("n"), Some("2"));

        let history =
            history_entries_field(&content, "content", "history").expect("history should parse");
        assert_eq!(history[0].input, "In[2]");
        assert_eq!(history[0].output, None);
        assert_eq!(history[1].output.as_deref(), Some("Out[3]"));

        let info = language_info_field(&content, "content", "language_info")
            .expect("language info should parse");
        assert_eq!(info.name, "rum");
        assert_eq!(info.codemirror_mode, "mathematica");

        let links =
            help_links_field(&content, "content", "help_links").expect("help links should parse");
        assert_eq!(links[0].text, "Docs");
        assert_eq!(links[0].url, "https://example.invalid");

        let error = parse_error_fields(&content, "content").expect("error fields should parse");
        assert_eq!(error.error_name, "Failure");
        assert_eq!(error.rendered.as_deref(), Some("rendered"));

        assert_eq!(
            execute_payloads_field(&BTreeMap::new(), "content", "payload"),
            Ok(Vec::new())
        );
        assert_eq!(
            help_links_field(&BTreeMap::new(), "content", "help_links"),
            Ok(Vec::new())
        );
    }

    #[test]
    fn aggregate_field_helpers_reject_malformed_nested_shapes() {
        let bad_payload = parse_object(r#"{"payload":[true]}"#, "content").unwrap();
        assert_eq!(
            execute_payloads_field(&bad_payload, "content", "payload"),
            Err(KernelWireError::InvalidField {
                context: "content",
                field: "payload",
                value: "bool".to_string(),
            })
        );

        let bad_history = parse_object(r#"{"history":[[1,"line","input"]]}"#, "content").unwrap();
        assert_eq!(
            history_entries_field(&bad_history, "content", "history"),
            Err(KernelWireError::InvalidField {
                context: "content",
                field: "history",
                value: "string".to_string(),
            })
        );

        let bad_user_expressions =
            parse_object(r#"{"user_expressions":{"expr":false}}"#, "content").unwrap();
        assert_eq!(
            mime_bundle_map_field(&bad_user_expressions, "content", "user_expressions"),
            Err(KernelWireError::InvalidField {
                context: "content",
                field: "user_expressions",
                value: "bool".to_string(),
            })
        );

        assert_eq!(
            parse_object("[1]", "content"),
            Err(KernelWireError::InvalidField {
                context: "content",
                field: "<root>",
                value: "array".to_string(),
            })
        );
        assert_eq!(
            parse_object(r#"{"unterminated":"#, "content"),
            Err(KernelWireError::InvalidJson {
                message: "unexpected end of input".to_string(),
            })
        );
    }

    #[test]
    fn status_parsers_cover_all_known_values_and_error_fields() {
        assert_eq!(parse_reply_status("ok"), Ok(KernelReplyStatus::Ok));
        assert_eq!(parse_reply_status("error"), Ok(KernelReplyStatus::Error));
        assert_eq!(parse_reply_status("abort"), Ok(KernelReplyStatus::Abort));
        assert_eq!(
            parse_reply_status("paused"),
            Err(KernelWireError::InvalidField {
                context: "content",
                field: "status",
                value: "paused".to_string(),
            })
        );

        assert_eq!(
            parse_is_complete_status("complete"),
            Ok(KernelIsCompleteStatus::Complete)
        );
        assert_eq!(
            parse_is_complete_status("incomplete"),
            Ok(KernelIsCompleteStatus::Incomplete)
        );
        assert_eq!(
            parse_is_complete_status("invalid"),
            Ok(KernelIsCompleteStatus::Invalid)
        );
        assert_eq!(
            parse_is_complete_status("unknown"),
            Ok(KernelIsCompleteStatus::Unknown)
        );
        assert_eq!(
            parse_is_complete_status("waiting"),
            Err(KernelWireError::InvalidField {
                context: "content",
                field: "status",
                value: "waiting".to_string(),
            })
        );

        assert_eq!(
            parse_execution_state("starting"),
            Ok(KernelExecutionState::Starting)
        );
        assert_eq!(
            parse_execution_state("busy"),
            Ok(KernelExecutionState::Busy)
        );
        assert_eq!(
            parse_execution_state("idle"),
            Ok(KernelExecutionState::Idle)
        );
        assert_eq!(
            parse_execution_state("sleeping"),
            Err(KernelWireError::InvalidField {
                context: "content",
                field: "execution_state",
                value: "sleeping".to_string(),
            })
        );

        assert_eq!(
            parse_lifecycle_state("running"),
            Ok(KernelLifecycleState::Running)
        );
        assert_eq!(
            parse_lifecycle_state("interrupt_requested"),
            Ok(KernelLifecycleState::InterruptRequested)
        );
        assert_eq!(
            parse_lifecycle_state("shutdown_requested"),
            Ok(KernelLifecycleState::ShutdownRequested)
        );
        assert_eq!(
            parse_lifecycle_state("shutdown"),
            Ok(KernelLifecycleState::Shutdown)
        );
        assert_eq!(
            parse_lifecycle_state("restarting"),
            Err(KernelWireError::InvalidField {
                context: "content",
                field: "lifecycle_state",
                value: "restarting".to_string(),
            })
        );

        assert_eq!(parse_stream_name("stdout"), Ok(KernelStreamName::Stdout));
        assert_eq!(parse_stream_name("stderr"), Ok(KernelStreamName::Stderr));
        assert_eq!(
            parse_stream_name("display"),
            Err(KernelWireError::InvalidField {
                context: "content",
                field: "name",
                value: "display".to_string(),
            })
        );
    }

    #[test]
    fn message_header_parsing_defaults_date_and_reports_field_errors() {
        let header = parse_header(
            r#"{"msg_id":"m1","session":"s1","username":"u","msg_type":"execute_request","version":"5.3"}"#,
        )
        .expect("header should parse");
        assert_eq!(header.msg_id, "m1");
        assert_eq!(header.date, "1970-01-01T00:00:00Z");

        assert_eq!(
            parse_optional_header("{}").expect("empty parent header should parse"),
            None
        );
        let parent = parse_optional_header(
            r#"{"msg_id":"p1","session":"s1","username":"u","msg_type":"execute_request","version":"5.3","date":"2026-07-02T00:00:00Z"}"#,
        )
        .expect("parent header should parse")
        .expect("parent header should be present");
        assert_eq!(parent.date, "2026-07-02T00:00:00Z");

        assert_eq!(
            parse_header(r#"{"session":"s1"}"#),
            Err(KernelWireError::MissingField {
                context: "header",
                field: "msg_id",
            })
        );
        assert_eq!(
            parse_optional_header(r#"{"msg_id":1}"#),
            Err(KernelWireError::InvalidField {
                context: "parent_header",
                field: "msg_id",
                value: "number".to_string(),
            })
        );
    }

    #[test]
    fn request_content_parsing_covers_defaults_known_kinds_and_ignored_messages() {
        match parse_request_content("execute_request", r#"{"code":"1+1"}"#).unwrap() {
            KernelRequest::Execute(request) => {
                assert_eq!(request.code, "1+1");
                assert!(request.store_history);
                assert!(request.allow_stdin);
            }
            other => panic!("expected execute request, got {other:?}"),
        }

        match parse_request_content("complete_request", r#"{"code":"System`","cursor_pos":7}"#)
            .unwrap()
        {
            KernelRequest::Complete(request) => assert_eq!(request.cursor_offset, 7),
            other => panic!("expected complete request, got {other:?}"),
        }

        match parse_request_content("comm_info_request", r#"{"target_name":null}"#).unwrap() {
            KernelRequest::CommInfo(request) => assert_eq!(request.target_name, None),
            other => panic!("expected comm_info request, got {other:?}"),
        }

        match parse_request_content("debug_request", r#"{"seq":5,"command":"threads"}"#).unwrap() {
            KernelRequest::Debug(request) => {
                assert_eq!(request.request_seq, Some(5));
                assert_eq!(request.command.as_deref(), Some("threads"));
            }
            other => panic!("expected debug request, got {other:?}"),
        }

        match parse_request_content("shutdown_request", "{}").unwrap() {
            KernelRequest::Shutdown(request) => assert!(!request.restart),
            other => panic!("expected shutdown request, got {other:?}"),
        }

        match parse_request_content("comm_open", r#"{"comm_id":"c"}"#).unwrap() {
            KernelRequest::Ignored(request) => assert_eq!(request.msg_type, "comm_open"),
            other => panic!("expected ignored request, got {other:?}"),
        }

        assert_eq!(
            parse_request_content("inspect_request", r#"{"code":"x","cursor_pos":"bad"}"#),
            Err(KernelWireError::InvalidField {
                context: "content",
                field: "cursor_pos",
                value: "string".to_string(),
            })
        );
        assert_eq!(
            parse_input_request_content("input_request", r#"{"prompt":"? "}"#).unwrap(),
            KernelInputRequest {
                prompt: "? ".to_string(),
                password: false,
            }
        );
        assert_eq!(
            parse_input_reply_content("input_reply", r#"{"value":"answer"}"#).unwrap(),
            KernelInputReply {
                value: "answer".to_string(),
            }
        );
        assert_eq!(
            parse_input_reply_content("input_request", r#"{"value":"answer"}"#),
            Err(KernelWireError::UnsupportedMessageType {
                msg_type: "input_request".to_string(),
            })
        );
    }

    #[test]
    fn response_content_parsing_covers_reply_variants_and_malformed_statuses() {
        let mut metadata = KernelMetadata::new();
        metadata.insert("target", "symbol");
        metadata.insert("prefix", "Sy");
        metadata.insert("status", "ok");

        match parse_response_content(
            "execute_reply",
            r#"{
                "status":"error","execution_count":9,
                "payload":[{"source":"page","data":{"text/plain":"payload"},"metadata":{}}],
                "user_expressions":{"expr":{"text/plain":"value"}},
                "ename":"Failure","evalue":"bad","traceback":["rendered"]
            }"#,
            &metadata,
        )
        .unwrap()
        {
            KernelResponse::Execute(response) => {
                assert_eq!(response.status, KernelReplyStatus::Error);
                assert_eq!(response.execution_count, Some(9));
                assert_eq!(response.payload[0].source, "page");
                assert_eq!(
                    response.user_expressions["expr"].get("text/plain"),
                    Some("value")
                );
                assert_eq!(
                    response.error.unwrap().rendered.as_deref(),
                    Some("rendered")
                );
                assert_eq!(response.outcome, KernelExecuteOutcome::WireDecoded);
            }
            other => panic!("expected execute response, got {other:?}"),
        }

        match parse_response_content(
            "inspect_reply",
            r#"{"status":"ok","found":true,"data":{"text/plain":"doc"},"metadata":{"scope":"global"}}"#,
            &metadata,
        )
        .unwrap()
        {
            KernelResponse::Inspect(response) => {
                assert_eq!(response.target.as_deref(), Some("symbol"));
                assert!(response.found);
                assert_eq!(response.data.get("text/plain"), Some("doc"));
            }
            other => panic!("expected inspect response, got {other:?}"),
        }

        match parse_response_content(
            "complete_reply",
            r#"{"status":"ok","cursor_start":0,"cursor_end":2,"matches":["System"],"metadata":{}}"#,
            &metadata,
        )
        .unwrap()
        {
            KernelResponse::Complete(response) => {
                assert_eq!(response.prefix, "Sy");
                assert_eq!(response.matches, vec!["System".to_string()]);
            }
            other => panic!("expected complete response, got {other:?}"),
        }

        match parse_response_content(
            "is_complete_reply",
            r#"{"status":"incomplete","indent":"  "}"#,
            &metadata,
        )
        .unwrap()
        {
            KernelResponse::IsComplete(response) => {
                assert_eq!(response.reply_status, KernelReplyStatus::Ok);
                assert_eq!(
                    response.completion_status,
                    KernelIsCompleteStatus::Incomplete
                );
                assert_eq!(response.indent.as_deref(), Some("  "));
            }
            other => panic!("expected is_complete response, got {other:?}"),
        }

        match parse_response_content(
            "kernel_info_reply",
            r#"{
                "status":"ok","protocol_version":"5.3","implementation":"rum",
                "implementation_version":"0.1","banner":"Rum",
                "language_info":{
                    "name":"rum","version":"0.1","mimetype":"text/x-rum",
                    "file_extension":".wl","pygments_lexer":"mma","codemirror_mode":"mathematica"
                },
                "help_links":[{"text":"Docs","url":"https://example.invalid"}]
            }"#,
            &metadata,
        )
        .unwrap()
        {
            KernelResponse::KernelInfo(response) => {
                assert_eq!(response.implementation, "rum");
                assert_eq!(response.help_links[0].text, "Docs");
            }
            other => panic!("expected kernel-info response, got {other:?}"),
        }

        match parse_response_content("debug_reply", r#"{"message":"unsupported"}"#, &metadata)
            .unwrap()
        {
            KernelResponse::Debug(response) => {
                assert_eq!(response.status, KernelReplyStatus::Error);
                assert_eq!(response.message, "unsupported");
            }
            other => panic!("expected debug response, got {other:?}"),
        }

        assert_eq!(
            parse_response_content("execute_reply", r#"{"status":"bad"}"#, &metadata),
            Err(KernelWireError::InvalidField {
                context: "content",
                field: "status",
                value: "bad".to_string(),
            })
        );
        assert_eq!(
            parse_response_content("unknown_reply", "{}", &metadata),
            Err(KernelWireError::UnsupportedMessageType {
                msg_type: "unknown_reply".to_string(),
            })
        );
    }

    #[test]
    fn iopub_content_parsing_covers_payload_variants_and_errors() {
        let metadata = KernelMetadata::new();

        match parse_iopub_content(
            "status",
            r#"{"execution_state":"starting","lifecycle_state":"running"}"#,
            &metadata,
        )
        .unwrap()
        {
            KernelIoPubPayload::Status(status) => {
                assert_eq!(status.execution_state, KernelExecutionState::Starting);
                assert_eq!(status.lifecycle_state, KernelLifecycleState::Running);
            }
            other => panic!("expected status output, got {other:?}"),
        }

        match parse_iopub_content(
            "execute_input",
            r#"{"execution_count":null,"code":"x"}"#,
            &metadata,
        )
        .unwrap()
        {
            KernelIoPubPayload::ExecuteInput(input) => {
                assert_eq!(input.execution_count, None);
                assert_eq!(input.code, "x");
            }
            other => panic!("expected execute_input, got {other:?}"),
        }

        match parse_iopub_content("stream", r#"{"name":"stderr","text":"err"}"#, &metadata).unwrap()
        {
            KernelIoPubPayload::Stream(stream) => {
                assert_eq!(stream.name, KernelStreamName::Stderr);
                assert_eq!(stream.text, "err");
            }
            other => panic!("expected stream output, got {other:?}"),
        }

        match parse_iopub_content(
            "display_data",
            r#"{"data":{"text/plain":"shown"},"metadata":{}}"#,
            &metadata,
        )
        .unwrap()
        {
            KernelIoPubPayload::DisplayData(display) => {
                assert_eq!(display.data.get("text/plain"), Some("shown"));
            }
            other => panic!("expected display_data output, got {other:?}"),
        }

        match parse_iopub_content(
            "error",
            r#"{"ename":"Failure","evalue":"bad","traceback":["rendered"]}"#,
            &metadata,
        )
        .unwrap()
        {
            KernelIoPubPayload::Error(error) => {
                assert_eq!(error.error_name, "Failure");
                assert_eq!(error.rendered.as_deref(), Some("rendered"));
            }
            other => panic!("expected error output, got {other:?}"),
        }

        assert_eq!(
            parse_iopub_content("stream", r#"{"name":"display","text":"x"}"#, &metadata),
            Err(KernelWireError::InvalidField {
                context: "content",
                field: "name",
                value: "display".to_string(),
            })
        );
        assert_eq!(
            parse_iopub_content("unknown", "{}", &metadata),
            Err(KernelWireError::UnsupportedMessageType {
                msg_type: "unknown".to_string(),
            })
        );
    }
}
