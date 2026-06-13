use std::{
    collections::BTreeMap,
    time::{SystemTime, UNIX_EPOCH},
};

use rum_host::{HostEvalSuccess, HostFailure, HostInputSuspension, HostInspectResponse};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelExecuteRequest {
    pub code: String,
    pub store_history: bool,
    pub allow_stdin: bool,
}

impl KernelExecuteRequest {
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            store_history: true,
            allow_stdin: true,
        }
    }

    pub fn without_history(mut self) -> Self {
        self.store_history = false;
        self
    }

    pub fn without_stdin(mut self) -> Self {
        self.allow_stdin = false;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KernelExecuteOutcome {
    Success(HostEvalSuccess),
    InputRequired(HostInputSuspension),
    Failure(HostFailure),
    WireDecoded,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KernelReplyStatus {
    Ok,
    Error,
    Abort,
}

impl KernelReplyStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Error => "error",
            Self::Abort => "abort",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KernelStreamName {
    Stdout,
    Stderr,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KernelExecutionState {
    Starting,
    Busy,
    Idle,
}

impl KernelExecutionState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Starting => "starting",
            Self::Busy => "busy",
            Self::Idle => "idle",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KernelLifecycleState {
    Running,
    InterruptRequested,
    ShutdownRequested,
    Shutdown,
}

impl KernelLifecycleState {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::InterruptRequested => "interrupt_requested",
            Self::ShutdownRequested => "shutdown_requested",
            Self::Shutdown => "shutdown",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KernelChannel {
    Shell,
    Control,
    IoPub,
    Stdin,
}

impl KernelChannel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Shell => "shell",
            Self::Control => "control",
            Self::IoPub => "iopub",
            Self::Stdin => "stdin",
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KernelMetadata {
    pub items: BTreeMap<String, String>,
}

impl KernelMetadata {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.items.insert(key.into(), value.into());
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.items.get(key).map(String::as_str)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KernelMimeBundle {
    pub items: BTreeMap<String, String>,
}

impl KernelMimeBundle {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn text_plain(value: impl Into<String>) -> Self {
        let mut bundle = Self::new();
        bundle.insert("text/plain", value);
        bundle
    }

    pub fn insert(&mut self, mime_type: impl Into<String>, value: impl Into<String>) {
        self.items.insert(mime_type.into(), value.into());
    }

    pub fn get(&self, mime_type: &str) -> Option<&str> {
        self.items.get(mime_type).map(String::as_str)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelStreamOutput {
    pub name: KernelStreamName,
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelStatusOutput {
    pub execution_state: KernelExecutionState,
    pub lifecycle_state: KernelLifecycleState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelExecuteInput {
    pub execution_count: Option<u64>,
    pub code: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelInputRequest {
    pub prompt: String,
    pub password: bool,
}

impl KernelInputRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            password: false,
        }
    }

    pub fn password(mut self) -> Self {
        self.password = true;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelInputReply {
    pub value: String,
}

impl KernelInputReply {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelExecutePayload {
    pub source: String,
    pub data: KernelMimeBundle,
    pub metadata: KernelMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelExecuteResult {
    pub execution_count: Option<u64>,
    pub data: KernelMimeBundle,
    pub metadata: KernelMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelDisplayOutput {
    pub data: KernelMimeBundle,
    pub metadata: KernelMetadata,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelErrorOutput {
    pub error_name: String,
    pub error_value: String,
    pub rendered: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KernelOutput {
    Stream(KernelStreamOutput),
    ExecuteResult(KernelExecuteResult),
    DisplayData(KernelDisplayOutput),
    Error(KernelErrorOutput),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KernelIoPubPayload {
    Status(KernelStatusOutput),
    ExecuteInput(KernelExecuteInput),
    Stream(KernelStreamOutput),
    ExecuteResult(KernelExecuteResult),
    DisplayData(KernelDisplayOutput),
    Error(KernelErrorOutput),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelExecuteResponse {
    pub status: KernelReplyStatus,
    pub execution_count: Option<u64>,
    pub payload: Vec<KernelExecutePayload>,
    pub user_expressions: BTreeMap<String, KernelMimeBundle>,
    pub error: Option<KernelErrorOutput>,
    pub outputs: Vec<KernelOutput>,
    pub outcome: KernelExecuteOutcome,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KernelStdinError {
    StdinNotAllowed,
    AlreadyAwaitingInput,
    NoPendingInputRequest,
    ReplyParentMismatch { expected: String, found: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelInspectRequest {
    pub code: String,
    pub cursor_offset: usize,
}

impl KernelInspectRequest {
    pub fn new(code: impl Into<String>, cursor_offset: usize) -> Self {
        Self {
            code: code.into(),
            cursor_offset,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelInspectResponse {
    pub status: KernelReplyStatus,
    pub target: Option<String>,
    pub found: bool,
    pub data: KernelMimeBundle,
    pub metadata: KernelMetadata,
    pub result: Option<HostInspectResponse>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelCompleteRequest {
    pub code: String,
    pub cursor_offset: usize,
}

impl KernelCompleteRequest {
    pub fn new(code: impl Into<String>, cursor_offset: usize) -> Self {
        Self {
            code: code.into(),
            cursor_offset,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelCompleteResponse {
    pub status: KernelReplyStatus,
    pub replacement_start: usize,
    pub replacement_end: usize,
    pub matches: Vec<String>,
    pub metadata: KernelMetadata,
    pub prefix: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KernelInfoRequest;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelLanguageInfo {
    pub name: String,
    pub version: String,
    pub mimetype: String,
    pub file_extension: String,
    pub pygments_lexer: String,
    pub codemirror_mode: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelHelpLink {
    pub text: String,
    pub url: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelInfoResponse {
    pub status: KernelReplyStatus,
    pub protocol_version: String,
    pub implementation: String,
    pub implementation_version: String,
    pub banner: String,
    pub language_info: KernelLanguageInfo,
    pub help_links: Vec<KernelHelpLink>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KernelShutdownRequest {
    pub restart: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct KernelShutdownResponse {
    pub status: KernelReplyStatus,
    pub restart: bool,
    pub lifecycle_state: KernelLifecycleState,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KernelInterruptRequest;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelInterruptResponse {
    pub status: KernelReplyStatus,
    pub interrupt_requested: bool,
    pub lifecycle_state: KernelLifecycleState,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelIsCompleteRequest {
    pub code: String,
}

impl KernelIsCompleteRequest {
    pub fn new(code: impl Into<String>) -> Self {
        Self { code: code.into() }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KernelIsCompleteStatus {
    Complete,
    Incomplete,
    Invalid,
    Unknown,
}

impl KernelIsCompleteStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Complete => "complete",
            Self::Incomplete => "incomplete",
            Self::Invalid => "invalid",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelIsCompleteResponse {
    pub reply_status: KernelReplyStatus,
    pub completion_status: KernelIsCompleteStatus,
    pub indent: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KernelHistoryRequest;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelHistoryEntry {
    pub session: u64,
    pub line_number: u64,
    pub input: String,
    pub output: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelHistoryResponse {
    pub status: KernelReplyStatus,
    pub history: Vec<KernelHistoryEntry>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct KernelCommInfoRequest {
    pub target_name: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelCommInfoResponse {
    pub status: KernelReplyStatus,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelDebugRequest {
    pub request_seq: Option<u64>,
    pub command: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelDebugResponse {
    pub status: KernelReplyStatus,
    pub request_seq: Option<u64>,
    pub command: Option<String>,
    pub message: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelIgnoredRequest {
    pub msg_type: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KernelRequest {
    Execute(KernelExecuteRequest),
    Inspect(KernelInspectRequest),
    Complete(KernelCompleteRequest),
    IsComplete(KernelIsCompleteRequest),
    History(KernelHistoryRequest),
    KernelInfo(KernelInfoRequest),
    CommInfo(KernelCommInfoRequest),
    Debug(KernelDebugRequest),
    Ignored(KernelIgnoredRequest),
    Interrupt(KernelInterruptRequest),
    Shutdown(KernelShutdownRequest),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelMessageHeader {
    pub msg_id: String,
    pub session_id: String,
    pub username: String,
    pub msg_type: String,
    pub protocol_version: String,
    pub date: String,
}

impl KernelMessageHeader {
    pub fn new(
        msg_id: impl Into<String>,
        session_id: impl Into<String>,
        username: impl Into<String>,
        msg_type: impl Into<String>,
    ) -> Self {
        Self {
            msg_id: msg_id.into(),
            session_id: session_id.into(),
            username: username.into(),
            msg_type: msg_type.into(),
            protocol_version: "5.3".to_string(),
            date: current_timestamp_rfc3339(),
        }
    }

    pub fn with_date(mut self, date: impl Into<String>) -> Self {
        self.date = date.into();
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelRequestEnvelope {
    pub channel: KernelChannel,
    pub header: KernelMessageHeader,
    pub parent_header: Option<KernelMessageHeader>,
    pub metadata: KernelMetadata,
    pub request: KernelRequest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelInputRequestEnvelope {
    pub channel: KernelChannel,
    pub header: KernelMessageHeader,
    pub parent_header: Option<KernelMessageHeader>,
    pub metadata: KernelMetadata,
    pub request: KernelInputRequest,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelInputReplyEnvelope {
    pub channel: KernelChannel,
    pub header: KernelMessageHeader,
    pub parent_header: Option<KernelMessageHeader>,
    pub metadata: KernelMetadata,
    pub reply: KernelInputReply,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum KernelResponse {
    Execute(KernelExecuteResponse),
    Inspect(KernelInspectResponse),
    Complete(KernelCompleteResponse),
    IsComplete(KernelIsCompleteResponse),
    History(KernelHistoryResponse),
    KernelInfo(KernelInfoResponse),
    CommInfo(KernelCommInfoResponse),
    Debug(KernelDebugResponse),
    Interrupt(KernelInterruptResponse),
    Shutdown(KernelShutdownResponse),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelResponseEnvelope {
    pub channel: KernelChannel,
    pub header: KernelMessageHeader,
    pub parent_header: Option<KernelMessageHeader>,
    pub metadata: KernelMetadata,
    pub response: KernelResponse,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelIoPubEnvelope {
    pub channel: KernelChannel,
    pub header: KernelMessageHeader,
    pub parent_header: Option<KernelMessageHeader>,
    pub metadata: KernelMetadata,
    pub payload: KernelIoPubPayload,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KernelDispatchResult {
    pub reply: Option<KernelResponseEnvelope>,
    pub iopub: Vec<KernelIoPubEnvelope>,
    pub stdin: Vec<KernelInputRequestEnvelope>,
}

fn current_timestamp_rfc3339() -> String {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let total_seconds = duration.as_secs();
    let days = (total_seconds / 86_400) as i64;
    let seconds_of_day = total_seconds % 86_400;
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;

    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    if month <= 2 {
        year += 1;
    }

    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}
