use super::*;

#[test]
fn dispatch_request_emits_iopub_status_execute_input_and_execute_result() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);

    let initial = adapter.dispatch_request(KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new("client-msg-20", "session-3", "tester", "execute_request"),
        parent_header: None,
        metadata: KernelMetadata::new(),
        request: KernelRequest::Execute(KernelExecuteRequest::new("1")),
    });
    assert!(initial.reply.is_none());
    assert_eq!(initial.iopub.len(), 2);
    assert_eq!(adapter.execution_state(), KernelExecutionState::Busy);
    let dispatch = finish_adapter_execute(&mut adapter, initial);
    let reply = dispatch
        .reply
        .as_ref()
        .expect("execute should reply after worker completion");

    assert_eq!(reply.channel, KernelChannel::Shell);
    assert_eq!(reply.header.msg_type, "execute_reply");
    assert_eq!(reply.metadata.get("status"), Some("ok"));
    assert_eq!(adapter.execution_state(), KernelExecutionState::Idle);
    assert_eq!(dispatch.iopub.len(), 4);

    match &dispatch.iopub[0].payload {
        KernelIoPubPayload::Status(status) => {
            assert_eq!(status.execution_state, KernelExecutionState::Busy);
            assert_eq!(status.lifecycle_state, KernelLifecycleState::Running);
        }
        other => panic!("expected busy status payload, got {other:?}"),
    }

    match &dispatch.iopub[1].payload {
        KernelIoPubPayload::ExecuteInput(input) => {
            assert_eq!(input.execution_count, Some(1));
            assert_eq!(input.code, "1");
        }
        other => panic!("expected execute_input payload, got {other:?}"),
    }

    match &dispatch.iopub[2].payload {
        KernelIoPubPayload::ExecuteResult(result) => {
            assert_eq!(result.data.get("text/plain"), Some("1"));
        }
        other => panic!("expected execute_result payload, got {other:?}"),
    }

    match &dispatch.iopub[3].payload {
        KernelIoPubPayload::Status(status) => {
            assert_eq!(status.execution_state, KernelExecutionState::Idle);
            assert_eq!(status.lifecycle_state, KernelLifecycleState::Running);
        }
        other => panic!("expected idle status payload, got {other:?}"),
    }

    assert_eq!(reply.metadata.get("channel"), Some("shell"));
    assert_eq!(
        reply.metadata.get("parent_msg_type"),
        Some("execute_request")
    );
    assert_eq!(dispatch.iopub[0].metadata.get("channel"), Some("iopub"));
    assert_eq!(
        dispatch.iopub[0].metadata.get("parent_msg_type"),
        Some("execute_request")
    );
    assert_eq!(
        dispatch.iopub[0].metadata.get("lifecycle_state"),
        Some("running")
    );
}

#[test]
fn dispatch_request_emits_busy_and_idle_status_for_kernel_info() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);

    let dispatch = adapter.dispatch_request(KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new(
            "client-msg-kinfo-1",
            "session-kinfo",
            "tester",
            "kernel_info_request",
        ),
        parent_header: None,
        metadata: KernelMetadata::new(),
        request: KernelRequest::KernelInfo(KernelInfoRequest),
    });

    assert_eq!(dispatch.iopub.len(), 2);
    match &dispatch.iopub[0].payload {
        KernelIoPubPayload::Status(status) => {
            assert_eq!(status.execution_state, KernelExecutionState::Busy);
            assert_eq!(status.lifecycle_state, KernelLifecycleState::Running);
        }
        other => panic!("expected busy status payload, got {other:?}"),
    }
    match &dispatch.iopub[1].payload {
        KernelIoPubPayload::Status(status) => {
            assert_eq!(status.execution_state, KernelExecutionState::Idle);
            assert_eq!(status.lifecycle_state, KernelLifecycleState::Running);
        }
        other => panic!("expected idle status payload, got {other:?}"),
    }
}

#[test]
fn dispatch_request_handles_comm_info_without_touching_runtime_session() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);

    let dispatch = adapter.dispatch_request(KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new(
            "client-msg-comm-info",
            "session-comm",
            "tester",
            "comm_info_request",
        ),
        parent_header: None,
        metadata: KernelMetadata::new(),
        request: KernelRequest::CommInfo(KernelCommInfoRequest::default()),
    });

    let reply = dispatch
        .reply
        .as_ref()
        .expect("comm_info should reply immediately");
    assert_eq!(reply.channel, KernelChannel::Shell);
    assert_eq!(reply.header.msg_type, "comm_info_reply");
    assert_eq!(reply.metadata.get("status"), Some("ok"));
    assert_eq!(dispatch.iopub.len(), 2);
    assert_eq!(adapter.execution_count(), 0);

    match &reply.response {
        KernelResponse::CommInfo(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
        }
        other => panic!("expected comm-info response, got {other:?}"),
    }
}

#[test]
fn dispatch_request_replies_to_debug_request_as_unsupported_without_exiting() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);

    let dispatch = adapter.dispatch_request(KernelRequestEnvelope {
        channel: KernelChannel::Control,
        header: KernelMessageHeader::new(
            "client-msg-debug",
            "session-debug",
            "tester",
            "debug_request",
        ),
        parent_header: None,
        metadata: KernelMetadata::new(),
        request: KernelRequest::Debug(KernelDebugRequest {
            request_seq: Some(7),
            command: Some("initialize".to_string()),
        }),
    });

    let reply = dispatch
        .reply
        .as_ref()
        .expect("debug request should receive an unsupported reply");
    assert_eq!(reply.channel, KernelChannel::Control);
    assert_eq!(reply.header.msg_type, "debug_reply");
    assert_eq!(reply.metadata.get("status"), Some("error"));
    assert_eq!(adapter.execution_count(), 0);
    assert_eq!(adapter.lifecycle_state(), KernelLifecycleState::Running);

    match &reply.response {
        KernelResponse::Debug(response) => {
            assert_eq!(response.status, KernelReplyStatus::Error);
            assert_eq!(response.request_seq, Some(7));
            assert_eq!(response.command.as_deref(), Some("initialize"));
            assert!(response.message.contains("debug protocol"));
        }
        other => panic!("expected debug response, got {other:?}"),
    }
}

#[test]
fn dispatch_request_ignores_comm_messages_without_affecting_session_state() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);

    let dispatch = adapter.dispatch_request(KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new(
            "client-msg-comm-open",
            "session-comm-open",
            "tester",
            "comm_open",
        ),
        parent_header: None,
        metadata: KernelMetadata::new(),
        request: KernelRequest::Ignored(KernelIgnoredRequest {
            msg_type: "comm_open".to_string(),
        }),
    });

    assert!(dispatch.reply.is_none());
    assert!(dispatch.iopub.is_empty());
    assert!(dispatch.stdin.is_empty());
    assert_eq!(adapter.execution_count(), 0);
    assert_eq!(adapter.lifecycle_state(), KernelLifecycleState::Running);
    assert_eq!(adapter.execution_state(), KernelExecutionState::Idle);
}

#[test]
fn shell_request_envelope_dispatches_kernel_info_and_shutdown() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);

    let info = adapter
        .handle_shell_request(KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: KernelMessageHeader::new(
                "client-msg-7",
                "session-1",
                "tester",
                "kernel_info_request",
            ),
            parent_header: None,
            metadata: KernelMetadata::new(),
            request: KernelRequest::KernelInfo(KernelInfoRequest),
        })
        .expect("kernel_info should reply immediately");

    assert_eq!(info.channel, KernelChannel::Shell);
    assert_eq!(info.header.msg_type, "kernel_info_reply");
    assert_eq!(info.header.session_id, "session-1");
    assert_eq!(
        info.parent_header
            .as_ref()
            .map(|header| header.msg_id.as_str()),
        Some("client-msg-7")
    );
    assert_eq!(info.metadata.get("status"), Some("ok"));
    match info.response {
        KernelResponse::KernelInfo(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(response.implementation, "rum");
            assert_eq!(response.protocol_version, "5.3");
            assert_eq!(response.language_info.name, "rum");
        }
        other => panic!("expected kernel-info response, got {other:?}"),
    }

    let shutdown = adapter
        .handle_shell_request(KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: KernelMessageHeader::new(
                "client-msg-8",
                "session-1",
                "tester",
                "shutdown_request",
            ),
            parent_header: None,
            metadata: KernelMetadata::new(),
            request: KernelRequest::Shutdown(KernelShutdownRequest { restart: false }),
        })
        .expect("shutdown should reply immediately");

    assert_eq!(shutdown.channel, KernelChannel::Shell);
    assert_eq!(shutdown.header.msg_type, "shutdown_reply");
    assert!(adapter.shutdown_requested());
    match shutdown.response {
        KernelResponse::Shutdown(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert!(!response.restart);
            assert_eq!(
                response.lifecycle_state,
                KernelLifecycleState::ShutdownRequested
            );
        }
        other => panic!("expected shutdown response, got {other:?}"),
    }
}

#[test]
fn shell_request_envelope_dispatches_execute_requests() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);

    let initial = adapter.dispatch_request(KernelRequestEnvelope {
        channel: KernelChannel::Shell,
        header: KernelMessageHeader::new("client-msg-12", "session-2", "tester", "execute_request"),
        parent_header: None,
        metadata: KernelMetadata::new(),
        request: KernelRequest::Execute(KernelExecuteRequest::new("1")),
    });
    assert!(initial.reply.is_none());
    let dispatch = finish_adapter_execute(&mut adapter, initial);
    let response = dispatch
        .reply
        .expect("simple execute should reply after worker completion");

    assert_eq!(response.channel, KernelChannel::Shell);
    assert_eq!(response.header.msg_type, "execute_reply");
    assert_eq!(
        response
            .parent_header
            .as_ref()
            .map(|header| header.msg_id.as_str()),
        Some("client-msg-12")
    );
    assert_eq!(response.metadata.get("status"), Some("ok"));
    match response.response {
        KernelResponse::Execute(execute) => {
            assert_eq!(execute.execution_count, Some(1));
            assert_eq!(execute.outputs.len(), 1);
        }
        other => panic!("expected execute response, got {other:?}"),
    }
}

#[test]
fn control_request_interrupt_updates_lifecycle_state() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);

    let dispatch = adapter.dispatch_request(KernelRequestEnvelope {
        channel: KernelChannel::Control,
        header: KernelMessageHeader::new(
            "client-msg-30",
            "session-4",
            "tester",
            "interrupt_request",
        ),
        parent_header: None,
        metadata: KernelMetadata::new(),
        request: KernelRequest::Interrupt(KernelInterruptRequest),
    });
    let reply = dispatch
        .reply
        .as_ref()
        .expect("interrupt should reply immediately");

    assert_eq!(reply.channel, KernelChannel::Control);
    assert_eq!(dispatch.iopub.len(), 2);
    match &dispatch.iopub[0].payload {
        KernelIoPubPayload::Status(status) => {
            assert_eq!(status.execution_state, KernelExecutionState::Busy);
            assert_eq!(
                status.lifecycle_state,
                KernelLifecycleState::InterruptRequested
            );
        }
        other => panic!("expected busy status payload, got {other:?}"),
    }
    match &dispatch.iopub[1].payload {
        KernelIoPubPayload::Status(status) => {
            assert_eq!(status.execution_state, KernelExecutionState::Idle);
            assert_eq!(
                status.lifecycle_state,
                KernelLifecycleState::InterruptRequested
            );
        }
        other => panic!("expected idle status payload, got {other:?}"),
    }
    assert_eq!(reply.header.msg_type, "interrupt_reply");
    assert_eq!(
        reply.metadata.get("lifecycle_state"),
        Some("interrupt_requested")
    );
    assert_eq!(
        adapter.lifecycle_state(),
        KernelLifecycleState::InterruptRequested
    );
    assert!(adapter.interrupt_requested());

    match &reply.response {
        KernelResponse::Interrupt(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert!(response.interrupt_requested);
            assert_eq!(
                response.lifecycle_state,
                KernelLifecycleState::InterruptRequested
            );
        }
        other => panic!("expected interrupt response, got {other:?}"),
    }
}

#[test]
fn control_request_shutdown_transitions_kernel_to_shutdown() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);

    let dispatch = adapter.dispatch_request(KernelRequestEnvelope {
        channel: KernelChannel::Control,
        header: KernelMessageHeader::new(
            "client-msg-31",
            "session-4",
            "tester",
            "shutdown_request",
        ),
        parent_header: None,
        metadata: KernelMetadata::new(),
        request: KernelRequest::Shutdown(KernelShutdownRequest { restart: false }),
    });
    let reply = dispatch
        .reply
        .as_ref()
        .expect("shutdown should reply immediately");

    assert_eq!(reply.channel, KernelChannel::Control);
    assert_eq!(reply.header.msg_type, "shutdown_reply");
    assert_eq!(
        reply.metadata.get("lifecycle_state"),
        Some("shutdown_requested")
    );
    assert_eq!(adapter.lifecycle_state(), KernelLifecycleState::Shutdown);
    assert!(adapter.shutdown_requested());
}

#[test]
fn shell_request_envelope_dispatches_is_complete_requests() {
    let session = Session::new();
    let mut adapter = KernelAdapter::new(session);

    let response = adapter
        .handle_shell_request(KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: KernelMessageHeader::new(
                "client-msg-40",
                "session-5",
                "tester",
                "is_complete_request",
            ),
            parent_header: None,
            metadata: KernelMetadata::new(),
            request: KernelRequest::IsComplete(KernelIsCompleteRequest::new("f[\n  1")),
        })
        .expect("is_complete should reply immediately");

    assert_eq!(response.header.msg_type, "is_complete_reply");
    assert_eq!(response.metadata.get("status"), Some("ok"));
    assert_eq!(
        response.metadata.get("completion_status"),
        Some("incomplete")
    );

    match response.response {
        KernelResponse::IsComplete(result) => {
            assert_eq!(result.completion_status, KernelIsCompleteStatus::Incomplete);
            assert_eq!(result.indent.as_deref(), Some("    "));
        }
        other => panic!("expected is-complete response, got {other:?}"),
    }
}
