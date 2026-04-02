use super::*;

#[test]
fn event_pump_preserves_routing_ids_on_replies_and_routes_iopub_separately() {
    let codec = KernelWireCodec::new(
        KernelWireConfig::default()
            .with_session_id("session-13")
            .with_signature_key("secret"),
    );
    let adapter = KernelAdapter::new(Session::new());
    let mut pump = KernelEventPump::new(adapter, codec.clone());
    let mut transport = MockSocketTransport::default();

    let mut request = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: KernelMessageHeader::new(
                "client-msg-66",
                "session-13",
                "tester",
                "execute_request",
            ),
            parent_header: None,
            metadata: super::KernelMetadata::new(),
            request: KernelRequest::Execute(KernelExecuteRequest::new("1")),
        })
        .expect("request should encode");
    request.routing_ids = vec![b"frontend".to_vec(), b"shell".to_vec()];
    transport.queue_incoming(request);

    assert!(
        pump.pump_once(&mut transport, KernelChannel::Shell)
            .expect("pump should succeed")
    );
    assert_eq!(transport.shell.len(), 1);
    assert_eq!(transport.iopub.len(), 4);
    assert_eq!(
        transport.shell[0].routing_ids,
        vec![b"frontend".to_vec(), b"shell".to_vec()]
    );
    assert!(
        transport
            .iopub
            .iter()
            .all(|message| message.routing_ids.is_empty())
    );
}

#[test]
fn pump_next_prioritizes_control_messages_before_shell_messages() {
    let codec = KernelWireCodec::new(KernelWireConfig::default().with_session_id("session-15"));
    let adapter = KernelAdapter::new(Session::new());
    let mut pump = KernelEventPump::new(adapter, codec.clone());
    let mut transport = MockSocketTransport::default();

    transport.queue_incoming(
        codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Shell,
                header: KernelMessageHeader::new(
                    "client-msg-68",
                    "session-15",
                    "tester",
                    "execute_request",
                ),
                parent_header: None,
                metadata: super::KernelMetadata::new(),
                request: KernelRequest::Execute(KernelExecuteRequest::new("1")),
            })
            .expect("shell request should encode"),
    );
    transport.queue_incoming(
        codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Control,
                header: KernelMessageHeader::new(
                    "client-msg-69",
                    "session-15",
                    "tester",
                    "interrupt_request",
                ),
                parent_header: None,
                metadata: super::KernelMetadata::new(),
                request: KernelRequest::Interrupt(KernelInterruptRequest),
            })
            .expect("control request should encode"),
    );

    let channel = pump
        .pump_next(&mut transport)
        .expect("pump should succeed")
        .expect("one request should be processed");

    assert_eq!(channel, KernelChannel::Control);
    assert_eq!(transport.control.len(), 1);
    assert!(transport.shell.is_empty());
    assert_eq!(transport.incoming_len(KernelChannel::Shell), 1);
}

#[test]
fn pump_next_prioritizes_stdin_replies_while_kernel_is_waiting_for_input() {
    let codec = KernelWireCodec::new(KernelWireConfig::default().with_session_id("session-15b"));
    let adapter = KernelAdapter::new(Session::new());
    let mut pump = KernelEventPump::new(adapter, codec.clone());
    let mut transport = MockSocketTransport::default();

    let parent_header = KernelMessageHeader::new(
        "client-msg-stdin-parent",
        "session-15b",
        "tester",
        "execute_request",
    );
    let pending = pump
        .adapter_mut()
        .request_stdin_for_execute(
            &parent_header,
            &KernelExecuteRequest::new("futureInput[]"),
            KernelInputRequest::new("name: "),
        )
        .expect("stdin request should stage");

    transport.queue_incoming(
        codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Shell,
                header: KernelMessageHeader::new(
                    "client-msg-shell",
                    "session-15b",
                    "tester",
                    "execute_request",
                ),
                parent_header: None,
                metadata: super::KernelMetadata::new(),
                request: KernelRequest::Execute(KernelExecuteRequest::new("1")),
            })
            .expect("shell request should encode"),
    );
    transport.queue_incoming(encode_input_reply_envelope(&KernelInputReplyEnvelope {
        channel: KernelChannel::Stdin,
        header: KernelMessageHeader::new(
            "client-msg-stdin-reply",
            "session-15b",
            "tester",
            "input_reply",
        ),
        parent_header: Some(pending.header.clone()),
        metadata: super::KernelMetadata::new(),
        reply: KernelInputReply::new("Alice"),
    }));

    let channel = pump
        .pump_next(&mut transport)
        .expect("pump should succeed")
        .expect("one request should be processed");

    assert_eq!(channel, KernelChannel::Stdin);
    assert!(!pump.adapter().awaiting_stdin());
    assert_eq!(
        pump.adapter_mut()
            .take_last_input_reply()
            .map(|reply| reply.value),
        Some("Alice".to_string())
    );
    assert!(transport.shell.is_empty());
    assert_eq!(transport.incoming_len(KernelChannel::Shell), 1);
}

#[test]
fn pump_until_idle_stops_after_shutdown_and_leaves_later_shell_work_queued() {
    let codec = KernelWireCodec::new(KernelWireConfig::default().with_session_id("session-16"));
    let adapter = KernelAdapter::new(Session::new());
    let mut pump = KernelEventPump::new(adapter, codec.clone());
    let mut transport = MockSocketTransport::default();

    transport.queue_incoming(
        codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Shell,
                header: KernelMessageHeader::new(
                    "client-msg-70",
                    "session-16",
                    "tester",
                    "execute_request",
                ),
                parent_header: None,
                metadata: super::KernelMetadata::new(),
                request: KernelRequest::Execute(KernelExecuteRequest::new("1")),
            })
            .expect("shell request should encode"),
    );
    transport.queue_incoming(
        codec
            .encode_request_envelope(&KernelRequestEnvelope {
                channel: KernelChannel::Control,
                header: KernelMessageHeader::new(
                    "client-msg-71",
                    "session-16",
                    "tester",
                    "shutdown_request",
                ),
                parent_header: None,
                metadata: super::KernelMetadata::new(),
                request: KernelRequest::Shutdown(KernelShutdownRequest { restart: true }),
            })
            .expect("shutdown request should encode"),
    );

    let report = pump
        .pump_until_idle(&mut transport)
        .expect("loop should succeed");

    assert_eq!(report.control_messages, 1);
    assert_eq!(report.shell_messages, 0);
    assert_eq!(report.total_messages(), 1);
    assert!(report.stopped_for_shutdown);
    assert_eq!(transport.control.len(), 1);
    assert!(transport.shell.is_empty());
    assert_eq!(transport.incoming_len(KernelChannel::Shell), 1);
    assert!(pump.adapter().shutdown_requested());

    let reply = decode_response_message(&transport.control[0]).expect("reply should decode");
    match reply.response {
        KernelResponse::Shutdown(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert!(response.restart);
            assert_eq!(
                response.lifecycle_state,
                KernelLifecycleState::ShutdownRequested
            );
        }
        other => panic!("expected shutdown reply, got {other:?}"),
    }
}
