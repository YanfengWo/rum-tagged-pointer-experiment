use super::*;

#[test]
fn native_zeromq_backend_frontend_style_execute_preserves_headers_and_iopub_metadata() {
    let bootstrap = loopback_connection();
    let codec = bootstrap.wire_codec();
    let shell_endpoint = bootstrap.transport_layout().shell.uri();
    let iopub_endpoint = bootstrap.transport_layout().iopub.uri();
    let runtime = test_runtime();
    let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
        .expect("native zeromq backend should build");
    let mut driver = backend.socket_driver(Session::new());

    let mut shell = runtime.block_on(async {
        let mut shell = DealerSocket::new();
        shell
            .connect(&shell_endpoint)
            .await
            .expect("shell dealer should connect");
        shell
    });
    let mut iopub = runtime.block_on(async {
        let mut iopub = SubSocket::new();
        iopub
            .connect(&iopub_endpoint)
            .await
            .expect("iopub sub should connect");
        iopub
            .subscribe("")
            .await
            .expect("iopub sub should subscribe");
        iopub
    });

    runtime.block_on(async {
        tokio::time::sleep(Duration::from_millis(50)).await;
    });

    let execute_header = KernelMessageHeader::new(
        "execute-zmq-metadata-1",
        "frontend-session",
        "frontend",
        "execute_request",
    );
    let execute_request = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: execute_header.clone(),
            parent_header: None,
            metadata: KernelMetadata::new(),
            request: KernelRequest::Execute(crate::KernelExecuteRequest::new("1")),
        })
        .expect("execute request should encode");
    runtime.block_on(async {
        shell
            .send(frames_to_zmq_message(
                wire_message_to_frames(execute_request).frames,
            ))
            .await
            .expect("shell dealer should send execute request");
    });

    wait_for_socket_event(
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::Channel(KernelChannel::Shell),
    );

    let shell_reply_wire = runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(2), shell.recv())
            .await
            .expect("shell reply should arrive before timeout")
            .expect("shell reply should decode")
    });
    let shell_reply = decode_response_message(
        &frames_to_wire_message(
            KernelChannel::Shell,
            &bootstrap.wire_config().delimiter,
            zmq_message_to_frames(shell_reply_wire),
        )
        .expect("shell reply frames should convert"),
    )
    .expect("shell reply should decode");

    assert_eq!(shell_reply.header.username, "rum-kernel");
    assert_eq!(
        shell_reply.header.protocol_version,
        execute_header.protocol_version
    );
    assert_eq!(shell_reply.parent_header, Some(execute_header.clone()));
    assert_eq!(shell_reply.metadata.get("channel"), Some("shell"));
    assert_eq!(
        shell_reply.metadata.get("parent_msg_type"),
        Some("execute_request")
    );

    match shell_reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(response.execution_count, Some(1));
        }
        other => panic!("expected execute reply, got {other:?}"),
    }

    let mut iopub_messages = Vec::new();
    while iopub_messages.len() < 4 {
        let message = runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(2), iopub.recv())
                .await
                .expect("iopub message should arrive before timeout")
                .expect("iopub message should decode")
        });
        let envelope = crate::decode_iopub_message(
            &frames_to_wire_message(
                KernelChannel::IoPub,
                &bootstrap.wire_config().delimiter,
                zmq_message_to_frames(message),
            )
            .expect("iopub frames should convert"),
        )
        .expect("iopub message should parse");
        iopub_messages.push(envelope);
    }

    for message in &iopub_messages {
        assert_eq!(message.header.username, "rum-kernel");
        assert_eq!(
            message.header.protocol_version,
            execute_header.protocol_version
        );
        assert_eq!(message.parent_header, Some(execute_header.clone()));
        assert_eq!(message.metadata.get("channel"), Some("iopub"));
        assert_eq!(
            message.metadata.get("parent_msg_type"),
            Some("execute_request")
        );
    }

    assert!(iopub_messages.iter().any(|message| matches!(
        &message.payload,
        crate::KernelIoPubPayload::Status(status)
            if status.execution_state == crate::KernelExecutionState::Busy
                && status.lifecycle_state == crate::KernelLifecycleState::Running
    )));
    assert!(iopub_messages.iter().any(|message| matches!(
        &message.payload,
        crate::KernelIoPubPayload::ExecuteInput(input)
            if input.code == "1" && input.execution_count == Some(1)
    )));
    assert!(iopub_messages.iter().any(|message| matches!(
        &message.payload,
        crate::KernelIoPubPayload::ExecuteResult(result) if result.data.get("text/plain").is_some()
    )));
    assert!(iopub_messages.iter().any(|message| matches!(
        &message.payload,
        crate::KernelIoPubPayload::Status(status)
            if status.execution_state == crate::KernelExecutionState::Idle
                && status.lifecycle_state == crate::KernelLifecycleState::Running
    )));
}

#[test]
fn native_zeromq_backend_frontend_style_stdin_round_trip_emits_iopub_and_shell_reply() {
    let bootstrap = loopback_connection();
    let codec = bootstrap.wire_codec();
    let shell_endpoint = bootstrap.transport_layout().shell.uri();
    let stdin_endpoint = bootstrap.transport_layout().stdin.uri();
    let iopub_endpoint = bootstrap.transport_layout().iopub.uri();
    let frontend_identity =
        PeerIdentity::from_str("frontend-zmq-stdin-client").expect("identity should parse");
    let runtime = test_runtime();
    let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
        .expect("native zeromq backend should build");
    let mut driver = backend.socket_driver(Session::new());

    let mut shell = runtime.block_on(async {
        let mut options = SocketOptions::default();
        options.peer_identity(frontend_identity.clone());
        let mut shell = DealerSocket::with_options(options);
        shell
            .connect(&shell_endpoint)
            .await
            .expect("shell dealer should connect");
        shell
    });
    let mut stdin = runtime.block_on(async {
        let mut options = SocketOptions::default();
        options.peer_identity(frontend_identity);
        let mut stdin = DealerSocket::with_options(options);
        stdin
            .connect(&stdin_endpoint)
            .await
            .expect("stdin dealer should connect");
        stdin
    });
    let mut iopub = runtime.block_on(async {
        let mut iopub = SubSocket::new();
        iopub
            .connect(&iopub_endpoint)
            .await
            .expect("iopub sub should connect");
        iopub
            .subscribe("")
            .await
            .expect("iopub sub should subscribe");
        iopub
    });

    runtime.block_on(async {
        tokio::time::sleep(Duration::from_millis(50)).await;
    });

    let execute_header = KernelMessageHeader::new(
        "execute-zmq-stdin-1",
        "frontend-session",
        "frontend",
        "execute_request",
    );
    let execute_request = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: execute_header.clone(),
            parent_header: None,
            metadata: KernelMetadata::new(),
            request: KernelRequest::Execute(crate::KernelExecuteRequest::new(
                "InputString[\"Name: \"]",
            )),
        })
        .expect("execute request should encode");
    runtime.block_on(async {
        shell
            .send(frames_to_zmq_message(
                wire_message_to_frames(execute_request).frames,
            ))
            .await
            .expect("shell dealer should send execute request");
    });

    wait_for_socket_event(
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::Channel(KernelChannel::Shell),
    );

    let stdin_request_wire = runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(2), stdin.recv())
            .await
            .expect("stdin request should arrive before timeout")
            .expect("stdin request should decode")
    });
    let stdin_request_wire = frames_to_wire_message(
        KernelChannel::Stdin,
        &bootstrap.wire_config().delimiter,
        zmq_message_to_frames(stdin_request_wire),
    )
    .expect("stdin request frames should convert");
    let stdin_request = crate::decode_input_request_message(&stdin_request_wire)
        .expect("stdin request should parse");
    assert_eq!(stdin_request.request.prompt, "Name: ");

    let input_reply = codec
        .encode_input_reply_envelope(&crate::KernelInputReplyEnvelope {
            channel: KernelChannel::Stdin,
            header: KernelMessageHeader::new(
                "input-zmq-reply-1",
                "frontend-session",
                "frontend",
                "input_reply",
            ),
            parent_header: Some(stdin_request.header.clone()),
            metadata: KernelMetadata::new(),
            reply: crate::KernelInputReply::new("Ada"),
        })
        .expect("input reply should encode");
    runtime.block_on(async {
        stdin
            .send(frames_to_zmq_message(
                wire_message_to_frames(input_reply).frames,
            ))
            .await
            .expect("stdin dealer should send input reply");
    });

    wait_for_socket_event(
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::Channel(KernelChannel::Stdin),
    );

    let shell_reply_wire = runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(2), shell.recv())
            .await
            .expect("shell reply should arrive before timeout")
            .expect("shell reply should decode")
    });
    let shell_reply = decode_response_message(
        &frames_to_wire_message(
            KernelChannel::Shell,
            &bootstrap.wire_config().delimiter,
            zmq_message_to_frames(shell_reply_wire),
        )
        .expect("shell reply frames should convert"),
    )
    .expect("shell reply should decode");
    match shell_reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(response.execution_count, Some(1));
        }
        other => panic!("expected execute reply, got {other:?}"),
    }

    let mut iopub_payloads = Vec::new();
    while iopub_payloads.len() < 4 {
        let message = runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(2), iopub.recv())
                .await
                .expect("iopub message should arrive before timeout")
                .expect("iopub message should decode")
        });
        let envelope = crate::decode_iopub_message(
            &frames_to_wire_message(
                KernelChannel::IoPub,
                &bootstrap.wire_config().delimiter,
                zmq_message_to_frames(message),
            )
            .expect("iopub frames should convert"),
        )
        .expect("iopub message should parse");
        iopub_payloads.push(envelope.payload);
    }

    assert!(iopub_payloads.iter().any(|payload| matches!(
        payload,
        crate::KernelIoPubPayload::Status(status)
            if status.execution_state == crate::KernelExecutionState::Busy
    )));
    assert!(iopub_payloads.iter().any(|payload| matches!(
        payload,
        crate::KernelIoPubPayload::ExecuteInput(input)
            if input.code == "InputString[\"Name: \"]"
    )));
    assert!(iopub_payloads.iter().any(|payload| matches!(
        payload,
        crate::KernelIoPubPayload::ExecuteResult(result)
            if result.data.get("text/plain") == Some("\"Ada\"")
    )));
    assert!(iopub_payloads.iter().any(|payload| matches!(
        payload,
        crate::KernelIoPubPayload::Status(status)
            if status.execution_state == crate::KernelExecutionState::Idle
    )));
}

#[test]
fn native_zeromq_backend_frontend_style_multi_message_session_flow_tracks_counts_and_shutdown() {
    let bootstrap = loopback_connection();
    let codec = bootstrap.wire_codec();
    let shell_endpoint = bootstrap.transport_layout().shell.uri();
    let stdin_endpoint = bootstrap.transport_layout().stdin.uri();
    let control_endpoint = bootstrap.transport_layout().control.uri();
    let iopub_endpoint = bootstrap.transport_layout().iopub.uri();
    let frontend_identity =
        PeerIdentity::from_str("frontend-zmq-multi-client").expect("identity should parse");
    let runtime = test_runtime();
    let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
        .expect("native zeromq backend should build");
    let mut driver = backend.socket_driver(Session::new());

    let mut shell = runtime.block_on(async {
        let mut options = SocketOptions::default();
        options.peer_identity(frontend_identity.clone());
        let mut shell = DealerSocket::with_options(options);
        shell
            .connect(&shell_endpoint)
            .await
            .expect("shell dealer should connect");
        shell
    });
    let mut stdin = runtime.block_on(async {
        let mut options = SocketOptions::default();
        options.peer_identity(frontend_identity);
        let mut stdin = DealerSocket::with_options(options);
        stdin
            .connect(&stdin_endpoint)
            .await
            .expect("stdin dealer should connect");
        stdin
    });
    let mut control = runtime.block_on(async {
        let mut control = DealerSocket::new();
        control
            .connect(&control_endpoint)
            .await
            .expect("control dealer should connect");
        control
    });
    let mut iopub = runtime.block_on(async {
        let mut iopub = SubSocket::new();
        iopub
            .connect(&iopub_endpoint)
            .await
            .expect("iopub sub should connect");
        iopub
            .subscribe("")
            .await
            .expect("iopub sub should subscribe");
        iopub
    });

    runtime.block_on(async {
        tokio::time::sleep(Duration::from_millis(50)).await;
    });

    let first_execute_header = KernelMessageHeader::new(
        "execute-zmq-multi-1",
        "frontend-session",
        "frontend",
        "execute_request",
    );
    let first_execute = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: first_execute_header.clone(),
            parent_header: None,
            metadata: KernelMetadata::new(),
            request: KernelRequest::Execute(crate::KernelExecuteRequest::new("1")),
        })
        .expect("first execute request should encode");
    runtime.block_on(async {
        shell
            .send(frames_to_zmq_message(
                wire_message_to_frames(first_execute).frames,
            ))
            .await
            .expect("shell dealer should send first execute");
    });

    wait_for_socket_event(
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::Channel(KernelChannel::Shell),
    );

    let first_reply_wire = runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(2), shell.recv())
            .await
            .expect("first shell reply should arrive before timeout")
            .expect("first shell reply should decode")
    });
    let first_reply = decode_response_message(
        &frames_to_wire_message(
            KernelChannel::Shell,
            &bootstrap.wire_config().delimiter,
            zmq_message_to_frames(first_reply_wire),
        )
        .expect("first shell reply frames should convert"),
    )
    .expect("first shell reply should decode");
    match first_reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(response.execution_count, Some(1));
        }
        other => panic!("expected first execute reply, got {other:?}"),
    }

    let mut first_iopub = Vec::new();
    while first_iopub.len() < 4 {
        let message = runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(2), iopub.recv())
                .await
                .expect("first iopub message should arrive before timeout")
                .expect("first iopub message should decode")
        });
        first_iopub.push(
            crate::decode_iopub_message(
                &frames_to_wire_message(
                    KernelChannel::IoPub,
                    &bootstrap.wire_config().delimiter,
                    zmq_message_to_frames(message),
                )
                .expect("first iopub frames should convert"),
            )
            .expect("first iopub message should parse"),
        );
    }
    assert!(first_iopub.iter().any(|message| matches!(
        &message.payload,
        crate::KernelIoPubPayload::ExecuteInput(input)
            if input.code == "1" && input.execution_count == Some(1)
    )));
    assert!(first_iopub.iter().any(|message| matches!(
        &message.payload,
        crate::KernelIoPubPayload::ExecuteResult(result)
            if result.data.get("text/plain") == Some("1")
    )));

    let second_execute_header = KernelMessageHeader::new(
        "execute-zmq-multi-2",
        "frontend-session",
        "frontend",
        "execute_request",
    );
    let second_execute = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: second_execute_header.clone(),
            parent_header: None,
            metadata: KernelMetadata::new(),
            request: KernelRequest::Execute(crate::KernelExecuteRequest::new(
                "InputString[\"Name: \"]",
            )),
        })
        .expect("second execute request should encode");
    runtime.block_on(async {
        shell
            .send(frames_to_zmq_message(
                wire_message_to_frames(second_execute).frames,
            ))
            .await
            .expect("shell dealer should send second execute");
    });

    wait_for_socket_event(
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::Channel(KernelChannel::Shell),
    );

    let stdin_request_wire = runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(2), stdin.recv())
            .await
            .expect("stdin request should arrive before timeout")
            .expect("stdin request should decode")
    });
    let stdin_request_wire = frames_to_wire_message(
        KernelChannel::Stdin,
        &bootstrap.wire_config().delimiter,
        zmq_message_to_frames(stdin_request_wire),
    )
    .expect("stdin request frames should convert");
    let stdin_request = crate::decode_input_request_message(&stdin_request_wire)
        .expect("stdin request should decode");
    assert_eq!(stdin_request.request.prompt, "Name: ");

    let input_reply = codec
        .encode_input_reply_envelope(&crate::KernelInputReplyEnvelope {
            channel: KernelChannel::Stdin,
            header: KernelMessageHeader::new(
                "input-zmq-multi-1",
                "frontend-session",
                "frontend",
                "input_reply",
            ),
            parent_header: Some(stdin_request.header.clone()),
            metadata: KernelMetadata::new(),
            reply: crate::KernelInputReply::new("Ada"),
        })
        .expect("stdin reply should encode");
    runtime.block_on(async {
        stdin
            .send(frames_to_zmq_message(
                wire_message_to_frames(input_reply).frames,
            ))
            .await
            .expect("stdin dealer should send input reply");
    });

    wait_for_socket_event(
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::Channel(KernelChannel::Stdin),
    );

    let second_reply_wire = runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(2), shell.recv())
            .await
            .expect("second shell reply should arrive before timeout")
            .expect("second shell reply should decode")
    });
    let second_reply = decode_response_message(
        &frames_to_wire_message(
            KernelChannel::Shell,
            &bootstrap.wire_config().delimiter,
            zmq_message_to_frames(second_reply_wire),
        )
        .expect("second shell reply frames should convert"),
    )
    .expect("second shell reply should decode");
    match second_reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(response.execution_count, Some(2));
        }
        other => panic!("expected second execute reply, got {other:?}"),
    }

    let mut second_iopub = Vec::new();
    while second_iopub.len() < 4 {
        let message = runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(2), iopub.recv())
                .await
                .expect("second iopub message should arrive before timeout")
                .expect("second iopub message should decode")
        });
        second_iopub.push(
            crate::decode_iopub_message(
                &frames_to_wire_message(
                    KernelChannel::IoPub,
                    &bootstrap.wire_config().delimiter,
                    zmq_message_to_frames(message),
                )
                .expect("second iopub frames should convert"),
            )
            .expect("second iopub message should parse"),
        );
    }
    assert!(second_iopub.iter().any(|message| matches!(
        &message.payload,
        crate::KernelIoPubPayload::ExecuteInput(input)
            if input.code == "InputString[\"Name: \"]" && input.execution_count == Some(2)
    )));
    assert!(second_iopub.iter().any(|message| matches!(
        &message.payload,
        crate::KernelIoPubPayload::ExecuteResult(result)
            if result.data.get("text/plain") == Some("\"Ada\"")
    )));

    let third_execute_header = KernelMessageHeader::new(
        "execute-zmq-multi-3",
        "frontend-session",
        "frontend",
        "execute_request",
    );
    let third_execute = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: third_execute_header.clone(),
            parent_header: None,
            metadata: KernelMetadata::new(),
            request: KernelRequest::Execute(crate::KernelExecuteRequest::new("2")),
        })
        .expect("third execute request should encode");
    runtime.block_on(async {
        shell
            .send(frames_to_zmq_message(
                wire_message_to_frames(third_execute).frames,
            ))
            .await
            .expect("shell dealer should send third execute");
    });

    wait_for_socket_event(
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::Channel(KernelChannel::Shell),
    );

    let third_reply_wire = runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(2), shell.recv())
            .await
            .expect("third shell reply should arrive before timeout")
            .expect("third shell reply should decode")
    });
    let third_reply = decode_response_message(
        &frames_to_wire_message(
            KernelChannel::Shell,
            &bootstrap.wire_config().delimiter,
            zmq_message_to_frames(third_reply_wire),
        )
        .expect("third shell reply frames should convert"),
    )
    .expect("third shell reply should decode");
    match third_reply.response {
        KernelResponse::Execute(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(response.execution_count, Some(3));
        }
        other => panic!("expected third execute reply, got {other:?}"),
    }

    let mut third_iopub = Vec::new();
    while third_iopub.len() < 4 {
        let message = runtime.block_on(async {
            tokio::time::timeout(Duration::from_secs(2), iopub.recv())
                .await
                .expect("third iopub message should arrive before timeout")
                .expect("third iopub message should decode")
        });
        third_iopub.push(
            crate::decode_iopub_message(
                &frames_to_wire_message(
                    KernelChannel::IoPub,
                    &bootstrap.wire_config().delimiter,
                    zmq_message_to_frames(message),
                )
                .expect("third iopub frames should convert"),
            )
            .expect("third iopub message should parse"),
        );
    }
    assert!(third_iopub.iter().any(|message| matches!(
        &message.payload,
        crate::KernelIoPubPayload::ExecuteInput(input)
            if input.code == "2" && input.execution_count == Some(3)
    )));
    assert!(third_iopub.iter().any(|message| matches!(
        &message.payload,
        crate::KernelIoPubPayload::ExecuteResult(result)
            if result.data.get("text/plain") == Some("2")
    )));

    let shutdown_header = KernelMessageHeader::new(
        "shutdown-zmq-multi-1",
        "frontend-session",
        "frontend",
        "shutdown_request",
    );
    let shutdown_request = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Control,
            header: shutdown_header.clone(),
            parent_header: None,
            metadata: KernelMetadata::new(),
            request: KernelRequest::Shutdown(crate::KernelShutdownRequest { restart: false }),
        })
        .expect("shutdown request should encode");
    runtime.block_on(async {
        control
            .send(frames_to_zmq_message(
                wire_message_to_frames(shutdown_request).frames,
            ))
            .await
            .expect("control dealer should send shutdown request");
    });

    wait_for_socket_event(
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::Channel(KernelChannel::Control),
    );

    let shutdown_reply_wire = runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(2), control.recv())
            .await
            .expect("shutdown reply should arrive before timeout")
            .expect("shutdown reply should decode")
    });
    let shutdown_reply = decode_response_message(
        &frames_to_wire_message(
            KernelChannel::Control,
            &bootstrap.wire_config().delimiter,
            zmq_message_to_frames(shutdown_reply_wire),
        )
        .expect("shutdown reply frames should convert"),
    )
    .expect("shutdown reply should decode");
    match shutdown_reply.response {
        KernelResponse::Shutdown(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert!(!response.restart);
            assert_eq!(
                response.lifecycle_state,
                crate::KernelLifecycleState::ShutdownRequested
            );
        }
        other => panic!("expected shutdown reply, got {other:?}"),
    }
}

#[test]
fn native_zeromq_backend_frontend_style_control_interrupt_round_trip() {
    let bootstrap = loopback_connection();
    let codec = bootstrap.wire_codec();
    let control_endpoint = bootstrap.transport_layout().control.uri();
    let runtime = test_runtime();
    let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
        .expect("native zeromq backend should build");
    let mut driver = backend.socket_driver(Session::new());

    let mut control = runtime.block_on(async {
        let mut control = DealerSocket::new();
        control
            .connect(&control_endpoint)
            .await
            .expect("control dealer should connect");
        control
    });

    runtime.block_on(async {
        tokio::time::sleep(Duration::from_millis(50)).await;
    });

    let interrupt_request = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Control,
            header: KernelMessageHeader::new(
                "interrupt-zmq-1",
                "frontend-session",
                "frontend",
                "interrupt_request",
            ),
            parent_header: None,
            metadata: KernelMetadata::new(),
            request: KernelRequest::Interrupt(crate::KernelInterruptRequest),
        })
        .expect("interrupt request should encode");
    runtime.block_on(async {
        control
            .send(frames_to_zmq_message(
                wire_message_to_frames(interrupt_request).frames,
            ))
            .await
            .expect("control dealer should send interrupt request");
    });

    wait_for_socket_event(
        &mut driver,
        backend.transport_mut(),
        crate::KernelSocketPollEvent::Channel(KernelChannel::Control),
    );

    let reply_wire = runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(2), control.recv())
            .await
            .expect("control reply should arrive before timeout")
            .expect("control reply should decode")
    });
    let reply = decode_response_message(
        &frames_to_wire_message(
            KernelChannel::Control,
            &bootstrap.wire_config().delimiter,
            zmq_message_to_frames(reply_wire),
        )
        .expect("control reply frames should convert"),
    )
    .expect("control reply should decode");
    match reply.response {
        KernelResponse::Interrupt(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert_eq!(
                response.lifecycle_state,
                crate::KernelLifecycleState::InterruptRequested
            );
        }
        other => panic!("expected interrupt reply, got {other:?}"),
    }
}

#[test]
fn native_zeromq_backend_shutdown_stops_shell_work_and_reports_pending_shell_messages() {
    let bootstrap = loopback_connection();
    let codec = bootstrap.wire_codec();
    let shell_endpoint = bootstrap.transport_layout().shell.uri();
    let control_endpoint = bootstrap.transport_layout().control.uri();
    let runtime = test_runtime();
    let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap.clone())
        .expect("native zeromq backend should build");
    let mut driver = backend.socket_driver(Session::new());

    let mut shell = runtime.block_on(async {
        let mut shell = DealerSocket::new();
        shell
            .connect(&shell_endpoint)
            .await
            .expect("shell dealer should connect");
        shell
    });
    let mut control = runtime.block_on(async {
        let mut control = DealerSocket::new();
        control
            .connect(&control_endpoint)
            .await
            .expect("control dealer should connect");
        control
    });

    runtime.block_on(async {
        tokio::time::sleep(Duration::from_millis(50)).await;
    });

    let execute_request = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Shell,
            header: KernelMessageHeader::new(
                "execute-zmq-shutdown-queued-1",
                "frontend-session",
                "frontend",
                "execute_request",
            ),
            parent_header: None,
            metadata: KernelMetadata::new(),
            request: KernelRequest::Execute(crate::KernelExecuteRequest::new("1")),
        })
        .expect("queued execute request should encode");
    let shutdown_header = KernelMessageHeader::new(
        "shutdown-zmq-1",
        "frontend-session",
        "frontend",
        "shutdown_request",
    );
    let shutdown_request = codec
        .encode_request_envelope(&KernelRequestEnvelope {
            channel: KernelChannel::Control,
            header: shutdown_header.clone(),
            parent_header: None,
            metadata: KernelMetadata::new(),
            request: KernelRequest::Shutdown(crate::KernelShutdownRequest { restart: true }),
        })
        .expect("shutdown request should encode");

    runtime.block_on(async {
        control
            .send(frames_to_zmq_message(
                wire_message_to_frames(shutdown_request).frames,
            ))
            .await
            .expect("control dealer should send shutdown request");
        tokio::time::sleep(Duration::from_millis(30)).await;
        shell
            .send(frames_to_zmq_message(
                wire_message_to_frames(execute_request).frames,
            ))
            .await
            .expect("shell dealer should send queued execute request");
        tokio::time::sleep(Duration::from_millis(70)).await;
    });

    let first_event = wait_for_next_socket_event(&mut driver, backend.transport_mut());
    assert_eq!(
        first_event,
        crate::KernelSocketPollEvent::Channel(KernelChannel::Control)
    );

    let report = driver
        .pump_until_idle(backend.transport_mut())
        .expect("driver should stop cleanly after shutdown");
    assert_eq!(report.control_messages, 0);
    assert_eq!(report.shell_messages, 0);
    assert_eq!(report.stdin_messages, 0);
    assert_eq!(report.heartbeat_messages, 0);
    assert!(report.stopped_for_shutdown);
    assert!(driver.event_pump().adapter().shutdown_requested());
    assert_eq!(
        driver.event_pump().adapter().lifecycle_state(),
        crate::KernelLifecycleState::Shutdown
    );

    let control_reply_wire = runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(2), control.recv())
            .await
            .expect("shutdown reply should arrive before timeout")
            .expect("shutdown reply should decode")
    });
    let control_reply = decode_response_message(
        &frames_to_wire_message(
            KernelChannel::Control,
            &bootstrap.wire_config().delimiter,
            zmq_message_to_frames(control_reply_wire),
        )
        .expect("shutdown reply frames should convert"),
    )
    .expect("shutdown reply should decode");
    assert_eq!(control_reply.header.username, "rum-kernel");
    assert_eq!(
        control_reply.header.protocol_version,
        shutdown_header.protocol_version
    );
    assert_eq!(control_reply.parent_header, Some(shutdown_header.clone()));

    match control_reply.response {
        KernelResponse::Shutdown(response) => {
            assert_eq!(response.status, KernelReplyStatus::Ok);
            assert!(response.restart);
            assert_eq!(
                response.lifecycle_state,
                crate::KernelLifecycleState::ShutdownRequested
            );
        }
        other => panic!("expected shutdown reply, got {other:?}"),
    }

    let shell_timeout = runtime
        .block_on(async { tokio::time::timeout(Duration::from_millis(200), shell.recv()).await });
    assert!(
        shell_timeout.is_err(),
        "queued shell work should remain pending after shutdown"
    );
}
