use super::*;

#[test]
fn adapter_can_stage_stdin_requests_for_future_execute_integration() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);
    let parent_header =
        KernelMessageHeader::new("client-msg-67b", "session-14b", "tester", "execute_request");
    let execute = KernelExecuteRequest::new("futureInput[]");

    let request = adapter
        .request_stdin_for_execute(&parent_header, &execute, KernelInputRequest::new("name: "))
        .expect("stdin request should be allowed");

    assert!(adapter.awaiting_stdin());
    assert_eq!(request.channel, KernelChannel::Stdin);
    assert_eq!(request.header.msg_type, "input_request");
    assert_eq!(request.request.prompt, "name: ");
    assert!(!request.request.password);
    assert_eq!(request.metadata.get("allow_stdin"), Some("true"));
    assert_eq!(request.metadata.get("channel"), Some("stdin"));
}

#[test]
fn adapter_rejects_stdin_requests_when_execute_disallows_them() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);
    let parent_header =
        KernelMessageHeader::new("client-msg-67c", "session-14c", "tester", "execute_request");
    let execute = KernelExecuteRequest::new("futureInput[]").without_stdin();

    let error = adapter
        .request_stdin_for_execute(&parent_header, &execute, KernelInputRequest::new("name: "))
        .expect_err("stdin request should be rejected");
    assert_eq!(error, crate::KernelStdinError::StdinNotAllowed);
    assert!(!adapter.awaiting_stdin());
}

#[test]
fn execute_request_with_inputstring_stages_stdin_and_defers_execute_reply() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);

    let dispatch = adapter.dispatch_request(KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new(
            "client-msg-input-1",
            "session-input-1",
            "tester",
            "execute_request",
        ),
        parent_header: None,
        metadata: super::KernelMetadata::new(),
        request: KernelRequest::Execute(KernelExecuteRequest::new("InputString[\"name: \"]")),
    });

    assert!(dispatch.reply.is_none());
    assert_eq!(dispatch.stdin.len(), 1);
    assert_eq!(dispatch.stdin[0].request.prompt, "name: ");
    assert_eq!(dispatch.iopub.len(), 2);
    assert!(adapter.awaiting_stdin());
    assert_eq!(adapter.execution_state(), KernelExecutionState::Busy);

    match &dispatch.iopub[0].payload {
        KernelIoPubPayload::Status(status) => {
            assert_eq!(status.execution_state, KernelExecutionState::Busy);
        }
        other => panic!("expected busy status payload, got {other:?}"),
    }

    match &dispatch.iopub[1].payload {
        KernelIoPubPayload::ExecuteInput(input) => {
            assert_eq!(input.code, "InputString[\"name: \"]");
        }
        other => panic!("expected execute_input payload, got {other:?}"),
    }
}

#[test]
fn execute_request_with_disallowed_stdin_returns_input_error_reply() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);

    let dispatch = adapter.dispatch_request(KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new(
            "client-msg-input-2",
            "session-input-2",
            "tester",
            "execute_request",
        ),
        parent_header: None,
        metadata: super::KernelMetadata::new(),
        request: KernelRequest::Execute(KernelExecuteRequest::new("InputString[]").without_stdin()),
    });
    let reply = dispatch
        .reply
        .as_ref()
        .expect("disallowed stdin should fail immediately");

    assert!(dispatch.stdin.is_empty());
    assert_eq!(dispatch.iopub.len(), 4);
    assert_eq!(reply.metadata.get("status"), Some("error"));

    match &reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(response.status, KernelReplyStatus::Error);
            assert!(matches!(
                response.outcome,
                KernelExecuteOutcome::InputRequired(_)
            ));
        }
        other => panic!("expected execute reply, got {other:?}"),
    }
}

#[test]
fn input_reply_resumes_execute_and_preserves_shell_routing_ids() {
    let codec = KernelWireCodec::new(KernelWireConfig::default().with_session_id("session-15c"));
    let adapter = KernelAdapter::new(Session::new());
    let mut pump = KernelEventPump::new(adapter, codec.clone());
    let mut transport = MockSocketTransport::default();

    let mut request = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: KernelMessageHeader::new(
                "client-msg-input-3",
                "session-15c",
                "tester",
                "execute_request",
            ),
            parent_header: None,
            metadata: super::KernelMetadata::new(),
            request: KernelRequest::Execute(KernelExecuteRequest::new("InputString[\"name: \"]")),
        })
        .expect("execute request should encode");
    request.routing_ids = vec![b"frontend".to_vec(), b"shell".to_vec()];
    transport.queue_incoming(request);

    let first_channel = pump
        .pump_next(&mut transport)
        .expect("first pump should succeed")
        .expect("one request should be processed");
    assert_eq!(first_channel, KernelChannel::Shell);
    assert!(transport.shell.is_empty());
    assert_eq!(transport.stdin.len(), 1);
    assert_eq!(transport.iopub.len(), 2);

    let input_request =
        decode_input_request_message(&transport.stdin[0]).expect("stdin request should decode");
    transport.queue_incoming(encode_input_reply_envelope(&KernelInputReplyEnvelope {
        channel: KernelChannel::Stdin,
        header: KernelMessageHeader::new(
            "client-msg-input-reply",
            "session-15c",
            "tester",
            "input_reply",
        ),
        parent_header: Some(input_request.header.clone()),
        metadata: super::KernelMetadata::new(),
        reply: KernelInputReply::new("Alice"),
    }));

    let second_channel = pump
        .pump_next(&mut transport)
        .expect("second pump should succeed")
        .expect("one request should be processed");
    assert_eq!(second_channel, KernelChannel::Stdin);
    assert_eq!(
        pump.adapter_mut()
            .take_last_input_reply()
            .map(|reply| reply.value),
        Some("Alice".to_string())
    );

    assert_eq!(transport.shell.len(), 1);
    assert_eq!(
        transport.shell[0].routing_ids,
        vec![b"frontend".to_vec(), b"shell".to_vec()]
    );
    assert_eq!(transport.iopub.len(), 4);

    let reply = decode_response_message(&transport.shell[0]).expect("reply should decode");
    match reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(response.execution_count, Some(1));
        }
        other => panic!("expected execute reply, got {other:?}"),
    }

    let execute_result =
        decode_iopub_message(&transport.iopub[2]).expect("execute_result should decode");
    match execute_result.payload {
        KernelIoPubPayload::ExecuteResult(result) => {
            assert_eq!(result.data.get("text/plain"), Some("\"Alice\""));
        }
        other => panic!("expected execute_result payload, got {other:?}"),
    }

    let idle_status = decode_iopub_message(&transport.iopub[3]).expect("idle status should decode");
    match idle_status.payload {
        KernelIoPubPayload::Status(status) => {
            assert_eq!(status.execution_state, KernelExecutionState::Idle);
        }
        other => panic!("expected status payload, got {other:?}"),
    }
}

#[test]
fn input_reply_without_parent_header_resumes_pending_execute() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);

    let dispatch = adapter.dispatch_request(KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new(
            "client-msg-input-3b",
            "session-input-3b",
            "tester",
            "execute_request",
        ),
        parent_header: None,
        metadata: super::KernelMetadata::new(),
        request: KernelRequest::Execute(KernelExecuteRequest::new("InputString[\"name: \"]")),
    });

    assert!(dispatch.reply.is_none());
    assert_eq!(dispatch.stdin.len(), 1);
    assert!(adapter.awaiting_stdin());

    let resumed = adapter
        .handle_input_reply(KernelInputReplyEnvelope {
            channel: KernelChannel::Stdin,
            header: KernelMessageHeader::new(
                "client-msg-input-reply-3b",
                "session-input-3b",
                "tester",
                "input_reply",
            ),
            parent_header: None,
            metadata: super::KernelMetadata::new(),
            reply: KernelInputReply::new("Alice"),
        })
        .expect("parent-less jupyter_client input_reply should be accepted")
        .expect("stdin reply should resume the suspended execute");

    let reply = resumed
        .reply
        .as_ref()
        .expect("resumed execute should now have a reply");

    match &reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(response.execution_count, Some(1));
            assert_eq!(response.outputs.len(), 1);
            match &response.outputs[0] {
                KernelOutput::ExecuteResult(result) => {
                    assert_eq!(result.data.get("text/plain"), Some("\"Alice\""));
                }
                other => panic!("expected execute result output, got {other:?}"),
            }
        }
        other => panic!("expected execute reply, got {other:?}"),
    }
}

#[test]
fn input_reply_resumes_execute_without_rerunning_prefix_side_effects() {
    let mut session = Session::new();
    eval_text(&mut session, "x = seed");
    let mut adapter = KernelAdapter::new(session);

    let dispatch = adapter.dispatch_request(KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new(
            "client-msg-input-4",
            "session-input-4",
            "tester",
            "execute_request",
        ),
        parent_header: None,
        metadata: super::KernelMetadata::new(),
        request: KernelRequest::Execute(KernelExecuteRequest::new(
            "List[x = List[x], InputString[\"name: \"], x = List[x]]",
        )),
    });

    assert!(dispatch.reply.is_none());
    assert_eq!(dispatch.stdin.len(), 1);

    let resumed = adapter
        .handle_input_reply(KernelInputReplyEnvelope {
            channel: KernelChannel::Stdin,
            header: KernelMessageHeader::new(
                "client-msg-input-reply-2",
                "session-input-4",
                "tester",
                "input_reply",
            ),
            parent_header: Some(dispatch.stdin[0].header.clone()),
            metadata: super::KernelMetadata::new(),
            reply: KernelInputReply::new("Alice"),
        })
        .expect("stdin reply should be accepted")
        .expect("stdin reply should resume the suspended execute");

    let reply = resumed
        .reply
        .as_ref()
        .expect("resumed execute should now have a reply");

    match &reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(response.execution_count, Some(1));
            assert_eq!(response.outputs.len(), 1);
            match &response.outputs[0] {
                KernelOutput::ExecuteResult(result) => {
                    assert_eq!(
                        result.data.get("text/plain"),
                        Some("List[List[seed], \"Alice\", List[List[seed]]]")
                    );
                }
                other => panic!("expected execute result output, got {other:?}"),
            }
        }
        other => panic!("expected execute reply, got {other:?}"),
    }

    let check = adapter.execute(KernelExecuteRequest::new("x").without_history());
    match check.outcome {
        KernelExecuteOutcome::Success(success) => {
            assert_eq!(success.value.text, "List[List[seed]]");
        }
        other => panic!("expected x check to succeed, got {other:?}"),
    }
}

#[test]
fn input_reply_resumes_delayed_message_templates_through_internal_string_driver() {
    let mut session = Session::new();
    eval_text(&mut session, "f::argx := InputString[\"msg: \"]");
    let mut adapter = KernelAdapter::new(session);

    let dispatch = adapter.dispatch_request(KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new(
            "client-msg-input-5",
            "session-input-5",
            "tester",
            "execute_request",
        ),
        parent_header: None,
        metadata: super::KernelMetadata::new(),
        request: KernelRequest::Execute(KernelExecuteRequest::new("Message[f::argx]")),
    });

    assert!(dispatch.reply.is_none());
    assert_eq!(dispatch.stdin.len(), 1);
    assert_eq!(dispatch.stdin[0].request.prompt, "msg: ");

    let resumed = adapter
        .handle_input_reply(KernelInputReplyEnvelope {
            channel: KernelChannel::Stdin,
            header: KernelMessageHeader::new(
                "client-msg-input-reply-3",
                "session-input-5",
                "tester",
                "input_reply",
            ),
            parent_header: Some(dispatch.stdin[0].header.clone()),
            metadata: super::KernelMetadata::new(),
            reply: KernelInputReply::new("Alice"),
        })
        .expect("stdin reply should be accepted")
        .expect("stdin reply should resume the suspended execute");

    let reply = resumed
        .reply
        .as_ref()
        .expect("resumed execute should now have a reply");

    match &reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(response.execution_count, Some(1));
            assert!(response.outputs.iter().any(|output| matches!(
                output,
                KernelOutput::Stream(stream)
                    if stream.text.contains("f::argx: Alice")
            )));
            assert!(response.outputs.iter().any(|output| matches!(
                output,
                KernelOutput::ExecuteResult(result)
                    if result.data.get("text/plain") == Some("Null")
            )));
        }
        other => panic!("expected execute reply, got {other:?}"),
    }
}

#[test]
fn input_reply_resumes_multi_sequence_matcher_backtracking_across_multiple_prompts() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);

    let dispatch = adapter.dispatch_request(KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new(
            "client-msg-input-6",
            "session-input-6",
            "tester",
            "execute_request",
        ),
        parent_header: None,
        metadata: super::KernelMetadata::new(),
        request: KernelRequest::Execute(KernelExecuteRequest::new(
            "MatchQ[f[1, 2, 3], f[x__ /; MatchQ[InputString[\"first: \"], \"yes\"], y__ /; MatchQ[InputString[\"second: \"], \"ok\"]]]",
        )),
    });

    assert!(dispatch.reply.is_none());
    assert_eq!(dispatch.stdin.len(), 1);
    assert_eq!(dispatch.stdin[0].request.prompt, "first: ");

    let second = adapter
        .handle_input_reply(KernelInputReplyEnvelope {
            channel: KernelChannel::Stdin,
            header: KernelMessageHeader::new(
                "client-msg-input-reply-4",
                "session-input-6",
                "tester",
                "input_reply",
            ),
            parent_header: Some(dispatch.stdin[0].header.clone()),
            metadata: super::KernelMetadata::new(),
            reply: KernelInputReply::new("yes"),
        })
        .expect("first stdin reply should be accepted")
        .expect("first stdin reply should stage the next matcher prompt");

    assert!(second.reply.is_none());
    assert_eq!(second.stdin.len(), 1);
    assert_eq!(second.stdin[0].request.prompt, "second: ");

    let third = adapter
        .handle_input_reply(KernelInputReplyEnvelope {
            channel: KernelChannel::Stdin,
            header: KernelMessageHeader::new(
                "client-msg-input-reply-5",
                "session-input-6",
                "tester",
                "input_reply",
            ),
            parent_header: Some(second.stdin[0].header.clone()),
            metadata: super::KernelMetadata::new(),
            reply: KernelInputReply::new("ok"),
        })
        .expect("second stdin reply should be accepted")
        .expect("backtracking should be able to stage the repeated matcher prompt");

    assert!(third.reply.is_none());
    assert_eq!(third.stdin.len(), 1);
    assert_eq!(third.stdin[0].request.prompt, "second: ");

    let resumed = adapter
        .handle_input_reply(KernelInputReplyEnvelope {
            channel: KernelChannel::Stdin,
            header: KernelMessageHeader::new(
                "client-msg-input-reply-6",
                "session-input-6",
                "tester",
                "input_reply",
            ),
            parent_header: Some(third.stdin[0].header.clone()),
            metadata: super::KernelMetadata::new(),
            reply: KernelInputReply::new("ok"),
        })
        .expect("third stdin reply should be accepted")
        .expect("third stdin reply should finally complete the execute");

    let reply = resumed
        .reply
        .as_ref()
        .expect("resumed execute should now have a reply");

    match &reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(response.execution_count, Some(1));
            assert!(response.outputs.iter().any(|output| matches!(
                output,
                KernelOutput::ExecuteResult(result)
                    if result.data.get("text/plain") == Some("True")
            )));
        }
        other => panic!("expected execute reply, got {other:?}"),
    }
}
