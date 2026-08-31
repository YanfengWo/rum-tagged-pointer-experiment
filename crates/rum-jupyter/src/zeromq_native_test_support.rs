use std::{
    io::ErrorKind,
    net::TcpListener,
    thread,
    time::{Duration, Instant},
};

use super::*;

fn loopback_ports() -> Option<Vec<u16>> {
    let mut listeners = Vec::with_capacity(5);
    for _ in 0..5 {
        match TcpListener::bind("127.0.0.1:0") {
            Ok(listener) => listeners.push(listener),
            Err(error) if error.kind() == ErrorKind::PermissionDenied => {
                eprintln!(
                    "skipping native ZeroMQ test: loopback socket binding is not permitted"
                );
                return None;
            }
            Err(error) => panic!("ephemeral port should bind: {error}"),
        }
    }

    let ports = listeners
        .iter()
        .map(|listener| {
            listener
                .local_addr()
                .expect("listener should expose a local address")
                .port()
        })
        .collect::<Vec<_>>();
    drop(listeners);
    Some(ports)
}

fn loopback_connection_with_signature(
    key: &str,
    signature_scheme: crate::KernelSignatureScheme,
) -> Option<KernelBootstrapConfig> {
    let ports = loopback_ports()?;

    Some(KernelBootstrapConfig::from_connection_file(KernelConnectionFile {
        transport: crate::KernelTransportKind::Tcp,
        ip: "127.0.0.1".to_string(),
        shell_port: ports[0],
        iopub_port: ports[1],
        stdin_port: ports[2],
        control_port: ports[3],
        hb_port: ports[4],
        key: key.to_string(),
        signature_scheme,
        kernel_name: Some("rum".to_string()),
    })
    .with_username("rum-kernel")
    .with_session_validation("frontend-session"))
}

pub(super) fn loopback_connection() -> Option<KernelBootstrapConfig> {
    loopback_connection_with_signature("", crate::KernelSignatureScheme::None)
}

pub(super) fn signed_loopback_connection() -> Option<KernelBootstrapConfig> {
    loopback_connection_with_signature(
        "secret",
        crate::KernelSignatureScheme::HmacSha256,
    )
}

pub(super) fn test_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("test runtime should build")
}

pub(super) fn acknowledge_iopub_subscription(
    runtime: &tokio::runtime::Runtime,
    heartbeat_endpoint: &str,
    driver: &mut KernelSocketDriver,
    transport: &mut ZeroMqSocketSession<NativeZeroMqSocket>,
) {
    let mut heartbeat = runtime.block_on(async {
        let mut heartbeat = ReqSocket::new();
        heartbeat
            .connect(heartbeat_endpoint)
            .await
            .expect("iopub subscription barrier should connect");
        heartbeat
    });
    let barrier = b"iopub-subscription-ready".to_vec();

    runtime.block_on(async {
        heartbeat
            .send(ZmqMessage::from(barrier.clone()))
            .await
            .expect("iopub subscription barrier should send");
    });
    wait_for_socket_event(
        driver,
        transport,
        crate::KernelSocketPollEvent::Heartbeat,
    );

    let reply = runtime.block_on(async {
        tokio::time::timeout(Duration::from_secs(2), heartbeat.recv())
            .await
            .expect("iopub subscription barrier reply should arrive before timeout")
            .expect("iopub subscription barrier reply should decode")
    });
    assert_eq!(zmq_message_to_frames(reply), vec![barrier]);
}

pub(super) fn wait_for_socket_event(
    driver: &mut KernelSocketDriver,
    transport: &mut ZeroMqSocketSession<NativeZeroMqSocket>,
    expected: crate::KernelSocketPollEvent,
) {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        let event = driver
            .pump_once(transport)
            .expect("socket driver should keep pumping");
        if event == Some(expected) {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for socket event {expected:?}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

pub(super) fn wait_for_pending_socket_channels(
    transport: &ZeroMqSocketSession<NativeZeroMqSocket>,
    channels: &[KernelChannel],
) {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if channels
            .iter()
            .all(|&channel| transport.has_pending_channel(channel))
        {
            return;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for pending socket channels {channels:?}"
        );
        thread::sleep(Duration::from_millis(10));
    }
}

pub(super) fn wait_for_next_socket_event(
    driver: &mut KernelSocketDriver,
    transport: &mut ZeroMqSocketSession<NativeZeroMqSocket>,
) -> crate::KernelSocketPollEvent {
    let deadline = Instant::now() + Duration::from_secs(2);
    loop {
        if let Some(event) = driver
            .pump_once(transport)
            .expect("socket driver should keep pumping")
        {
            return event;
        }
        assert!(
            Instant::now() < deadline,
            "timed out waiting for the next socket event"
        );
        thread::sleep(Duration::from_millis(10));
    }
}
