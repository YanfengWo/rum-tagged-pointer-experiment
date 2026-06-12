use std::{path::Path, thread, time::Duration};

use rum_host::HostSession;
use thiserror::Error;

use crate::{
    KernelBootstrapConfig, KernelChannel, KernelConnectionFile, KernelConnectionFileError,
    KernelTransportBackend, KernelWireError, NativeZeroMqTransportBackend, ZeroMqBackendError,
};

const DEFAULT_IDLE_SLEEP: Duration = Duration::from_millis(10);
const DEFAULT_USERNAME: &str = "rum-kernel";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeKernelLauncherConfig {
    pub username: String,
    pub idle_sleep: Duration,
}

impl Default for NativeKernelLauncherConfig {
    fn default() -> Self {
        Self {
            username: DEFAULT_USERNAME.to_string(),
            idle_sleep: DEFAULT_IDLE_SLEEP,
        }
    }
}

impl NativeKernelLauncherConfig {
    pub fn with_username(mut self, username: impl Into<String>) -> Self {
        self.username = username.into();
        self
    }

    pub fn with_idle_sleep(mut self, idle_sleep: Duration) -> Self {
        self.idle_sleep = idle_sleep;
        self
    }
}

#[derive(Debug, Error)]
pub enum NativeKernelLauncherError {
    #[error("{message}")]
    ConnectionFile { message: String },
    #[error("{message}")]
    Backend { message: String },
    #[error("{message}")]
    Wire { message: String },
}

pub fn run_native_kernel(
    connection_path: impl AsRef<Path>,
) -> Result<(), NativeKernelLauncherError> {
    run_native_kernel_with_config(connection_path, NativeKernelLauncherConfig::default())
}

pub fn run_native_kernel_with_config(
    connection_path: impl AsRef<Path>,
    config: NativeKernelLauncherConfig,
) -> Result<(), NativeKernelLauncherError> {
    let connection_path = connection_path.as_ref();
    let connection = KernelConnectionFile::read_from_path(connection_path).map_err(|error| {
        NativeKernelLauncherError::ConnectionFile {
            message: describe_connection_file_error(connection_path, &error),
        }
    })?;
    let bootstrap =
        KernelBootstrapConfig::from_connection_file(connection).with_username(config.username);
    let mut backend = NativeZeroMqTransportBackend::new_native(bootstrap).map_err(|error| {
        NativeKernelLauncherError::Backend {
            message: describe_backend_error(&error),
        }
    })?;

    let session = HostSession::new();
    let mut driver = backend.socket_driver(session);

    loop {
        let report = driver
            .pump_until_idle(backend.transport_mut())
            .map_err(|error| NativeKernelLauncherError::Wire {
                message: describe_wire_error(&error),
            })?;

        if driver.event_pump().adapter().shutdown_requested() {
            return Ok(());
        }

        if report.total_messages() == 0 {
            thread::sleep(config.idle_sleep);
        }
    }
}

fn describe_connection_file_error(
    connection_path: &Path,
    error: &KernelConnectionFileError,
) -> String {
    match error {
        KernelConnectionFileError::Io { path, message } => {
            format!("failed to read Jupyter connection file {path}: {message}")
        }
        KernelConnectionFileError::MissingField { field } => format!(
            "invalid Jupyter connection file {}: missing field `{field}`",
            connection_path.display()
        ),
        KernelConnectionFileError::InvalidField { field, value } => format!(
            "invalid Jupyter connection file {}: field `{field}` has invalid value `{value}`",
            connection_path.display()
        ),
        KernelConnectionFileError::InvalidJson { message } => format!(
            "invalid Jupyter connection file {}: {message}",
            connection_path.display()
        ),
    }
}

fn describe_backend_error(error: &ZeroMqBackendError) -> String {
    match error {
        ZeroMqBackendError::SocketCreation { kind, message } => {
            format!("failed to create native {kind} ZeroMQ socket: {message}")
        }
        ZeroMqBackendError::SocketSetup {
            endpoint,
            operation,
            message,
        } => format!("failed to {operation} ZeroMQ socket at {endpoint}: {message}"),
        ZeroMqBackendError::InvalidWireFrames { channel, message } => format!(
            "native ZeroMQ backend received invalid {} wire frames during startup: {message}",
            channel_name(*channel)
        ),
    }
}

fn describe_wire_error(error: &KernelWireError) -> String {
    format!("jupyter kernel transport error: {error:?}")
}

const fn channel_name(channel: KernelChannel) -> &'static str {
    match channel {
        KernelChannel::Shell => "shell",
        KernelChannel::Control => "control",
        KernelChannel::IoPub => "iopub",
        KernelChannel::Stdin => "stdin",
    }
}
