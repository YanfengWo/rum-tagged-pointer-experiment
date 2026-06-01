use rum_runtime::{
    DefinitionCatalogEntry, RuntimeInputKind, RuntimeInputRequest, RuntimeMessage,
    SessionCompletenessStatus, SessionEvalResponse, SessionInspectResponse, SessionRequestError,
    SessionRequestFailure, SymbolInfo, TraceEvent,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostParseRequest {
    pub source: String,
}

impl HostParseRequest {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
        }
    }
}

impl From<&str> for HostParseRequest {
    fn from(source: &str) -> Self {
        Self::new(source)
    }
}

impl From<String> for HostParseRequest {
    fn from(source: String) -> Self {
        Self::new(source)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostParseResponse {
    pub source: String,
    pub syntax: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostEvalRequest {
    pub source: String,
}

impl HostEvalRequest {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
        }
    }
}

impl From<&str> for HostEvalRequest {
    fn from(source: &str) -> Self {
        Self::new(source)
    }
}

impl From<String> for HostEvalRequest {
    fn from(source: String) -> Self {
        Self::new(source)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostEvalOutcome {
    Success(HostEvalSuccess),
    InputRequired(HostInputSuspension),
    Failure(HostFailure),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostEvalSuccess {
    pub value: HostValue,
    pub messages: Vec<HostMessage>,
    pub prints: Vec<String>,
    pub trace: Vec<HostTraceEvent>,
    pub outputs: Vec<HostStructuredOutput>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostInputSuspension {
    pub request: HostInputRequest,
    pub messages: Vec<HostMessage>,
    pub prints: Vec<String>,
    pub trace: Vec<HostTraceEvent>,
    pub outputs: Vec<HostStructuredOutput>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostFailure {
    pub code: HostFailureCode,
    pub message: String,
    pub messages: Vec<HostMessage>,
    pub prints: Vec<String>,
    pub trace: Vec<HostTraceEvent>,
    pub outputs: Vec<HostStructuredOutput>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostFailureCode {
    Syntax,
    InvalidInteger,
    Io,
    LoadPathNotFound,
    EvaluationCycleDetected,
    RecursionLimitExceeded,
    TimeConstraintExceeded,
    MemoryConstraintExceeded,
    IterationLimitExceeded,
    EvaluationPassLimitExceeded,
    Exit,
    Other,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostValue {
    pub text: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostMessage {
    pub symbol: String,
    pub tag: String,
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostInputKind {
    InputString,
    Input,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostInputRequest {
    pub kind: HostInputKind,
    pub prompt: String,
    pub password: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HostTraceEvent {
    EvaluationPass {
        before: String,
        after: String,
    },
    DefinitionStored {
        symbol: String,
        kind: String,
        value: String,
    },
    RuleApplied {
        source: String,
        input: String,
        rule: String,
        result: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostStructuredOutput {
    pub kind: HostStructuredOutputKind,
    pub text: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostStructuredOutputKind {
    Result,
    Print,
    Message,
    Trace,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostCompletionRequest {
    pub source: String,
    pub cursor_offset: usize,
}

impl HostCompletionRequest {
    pub fn new(source: impl Into<String>, cursor_offset: usize) -> Self {
        Self {
            source: source.into(),
            cursor_offset,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostCompletionResponse {
    pub prefix: String,
    pub matches: Vec<String>,
    pub replacement_start: usize,
    pub replacement_end: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostInspectRequest {
    pub source: String,
    pub cursor_offset: usize,
}

impl HostInspectRequest {
    pub fn new(source: impl Into<String>, cursor_offset: usize) -> Self {
        Self {
            source: source.into(),
            cursor_offset,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostInspectResponse {
    pub target: Option<String>,
    pub found: bool,
    pub info: Option<HostSymbolInfo>,
    pub definitions: Vec<HostDefinition>,
    pub message_catalog: Vec<HostMessageTemplate>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostSymbolInfo {
    pub qualified_name: String,
    pub context: String,
    pub display_name: String,
    pub summary: Option<String>,
    pub is_builtin: bool,
    pub attributes: Vec<String>,
    pub has_own_value: bool,
    pub has_delayed_value: bool,
    pub has_up_values: bool,
    pub has_down_values: bool,
    pub has_sub_values: bool,
    pub message_tags: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostDefinition {
    pub kind: String,
    pub lhs: String,
    pub rhs: String,
    pub delayed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostMessageTemplate {
    pub message_name: String,
    pub qualified_symbol_name: String,
    pub tag: String,
    pub template: String,
    pub delayed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostIsCompleteRequest {
    pub source: String,
}

impl HostIsCompleteRequest {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
        }
    }
}

impl From<&str> for HostIsCompleteRequest {
    fn from(source: &str) -> Self {
        Self::new(source)
    }
}

impl From<String> for HostIsCompleteRequest {
    fn from(source: String) -> Self {
        Self::new(source)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostIsCompleteResponse {
    pub status: HostCompletenessStatus,
    pub indent: Option<String>,
    pub diagnostic: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostCompletenessStatus {
    Complete,
    Incomplete,
    Invalid,
}

impl From<RuntimeMessage> for HostMessage {
    fn from(message: RuntimeMessage) -> Self {
        Self {
            symbol: message.symbol,
            tag: message.tag,
            text: message.text,
        }
    }
}

impl From<RuntimeInputRequest> for HostInputRequest {
    fn from(request: RuntimeInputRequest) -> Self {
        Self {
            kind: match request.kind {
                RuntimeInputKind::InputString => HostInputKind::InputString,
                RuntimeInputKind::Input => HostInputKind::Input,
            },
            prompt: request.prompt,
            password: request.password,
        }
    }
}

impl From<TraceEvent> for HostTraceEvent {
    fn from(event: TraceEvent) -> Self {
        match event {
            TraceEvent::EvaluationPass { before, after } => Self::EvaluationPass { before, after },
            TraceEvent::DefinitionStored {
                symbol,
                kind,
                value,
            } => Self::DefinitionStored {
                symbol,
                kind: format!("{kind:?}"),
                value,
            },
            TraceEvent::RuleApplied {
                source,
                input,
                rule,
                result,
            } => Self::RuleApplied {
                source: format!("{source:?}"),
                input,
                rule,
                result,
            },
        }
    }
}

impl From<SymbolInfo> for HostSymbolInfo {
    fn from(info: SymbolInfo) -> Self {
        Self {
            qualified_name: info.qualified_name,
            context: info.context,
            display_name: info.display_name,
            summary: info.summary,
            is_builtin: info.is_builtin,
            attributes: info.attributes,
            has_own_value: info.has_own_value,
            has_delayed_value: info.has_delayed_value,
            has_up_values: info.has_up_values,
            has_down_values: info.has_down_values,
            has_sub_values: info.has_sub_values,
            message_tags: info.message_tags,
        }
    }
}

impl From<DefinitionCatalogEntry> for HostDefinition {
    fn from(entry: DefinitionCatalogEntry) -> Self {
        Self {
            kind: format!("{:?}", entry.kind),
            lhs: entry.lhs.to_string(),
            rhs: entry.rhs.to_string(),
            delayed: entry.delayed,
        }
    }
}

pub(crate) fn eval_success_from_runtime(response: SessionEvalResponse) -> HostEvalSuccess {
    let value = HostValue {
        text: response.value.to_string(),
    };
    let messages = host_messages(response.messages);
    let prints = response.prints;
    let trace = host_trace(response.trace);
    let outputs = structured_outputs(Some(&value), &messages, &prints, &trace);

    HostEvalSuccess {
        value,
        messages,
        prints,
        trace,
        outputs,
    }
}

pub(crate) fn eval_failure_from_runtime(failure: SessionRequestFailure) -> HostEvalOutcome {
    if let SessionRequestError::InputRequired { request } = &failure.error {
        let request = request.clone().into();
        let messages = host_messages(failure.messages);
        let prints = failure.prints;
        let trace = host_trace(failure.trace);
        let outputs = structured_outputs(None, &messages, &prints, &trace);

        return HostEvalOutcome::InputRequired(HostInputSuspension {
            request,
            messages,
            prints,
            trace,
            outputs,
        });
    }

    HostEvalOutcome::Failure(host_failure_from_runtime(failure))
}

pub(crate) fn host_failure_from_runtime(failure: SessionRequestFailure) -> HostFailure {
    let code = failure_code(&failure.error);
    let message = failure.error.to_string();
    let messages = host_messages(failure.messages);
    let prints = failure.prints;
    let trace = host_trace(failure.trace);
    let outputs = structured_outputs(None, &messages, &prints, &trace);

    HostFailure {
        code,
        message,
        messages,
        prints,
        trace,
        outputs,
    }
}

pub(crate) fn inspect_response_from_runtime(
    target: Option<String>,
    response: Option<SessionInspectResponse>,
) -> HostInspectResponse {
    let Some(response) = response else {
        return HostInspectResponse {
            target,
            found: false,
            info: None,
            definitions: Vec::new(),
            message_catalog: Vec::new(),
        };
    };

    HostInspectResponse {
        target,
        found: true,
        info: Some(response.info.into()),
        definitions: response.definitions.into_iter().map(Into::into).collect(),
        message_catalog: response
            .message_catalog
            .into_iter()
            .map(|entry| HostMessageTemplate {
                message_name: entry.message_name,
                qualified_symbol_name: entry.qualified_symbol_name,
                tag: entry.tag,
                template: entry.template.to_string(),
                delayed: entry.delayed,
            })
            .collect(),
    }
}

pub(crate) fn completeness_status(status: SessionCompletenessStatus) -> HostCompletenessStatus {
    match status {
        SessionCompletenessStatus::Complete => HostCompletenessStatus::Complete,
        SessionCompletenessStatus::Incomplete => HostCompletenessStatus::Incomplete,
        SessionCompletenessStatus::Invalid => HostCompletenessStatus::Invalid,
    }
}

fn failure_code(error: &SessionRequestError) -> HostFailureCode {
    match error {
        SessionRequestError::Syntax(_) => HostFailureCode::Syntax,
        SessionRequestError::InvalidInteger { .. } => HostFailureCode::InvalidInteger,
        SessionRequestError::Io { .. } => HostFailureCode::Io,
        SessionRequestError::LoadPathNotFound { .. } => HostFailureCode::LoadPathNotFound,
        SessionRequestError::EvaluationCycleDetected { .. } => {
            HostFailureCode::EvaluationCycleDetected
        }
        SessionRequestError::RecursionLimitExceeded { .. } => {
            HostFailureCode::RecursionLimitExceeded
        }
        SessionRequestError::TimeConstraintExceeded => HostFailureCode::TimeConstraintExceeded,
        SessionRequestError::MemoryConstraintExceeded { .. } => {
            HostFailureCode::MemoryConstraintExceeded
        }
        SessionRequestError::IterationLimitExceeded { .. } => {
            HostFailureCode::IterationLimitExceeded
        }
        SessionRequestError::EvaluationPassLimitExceeded { .. } => {
            HostFailureCode::EvaluationPassLimitExceeded
        }
        SessionRequestError::Exit { .. } => HostFailureCode::Exit,
        SessionRequestError::InputRequired { .. } | SessionRequestError::Other { .. } => {
            HostFailureCode::Other
        }
    }
}

fn host_messages(messages: Vec<RuntimeMessage>) -> Vec<HostMessage> {
    messages.into_iter().map(Into::into).collect()
}

fn host_trace(trace: Vec<TraceEvent>) -> Vec<HostTraceEvent> {
    trace.into_iter().map(Into::into).collect()
}

fn structured_outputs(
    value: Option<&HostValue>,
    messages: &[HostMessage],
    prints: &[String],
    trace: &[HostTraceEvent],
) -> Vec<HostStructuredOutput> {
    let mut outputs = Vec::new();

    outputs.extend(prints.iter().cloned().map(|text| HostStructuredOutput {
        kind: HostStructuredOutputKind::Print,
        text,
    }));
    outputs.extend(messages.iter().map(|message| HostStructuredOutput {
        kind: HostStructuredOutputKind::Message,
        text: format!("{}::{}: {}", message.symbol, message.tag, message.text),
    }));
    outputs.extend(trace.iter().map(|event| HostStructuredOutput {
        kind: HostStructuredOutputKind::Trace,
        text: format!("{event:?}"),
    }));

    if let Some(value) = value {
        outputs.push(HostStructuredOutput {
            kind: HostStructuredOutputKind::Result,
            text: value.text.clone(),
        });
    }

    outputs
}
