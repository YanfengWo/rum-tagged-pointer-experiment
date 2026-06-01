use rum_runtime::{
    RuntimeInputKind, RuntimeInputRequest, RuntimeMessage, SessionEvalResponse,
    SessionRequestError, SessionRequestFailure, TraceEvent,
};

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
