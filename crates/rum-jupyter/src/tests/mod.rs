use rum_host::{HostEvalOutcome, HostSession as Session, HostSessionBuilder};

use super::{
    InMemoryTransport, KernelAdapter, KernelBootstrapConfig, KernelChannel, KernelCommInfoRequest,
    KernelCompleteRequest, KernelConnectionFile, KernelConnectionFileError, KernelDebugRequest,
    KernelEventPump, KernelExecuteOutcome, KernelExecuteRequest, KernelExecutionState,
    KernelHistoryRequest, KernelIgnoredRequest, KernelInfoRequest, KernelInputReply,
    KernelInputReplyEnvelope, KernelInputRequest, KernelInspectRequest, KernelInterruptRequest,
    KernelIoPubPayload, KernelIsCompleteRequest, KernelIsCompleteStatus, KernelLifecycleState,
    KernelMessageHeader, KernelMetadata, KernelOutput, KernelReplyStatus, KernelRequest,
    KernelRequestEnvelope, KernelResponse, KernelShutdownRequest, KernelSignaturePolicy,
    KernelSignatureScheme, KernelSocketBindMode, KernelSocketKind, KernelSocketPollEvent,
    KernelStreamName, KernelTransport, KernelTransportBackend, KernelTransportKind,
    KernelWireCodec, KernelWireConfig, KernelWireError, LocalSocketTransportBackend,
    MockSocketTransport, MockTransportBackend, decode_input_reply_message,
    decode_input_request_message, decode_iopub_message, decode_response_message,
    encode_dispatch_result, encode_input_reply_envelope, encode_request_envelope,
};

fn test_connection_file() -> KernelConnectionFile {
    KernelConnectionFile::parse_json(
        r#"{
            "transport": "tcp",
            "ip": "127.0.0.1",
            "shell_port": 57503,
            "iopub_port": 40885,
            "stdin_port": 52597,
            "control_port": 50160,
            "hb_port": 42540,
            "key": "secret",
            "signature_scheme": "hmac-sha256",
            "kernel_name": "rum"
        }"#,
    )
    .expect("connection file should parse")
}

fn eval_text(session: &mut Session, source: &str) {
    match session.eval(source) {
        HostEvalOutcome::Success(_) => {}
        other => panic!("expected host eval success for {source:?}, got {other:?}"),
    }
}

fn tracing_session() -> Session {
    HostSessionBuilder::new().with_tracing(true).build()
}

#[test]
fn crate_manifest_keeps_jupyter_on_host_facade() {
    let manifest = include_str!("../../Cargo.toml");

    assert!(manifest.contains("rum-host"));
    assert!(!manifest.contains("rum-runtime"));
    assert!(!manifest.contains("rum-stdlib"));
}

mod adapter;
mod bootstrap;
mod pump;
mod stdin;
mod wire;
