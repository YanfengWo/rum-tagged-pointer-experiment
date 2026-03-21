use std::{
    net::TcpListener,
    thread,
    time::{Duration, Instant},
};

use super::*;

pub(super) fn loopback_connection() -> KernelBootstrapConfig {
    let listeners = (0..5)
        .map(|_| TcpListener::bind("127.0.0.1:0").expect("ephemeral port should bind"))
        .collect::<Vec<_>>();
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

    KernelBootstrapConfig::from_connection_file(KernelConnectionFile {
        transport: crate::KernelTransportKind::Tcp,
        ip: "127.0.0.1".to_string(),
        shell_port: ports[0],
        iopub_port: ports[1],
        stdin_port: ports[2],
        control_port: ports[3],
        hb_port: ports[4],
        key: "".to_string(),
        signature_scheme: crate::KernelSignatureScheme::None,
        kernel_name: Some("rum".to_string()),
    })
    .with_username("rum-kernel")
    .with_session_validation("frontend-session")
}

pub(super) fn signed_loopback_connection() -> KernelBootstrapConfig {
    let listeners = (0..5)
        .map(|_| TcpListener::bind("127.0.0.1:0").expect("ephemeral port should bind"))
        .collect::<Vec<_>>();
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

    KernelBootstrapConfig::from_connection_file(KernelConnectionFile {
        transport: crate::KernelTransportKind::Tcp,
        ip: "127.0.0.1".to_string(),
        shell_port: ports[0],
        iopub_port: ports[1],
        stdin_port: ports[2],
        control_port: ports[3],
        hb_port: ports[4],
        key: "secret".to_string(),
        signature_scheme: crate::KernelSignatureScheme::HmacSha256,
        kernel_name: Some("rum".to_string()),
    })
    .with_username("rum-kernel")
    .with_session_validation("frontend-session")
}

pub(super) fn test_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("test runtime should build")
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
