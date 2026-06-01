use super::*;

pub(super) fn parse_reply_status(value: &str) -> Result<KernelReplyStatus, KernelWireError> {
    match value {
        "ok" => Ok(KernelReplyStatus::Ok),
        "error" => Ok(KernelReplyStatus::Error),
        "abort" => Ok(KernelReplyStatus::Abort),
        other => Err(KernelWireError::InvalidField {
            context: "content",
            field: "status",
            value: other.to_string(),
        }),
    }
}

pub(super) fn parse_is_complete_status(
    value: &str,
) -> Result<KernelIsCompleteStatus, KernelWireError> {
    match value {
        "complete" => Ok(KernelIsCompleteStatus::Complete),
        "incomplete" => Ok(KernelIsCompleteStatus::Incomplete),
        "invalid" => Ok(KernelIsCompleteStatus::Invalid),
        "unknown" => Ok(KernelIsCompleteStatus::Unknown),
        other => Err(KernelWireError::InvalidField {
            context: "content",
            field: "status",
            value: other.to_string(),
        }),
    }
}

pub(super) fn parse_execution_state(
    value: &str,
) -> Result<crate::KernelExecutionState, KernelWireError> {
    match value {
        "starting" => Ok(crate::KernelExecutionState::Starting),
        "busy" => Ok(crate::KernelExecutionState::Busy),
        "idle" => Ok(crate::KernelExecutionState::Idle),
        other => Err(KernelWireError::InvalidField {
            context: "content",
            field: "execution_state",
            value: other.to_string(),
        }),
    }
}

pub(super) fn parse_lifecycle_state(
    value: &str,
) -> Result<crate::KernelLifecycleState, KernelWireError> {
    match value {
        "running" => Ok(crate::KernelLifecycleState::Running),
        "interrupt_requested" => Ok(crate::KernelLifecycleState::InterruptRequested),
        "shutdown_requested" => Ok(crate::KernelLifecycleState::ShutdownRequested),
        "shutdown" => Ok(crate::KernelLifecycleState::Shutdown),
        other => Err(KernelWireError::InvalidField {
            context: "content",
            field: "lifecycle_state",
            value: other.to_string(),
        }),
    }
}

pub(super) fn parse_stream_name(value: &str) -> Result<KernelStreamName, KernelWireError> {
    match value {
        "stdout" => Ok(KernelStreamName::Stdout),
        "stderr" => Ok(KernelStreamName::Stderr),
        other => Err(KernelWireError::InvalidField {
            context: "content",
            field: "name",
            value: other.to_string(),
        }),
    }
}
