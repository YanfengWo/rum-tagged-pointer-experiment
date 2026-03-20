use rum_runtime::{
    RuleApplicationSource, RuntimeInputRequest, RuntimeMessage, Session, SessionCompletenessStatus,
    SessionInspectResponse, SessionRequestError, SessionRequestFailure, TraceEvent,
};

use crate::{
    KernelChannel, KernelCompleteRequest, KernelCompleteResponse, KernelDispatchResult,
    KernelDisplayOutput, KernelErrorOutput, KernelExecuteInput, KernelExecuteOutcome,
    KernelExecutePayload, KernelExecuteRequest, KernelExecuteResponse, KernelExecuteResult,
    KernelExecutionState, KernelHelpLink, KernelInfoRequest, KernelInfoResponse, KernelInputReply,
    KernelInputReplyEnvelope, KernelInputRequest, KernelInputRequestEnvelope, KernelInspectRequest,
    KernelInspectResponse, KernelInterruptRequest, KernelInterruptResponse, KernelIoPubEnvelope,
    KernelIoPubPayload, KernelIsCompleteRequest, KernelIsCompleteResponse, KernelIsCompleteStatus,
    KernelLanguageInfo, KernelLifecycleState, KernelMessageHeader, KernelMetadata,
    KernelMimeBundle, KernelOutput, KernelReplyStatus, KernelRequest, KernelRequestEnvelope,
    KernelResponse, KernelResponseEnvelope, KernelShutdownRequest, KernelShutdownResponse,
    KernelStatusOutput, KernelStdinError, KernelStreamName, KernelStreamOutput,
};

pub struct KernelAdapter {
    session: Session,
    execution_count: u64,
    next_message_id: u64,
    execution_state: KernelExecutionState,
    lifecycle_state: KernelLifecycleState,
    pending_input: Option<KernelInputRequestEnvelope>,
    pending_execute: Option<PendingExecute>,
    last_input_reply: Option<KernelInputReply>,
}

#[derive(Clone, Debug)]
struct PendingExecute {
    parent_header: KernelMessageHeader,
    request: KernelExecuteRequest,
    execution_count: Option<u64>,
}

impl KernelAdapter {
    pub fn new(session: Session) -> Self {
        Self {
            session,
            execution_count: 0,
            next_message_id: 0,
            execution_state: KernelExecutionState::Idle,
            lifecycle_state: KernelLifecycleState::Running,
            pending_input: None,
            pending_execute: None,
            last_input_reply: None,
        }
    }

    pub fn execution_count(&self) -> u64 {
        self.execution_count
    }

    pub fn execution_state(&self) -> KernelExecutionState {
        self.execution_state
    }

    pub fn lifecycle_state(&self) -> KernelLifecycleState {
        self.lifecycle_state
    }

    pub fn interrupt_requested(&self) -> bool {
        self.lifecycle_state == KernelLifecycleState::InterruptRequested
    }

    pub fn shutdown_requested(&self) -> bool {
        matches!(
            self.lifecycle_state,
            KernelLifecycleState::ShutdownRequested | KernelLifecycleState::Shutdown
        )
    }

    pub fn awaiting_stdin(&self) -> bool {
        self.pending_input.is_some()
    }

    pub fn pending_input_request(&self) -> Option<&KernelInputRequestEnvelope> {
        self.pending_input.as_ref()
    }

    pub fn take_last_input_reply(&mut self) -> Option<KernelInputReply> {
        self.last_input_reply.take()
    }

    pub fn session(&self) -> &Session {
        &self.session
    }

    pub fn session_mut(&mut self) -> &mut Session {
        &mut self.session
    }

    pub fn execute(&mut self, request: KernelExecuteRequest) -> KernelExecuteResponse {
        let execution_count = self.next_execution_count(request.store_history);
        let outcome = self.run_execute_request(&request);
        build_execute_response(execution_count, outcome)
    }

    fn next_execution_count(&mut self, store_history: bool) -> Option<u64> {
        if store_history {
            self.execution_count += 1;
            Some(self.execution_count)
        } else {
            None
        }
    }

    fn run_execute_request(&mut self, request: &KernelExecuteRequest) -> KernelExecuteOutcome {
        match self.session.eval_request(&request.code) {
            Ok(response) => KernelExecuteOutcome::Success(response),
            Err(failure) => KernelExecuteOutcome::Failure(failure),
        }
    }

    pub fn kernel_info(&self, _request: KernelInfoRequest) -> KernelInfoResponse {
        KernelInfoResponse {
            status: KernelReplyStatus::Ok,
            protocol_version: "5.3".to_string(),
            implementation: "rum".to_string(),
            implementation_version: env!("CARGO_PKG_VERSION").to_string(),
            banner: "rum experimental kernel adapter".to_string(),
            language_info: KernelLanguageInfo {
                name: "rum".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                mimetype: "text/x-rum".to_string(),
                file_extension: ".rum".to_string(),
                pygments_lexer: "text".to_string(),
                codemirror_mode: "text".to_string(),
            },
            help_links: vec![KernelHelpLink {
                text: "rum repository".to_string(),
                url: "https://example.invalid/rum".to_string(),
            }],
        }
    }

    pub fn shutdown(&mut self, request: KernelShutdownRequest) -> KernelShutdownResponse {
        self.lifecycle_state = KernelLifecycleState::ShutdownRequested;
        KernelShutdownResponse {
            status: KernelReplyStatus::Ok,
            restart: request.restart,
            lifecycle_state: self.lifecycle_state,
        }
    }

    pub fn interrupt(&mut self, _request: KernelInterruptRequest) -> KernelInterruptResponse {
        self.lifecycle_state = KernelLifecycleState::InterruptRequested;
        KernelInterruptResponse {
            status: KernelReplyStatus::Ok,
            interrupt_requested: true,
            lifecycle_state: self.lifecycle_state,
        }
    }

    pub fn inspect(&self, request: KernelInspectRequest) -> KernelInspectResponse {
        let target = symbol_at_cursor(&request.code, request.cursor_offset);
        let result = target
            .as_deref()
            .and_then(|symbol| self.session.inspect_request(symbol));
        let found = result.is_some();
        let data = result
            .as_ref()
            .map(inspect_mime_bundle)
            .unwrap_or_else(KernelMimeBundle::new);
        let metadata = inspect_metadata(target.as_deref(), found);

        KernelInspectResponse {
            status: KernelReplyStatus::Ok,
            target,
            found,
            data,
            metadata,
            result,
        }
    }

    pub fn complete(&self, request: KernelCompleteRequest) -> KernelCompleteResponse {
        let (replacement_start, replacement_end, prefix) =
            completion_prefix(&request.code, request.cursor_offset);
        let completion = self.session.complete_request(&prefix);
        let prefix = completion.prefix;
        let matches = completion.matches;

        KernelCompleteResponse {
            status: KernelReplyStatus::Ok,
            replacement_start,
            replacement_end,
            matches,
            metadata: complete_metadata(&prefix, replacement_start, replacement_end),
            prefix,
        }
    }

    pub fn is_complete(&self, request: KernelIsCompleteRequest) -> KernelIsCompleteResponse {
        let response = self.session.is_complete_request(&request.code);

        KernelIsCompleteResponse {
            reply_status: KernelReplyStatus::Ok,
            completion_status: match response.status {
                SessionCompletenessStatus::Complete => KernelIsCompleteStatus::Complete,
                SessionCompletenessStatus::Incomplete => KernelIsCompleteStatus::Incomplete,
                SessionCompletenessStatus::Invalid => KernelIsCompleteStatus::Invalid,
            },
            indent: response.indent,
        }
    }

    pub fn request_stdin_for_execute(
        &mut self,
        parent_header: &KernelMessageHeader,
        request: &KernelExecuteRequest,
        input: KernelInputRequest,
    ) -> Result<KernelInputRequestEnvelope, KernelStdinError> {
        if !request.allow_stdin {
            return Err(KernelStdinError::StdinNotAllowed);
        }

        if self.pending_input.is_some() {
            return Err(KernelStdinError::AlreadyAwaitingInput);
        }

        let metadata = input_request_metadata(parent_header, request, &input);
        let envelope = KernelInputRequestEnvelope {
            channel: KernelChannel::Stdin,
            header: self.next_response_header(parent_header, "input_request"),
            parent_header: Some(parent_header.clone()),
            metadata,
            request: input,
        };

        self.pending_input = Some(envelope.clone());

        Ok(envelope)
    }

    pub fn handle_input_reply(
        &mut self,
        envelope: KernelInputReplyEnvelope,
    ) -> Result<Option<KernelDispatchResult>, KernelStdinError> {
        let pending = self
            .pending_input
            .as_ref()
            .ok_or(KernelStdinError::NoPendingInputRequest)?;
        let found = envelope
            .parent_header
            .as_ref()
            .map(|header| header.msg_id.as_str())
            .unwrap_or_default()
            .to_string();

        if found != pending.header.msg_id {
            return Err(KernelStdinError::ReplyParentMismatch {
                expected: pending.header.msg_id.clone(),
                found,
            });
        }

        self.last_input_reply = Some(envelope.reply);
        self.pending_input = None;

        let Some(pending_execute) = self.pending_execute.take() else {
            return Ok(None);
        };

        Ok(Some(self.resume_pending_execute(pending_execute)))
    }

    pub fn dispatch_request(&mut self, envelope: KernelRequestEnvelope) -> KernelDispatchResult {
        let parent_header = envelope.header.clone();
        match envelope.request {
            KernelRequest::Execute(request) => {
                self.dispatch_execute_request(parent_header, request)
            }
            KernelRequest::Inspect(request) => self.finish_non_execute_request(
                envelope.channel,
                parent_header,
                KernelResponse::Inspect(self.inspect(request)),
            ),
            KernelRequest::Complete(request) => self.finish_non_execute_request(
                envelope.channel,
                parent_header,
                KernelResponse::Complete(self.complete(request)),
            ),
            KernelRequest::IsComplete(request) => self.finish_non_execute_request(
                envelope.channel,
                parent_header,
                KernelResponse::IsComplete(self.is_complete(request)),
            ),
            KernelRequest::KernelInfo(request) => self.finish_non_execute_request(
                envelope.channel,
                parent_header,
                KernelResponse::KernelInfo(self.kernel_info(request)),
            ),
            KernelRequest::Interrupt(request) => {
                let response = KernelResponse::Interrupt(self.interrupt(request));
                self.finish_non_execute_request(envelope.channel, parent_header, response)
            }
            KernelRequest::Shutdown(request) => {
                let response = KernelResponse::Shutdown(self.shutdown(request));
                self.finish_non_execute_request(envelope.channel, parent_header, response)
            }
        }
    }

    pub fn handle_shell_request(
        &mut self,
        envelope: KernelRequestEnvelope,
    ) -> Option<KernelResponseEnvelope> {
        self.dispatch_request(envelope).reply
    }

    fn dispatch_execute_request(
        &mut self,
        parent_header: KernelMessageHeader,
        request: KernelExecuteRequest,
    ) -> KernelDispatchResult {
        let execution_count = self.next_execution_count(request.store_history);
        self.execution_state = KernelExecutionState::Busy;

        let outcome = self.run_execute_request(&request);
        if let Some(input) = input_request_from_outcome(&outcome) {
            if request.allow_stdin {
                let mut iopub =
                    self.execute_prologue_iopub(&parent_header, execution_count, &request.code);
                iopub.extend(
                    build_suspended_outputs(&outcome)
                        .iter()
                        .map(|output| self.output_iopub(&parent_header, output)),
                );
                let stdin = self
                    .request_stdin_for_execute(
                        &parent_header,
                        &request,
                        kernel_input_request(&input),
                    )
                    .expect("execute request should be able to stage stdin exactly once");
                self.pending_execute = Some(PendingExecute {
                    parent_header: parent_header.clone(),
                    request: request.clone(),
                    execution_count,
                });

                return KernelDispatchResult {
                    reply: None,
                    iopub,
                    stdin: vec![stdin],
                };
            }
        }

        self.finish_execute_request(
            &parent_header,
            &request.code,
            execution_count,
            outcome,
            true,
        )
    }

    fn resume_pending_execute(&mut self, pending: PendingExecute) -> KernelDispatchResult {
        let input = self
            .last_input_reply
            .as_ref()
            .map(|reply| reply.value.clone())
            .expect("pending execute resumes only after an input reply");
        let outcome = match self.session.resume_eval_request_with_input(input) {
            Ok(response) => KernelExecuteOutcome::Success(response),
            Err(failure) => KernelExecuteOutcome::Failure(failure),
        };
        if let Some(input) = input_request_from_outcome(&outcome) {
            if pending.request.allow_stdin {
                let stdin = self
                    .request_stdin_for_execute(
                        &pending.parent_header,
                        &pending.request,
                        kernel_input_request(&input),
                    )
                    .expect("resumed execute request should be able to stage stdin exactly once");
                self.pending_execute = Some(pending);

                return KernelDispatchResult {
                    reply: None,
                    iopub: Vec::new(),
                    stdin: vec![stdin],
                };
            }
        }

        self.finish_execute_request(
            &pending.parent_header,
            &pending.request.code,
            pending.execution_count,
            outcome,
            false,
        )
    }

    fn execute_prologue_iopub(
        &mut self,
        parent_header: &KernelMessageHeader,
        execution_count: Option<u64>,
        code: &str,
    ) -> Vec<KernelIoPubEnvelope> {
        let mut iopub = vec![self.status_iopub(parent_header, KernelExecutionState::Busy)];

        if execution_count.is_some() {
            iopub.push(self.execute_input_iopub(parent_header, execution_count, code));
        }

        iopub
    }

    fn finish_execute_request(
        &mut self,
        parent_header: &KernelMessageHeader,
        code: &str,
        execution_count: Option<u64>,
        outcome: KernelExecuteOutcome,
        emit_prologue: bool,
    ) -> KernelDispatchResult {
        let response = build_execute_response(execution_count, outcome);
        let mut iopub = if emit_prologue {
            self.execute_prologue_iopub(parent_header, execution_count, code)
        } else {
            Vec::new()
        };

        iopub.extend(
            response
                .outputs
                .iter()
                .map(|output| self.output_iopub(parent_header, output)),
        );

        self.execution_state = KernelExecutionState::Idle;
        iopub.push(self.status_iopub(parent_header, KernelExecutionState::Idle));

        let reply = self.response_envelope(
            KernelChannel::Shell,
            parent_header,
            KernelResponse::Execute(response),
        );

        KernelDispatchResult {
            reply: Some(reply),
            iopub,
            stdin: Vec::new(),
        }
    }

    fn finish_non_execute_request(
        &mut self,
        channel: KernelChannel,
        parent_header: KernelMessageHeader,
        response: KernelResponse,
    ) -> KernelDispatchResult {
        KernelDispatchResult {
            reply: Some(self.response_envelope(channel, &parent_header, response)),
            iopub: Vec::new(),
            stdin: Vec::new(),
        }
    }

    fn response_envelope(
        &mut self,
        channel: KernelChannel,
        parent_header: &KernelMessageHeader,
        response: KernelResponse,
    ) -> KernelResponseEnvelope {
        let response_metadata = response_metadata(
            &response,
            channel,
            parent_header,
            self.execution_state,
            self.lifecycle_state,
        );
        let response_header =
            self.next_response_header(parent_header, response_msg_type(&response));

        let reply = KernelResponseEnvelope {
            channel,
            header: response_header,
            parent_header: Some(parent_header.clone()),
            metadata: response_metadata,
            response,
        };

        if matches!(reply.response, KernelResponse::Shutdown(_)) {
            self.lifecycle_state = KernelLifecycleState::Shutdown;
        }

        reply
    }

    fn next_response_header(
        &mut self,
        parent_header: &KernelMessageHeader,
        msg_type: &str,
    ) -> KernelMessageHeader {
        self.next_message_id += 1;
        let mut header = KernelMessageHeader::new(
            format!("rum-msg-{}", self.next_message_id),
            &parent_header.session_id,
            &parent_header.username,
            msg_type,
        );
        header.protocol_version = parent_header.protocol_version.clone();
        header
    }

    fn status_iopub(
        &mut self,
        parent_header: &KernelMessageHeader,
        execution_state: KernelExecutionState,
    ) -> KernelIoPubEnvelope {
        let mut metadata = KernelMetadata::new();
        metadata.insert("msg_type", "status");
        metadata.insert("channel", KernelChannel::IoPub.as_str());
        metadata.insert("execution_state", execution_state.as_str());
        metadata.insert("lifecycle_state", self.lifecycle_state.as_str());
        metadata.insert("parent_msg_type", &parent_header.msg_type);

        KernelIoPubEnvelope {
            channel: KernelChannel::IoPub,
            header: self.next_response_header(parent_header, "status"),
            parent_header: Some(parent_header.clone()),
            metadata,
            payload: KernelIoPubPayload::Status(KernelStatusOutput {
                execution_state,
                lifecycle_state: self.lifecycle_state,
            }),
        }
    }

    fn execute_input_iopub(
        &mut self,
        parent_header: &KernelMessageHeader,
        execution_count: Option<u64>,
        code: &str,
    ) -> KernelIoPubEnvelope {
        let mut metadata = KernelMetadata::new();
        metadata.insert("msg_type", "execute_input");
        metadata.insert("channel", KernelChannel::IoPub.as_str());
        metadata.insert("lifecycle_state", self.lifecycle_state.as_str());
        metadata.insert("parent_msg_type", &parent_header.msg_type);
        metadata.insert(
            "execution_count",
            execution_count
                .map(|count| count.to_string())
                .unwrap_or_else(|| "none".to_string()),
        );

        KernelIoPubEnvelope {
            channel: KernelChannel::IoPub,
            header: self.next_response_header(parent_header, "execute_input"),
            parent_header: Some(parent_header.clone()),
            metadata,
            payload: KernelIoPubPayload::ExecuteInput(KernelExecuteInput {
                execution_count,
                code: code.to_string(),
            }),
        }
    }

    fn output_iopub(
        &mut self,
        parent_header: &KernelMessageHeader,
        output: &KernelOutput,
    ) -> KernelIoPubEnvelope {
        let (msg_type, mut metadata, payload) = match output {
            KernelOutput::Stream(stream) => {
                let mut metadata = KernelMetadata::new();
                metadata.insert("msg_type", "stream");
                metadata.insert(
                    "name",
                    match stream.name {
                        KernelStreamName::Stdout => "stdout",
                        KernelStreamName::Stderr => "stderr",
                    },
                );
                (
                    "stream",
                    metadata,
                    KernelIoPubPayload::Stream(stream.clone()),
                )
            }
            KernelOutput::ExecuteResult(result) => (
                "execute_result",
                result.metadata.clone(),
                KernelIoPubPayload::ExecuteResult(result.clone()),
            ),
            KernelOutput::DisplayData(display) => (
                "display_data",
                display.metadata.clone(),
                KernelIoPubPayload::DisplayData(display.clone()),
            ),
            KernelOutput::Error(error) => {
                let mut metadata = KernelMetadata::new();
                metadata.insert("msg_type", "error");
                metadata.insert("error_name", &error.error_name);
                ("error", metadata, KernelIoPubPayload::Error(error.clone()))
            }
        };
        metadata.insert("channel", KernelChannel::IoPub.as_str());
        metadata.insert("lifecycle_state", self.lifecycle_state.as_str());
        metadata.insert("parent_msg_type", &parent_header.msg_type);

        KernelIoPubEnvelope {
            channel: KernelChannel::IoPub,
            header: self.next_response_header(parent_header, msg_type),
            parent_header: Some(parent_header.clone()),
            metadata,
            payload,
        }
    }
}

fn completion_prefix(source: &str, cursor_offset: usize) -> (usize, usize, String) {
    let cursor_offset = clamp_char_boundary(source, cursor_offset);
    let replacement_start = scan_symbol_left(source, cursor_offset);
    let replacement_end = scan_symbol_right(source, cursor_offset);
    let prefix = source[replacement_start..cursor_offset].to_string();

    (replacement_start, replacement_end, prefix)
}

fn symbol_at_cursor(source: &str, cursor_offset: usize) -> Option<String> {
    let cursor_offset = clamp_char_boundary(source, cursor_offset);
    let start = scan_symbol_left(source, cursor_offset);
    let end = scan_symbol_right(source, cursor_offset);

    (start < end).then(|| source[start..end].to_string())
}

fn scan_symbol_left(source: &str, cursor_offset: usize) -> usize {
    let mut start = cursor_offset;

    while let Some((previous_start, ch)) = previous_char(source, start) {
        if is_symbol_char(ch) {
            start = previous_start;
        } else {
            break;
        }
    }

    start
}

fn scan_symbol_right(source: &str, cursor_offset: usize) -> usize {
    let mut end = cursor_offset;

    while end < source.len() {
        let ch = source[end..]
            .chars()
            .next()
            .expect("end is always on a char boundary");

        if is_symbol_char(ch) {
            end += ch.len_utf8();
        } else {
            break;
        }
    }

    end
}

fn previous_char(source: &str, end: usize) -> Option<(usize, char)> {
    if end == 0 {
        return None;
    }

    let mut start = end - 1;
    while !source.is_char_boundary(start) {
        start -= 1;
    }

    source[start..end].chars().next().map(|ch| (start, ch))
}

fn clamp_char_boundary(source: &str, offset: usize) -> usize {
    let mut offset = offset.min(source.len());

    while offset > 0 && !source.is_char_boundary(offset) {
        offset -= 1;
    }

    offset
}

fn is_symbol_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '_' | '$' | '`')
}

fn input_request_from_outcome(outcome: &KernelExecuteOutcome) -> Option<RuntimeInputRequest> {
    match outcome {
        KernelExecuteOutcome::Failure(SessionRequestFailure {
            error: SessionRequestError::InputRequired { request },
            ..
        }) => Some(request.clone()),
        _ => None,
    }
}

fn kernel_input_request(request: &RuntimeInputRequest) -> KernelInputRequest {
    let mut input = KernelInputRequest::new(&request.prompt);
    if request.password {
        input = input.password();
    }
    input
}

fn build_execute_response(
    execution_count: Option<u64>,
    outcome: KernelExecuteOutcome,
) -> KernelExecuteResponse {
    let outputs = build_execute_outputs(execution_count, &outcome);
    let error = match &outcome {
        KernelExecuteOutcome::Success(_) => None,
        KernelExecuteOutcome::Failure(failure) => Some(error_output(failure)),
        KernelExecuteOutcome::WireDecoded => None,
    };

    KernelExecuteResponse {
        status: if matches!(outcome, KernelExecuteOutcome::Success(_)) {
            KernelReplyStatus::Ok
        } else {
            KernelReplyStatus::Error
        },
        execution_count,
        payload: Vec::<KernelExecutePayload>::new(),
        user_expressions: Default::default(),
        error,
        outputs,
        outcome,
    }
}

fn build_execute_outputs(
    execution_count: Option<u64>,
    outcome: &KernelExecuteOutcome,
) -> Vec<KernelOutput> {
    match outcome {
        KernelExecuteOutcome::Success(response) => {
            let mut outputs = runtime_message_streams(&response.messages);
            if !response.trace.is_empty() {
                outputs.push(KernelOutput::DisplayData(KernelDisplayOutput {
                    data: trace_mime_bundle(&response.trace),
                    metadata: trace_metadata(),
                }));
            }
            outputs.push(KernelOutput::ExecuteResult(KernelExecuteResult {
                execution_count,
                data: expr_mime_bundle(&response.value.to_string()),
                metadata: execute_result_metadata(execution_count),
            }));
            outputs
        }
        KernelExecuteOutcome::Failure(failure) => {
            let mut outputs = runtime_message_streams(&failure.messages);
            if !failure.trace.is_empty() {
                outputs.push(KernelOutput::DisplayData(KernelDisplayOutput {
                    data: trace_mime_bundle(&failure.trace),
                    metadata: trace_metadata(),
                }));
            }
            outputs.push(KernelOutput::Error(error_output(failure)));
            outputs
        }
        KernelExecuteOutcome::WireDecoded => Vec::new(),
    }
}

fn build_suspended_outputs(outcome: &KernelExecuteOutcome) -> Vec<KernelOutput> {
    match outcome {
        KernelExecuteOutcome::Failure(failure)
            if matches!(failure.error, SessionRequestError::InputRequired { .. }) =>
        {
            let mut outputs = runtime_message_streams(&failure.messages);
            if !failure.trace.is_empty() {
                outputs.push(KernelOutput::DisplayData(KernelDisplayOutput {
                    data: trace_mime_bundle(&failure.trace),
                    metadata: trace_metadata(),
                }));
            }
            outputs
        }
        _ => Vec::new(),
    }
}

fn runtime_message_streams(messages: &[RuntimeMessage]) -> Vec<KernelOutput> {
    messages
        .iter()
        .map(|message| {
            KernelOutput::Stream(KernelStreamOutput {
                name: KernelStreamName::Stderr,
                text: format!("{message}\n"),
            })
        })
        .collect()
}

fn error_output(failure: &SessionRequestFailure) -> KernelErrorOutput {
    let (error_name, rendered) = match &failure.error {
        SessionRequestError::Syntax(diagnostic) => {
            ("Syntax".to_string(), Some(diagnostic.rendered.clone()))
        }
        SessionRequestError::InvalidInteger { .. } => ("InvalidInteger".to_string(), None),
        SessionRequestError::Io { .. } => ("Io".to_string(), None),
        SessionRequestError::LoadPathNotFound { .. } => ("LoadPathNotFound".to_string(), None),
        SessionRequestError::EvaluationCycleDetected { .. } => {
            ("EvaluationCycleDetected".to_string(), None)
        }
        SessionRequestError::RecursionLimitExceeded { .. } => {
            ("RecursionLimitExceeded".to_string(), None)
        }
        SessionRequestError::IterationLimitExceeded { .. } => {
            ("IterationLimitExceeded".to_string(), None)
        }
        SessionRequestError::EvaluationPassLimitExceeded { .. } => {
            ("EvaluationPassLimitExceeded".to_string(), None)
        }
        SessionRequestError::InputRequired { .. } => ("InputRequired".to_string(), None),
        SessionRequestError::Other { .. } => ("Other".to_string(), None),
    };

    KernelErrorOutput {
        error_name,
        error_value: failure.error.to_string(),
        rendered,
    }
}

fn expr_mime_bundle(rendered_expr: &str) -> KernelMimeBundle {
    let mut bundle = KernelMimeBundle::text_plain(rendered_expr);
    bundle.insert("text/markdown", format!("```rum\n{rendered_expr}\n```"));
    bundle.insert("application/vnd.rum.expr", rendered_expr);
    bundle
}

fn trace_mime_bundle(trace: &[TraceEvent]) -> KernelMimeBundle {
    let rendered = trace
        .iter()
        .map(render_trace_event)
        .collect::<Vec<_>>()
        .join("\n");

    let mut bundle = KernelMimeBundle::text_plain(&rendered);
    bundle.insert("text/markdown", format!("```text\n{rendered}\n```"));
    bundle.insert("application/vnd.rum.trace", rendered);
    bundle
}

fn inspect_mime_bundle(response: &SessionInspectResponse) -> KernelMimeBundle {
    let plain = render_inspect_text(response);
    let markdown = render_inspect_markdown(response);

    let mut bundle = KernelMimeBundle::text_plain(&plain);
    bundle.insert("text/markdown", markdown);
    bundle.insert("application/vnd.rum.inspect", plain);
    bundle
}

fn execute_result_metadata(execution_count: Option<u64>) -> KernelMetadata {
    let mut metadata = KernelMetadata::new();
    metadata.insert("output_type", "execute_result");
    metadata.insert("bundle_kind", "expr");
    metadata.insert(
        "execution_count",
        execution_count
            .map(|count| count.to_string())
            .unwrap_or_else(|| "none".to_string()),
    );
    metadata
}

fn trace_metadata() -> KernelMetadata {
    let mut metadata = KernelMetadata::new();
    metadata.insert("output_type", "display_data");
    metadata.insert("bundle_kind", "trace");
    metadata
}

fn inspect_metadata(target: Option<&str>, found: bool) -> KernelMetadata {
    let mut metadata = KernelMetadata::new();
    metadata.insert("found", if found { "true" } else { "false" });
    metadata.insert(
        "target",
        target
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| "<none>".to_string()),
    );
    metadata.insert("bundle_kind", "inspect");
    metadata
}

fn response_msg_type(response: &KernelResponse) -> &'static str {
    match response {
        KernelResponse::Execute(_) => "execute_reply",
        KernelResponse::Inspect(_) => "inspect_reply",
        KernelResponse::Complete(_) => "complete_reply",
        KernelResponse::IsComplete(_) => "is_complete_reply",
        KernelResponse::KernelInfo(_) => "kernel_info_reply",
        KernelResponse::Interrupt(_) => "interrupt_reply",
        KernelResponse::Shutdown(_) => "shutdown_reply",
    }
}

fn response_metadata(
    response: &KernelResponse,
    channel: KernelChannel,
    parent_header: &KernelMessageHeader,
    execution_state: KernelExecutionState,
    lifecycle_state: KernelLifecycleState,
) -> KernelMetadata {
    let mut metadata = KernelMetadata::new();
    metadata.insert("msg_type", response_msg_type(response));
    metadata.insert("channel", channel.as_str());
    metadata.insert("execution_state", execution_state.as_str());
    metadata.insert("lifecycle_state", lifecycle_state.as_str());
    metadata.insert("parent_msg_type", &parent_header.msg_type);
    metadata.insert("protocol_version", &parent_header.protocol_version);

    let status = match response {
        KernelResponse::Execute(execute) => execute.status.as_str(),
        KernelResponse::Inspect(inspect) => inspect.status.as_str(),
        KernelResponse::Complete(complete) => complete.status.as_str(),
        KernelResponse::IsComplete(response) => response.reply_status.as_str(),
        KernelResponse::KernelInfo(info) => info.status.as_str(),
        KernelResponse::Interrupt(interrupt) => interrupt.status.as_str(),
        KernelResponse::Shutdown(shutdown) => shutdown.status.as_str(),
    };

    metadata.insert("status", status);

    if let KernelResponse::IsComplete(response) = response {
        metadata.insert("completion_status", response.completion_status.as_str());
    }

    metadata
}

fn complete_metadata(
    prefix: &str,
    replacement_start: usize,
    replacement_end: usize,
) -> KernelMetadata {
    let mut metadata = KernelMetadata::new();
    metadata.insert("cursor_start", replacement_start.to_string());
    metadata.insert("cursor_end", replacement_end.to_string());
    metadata.insert("prefix", prefix);
    metadata
}

fn input_request_metadata(
    parent_header: &KernelMessageHeader,
    request: &KernelExecuteRequest,
    input: &KernelInputRequest,
) -> KernelMetadata {
    let mut metadata = KernelMetadata::new();
    metadata.insert("msg_type", "input_request");
    metadata.insert("channel", KernelChannel::Stdin.as_str());
    metadata.insert("parent_msg_type", &parent_header.msg_type);
    metadata.insert(
        "allow_stdin",
        if request.allow_stdin { "true" } else { "false" },
    );
    metadata.insert("password", if input.password { "true" } else { "false" });
    metadata
}

fn render_inspect_text(response: &SessionInspectResponse) -> String {
    let info = &response.info;
    let mut lines = vec![
        format!("{} ({})", info.qualified_name, info.context),
        format!(
            "attributes: {}",
            if info.attributes.is_empty() {
                "<none>".to_string()
            } else {
                info.attributes.join(", ")
            }
        ),
        format!(
            "definitions: own={} delayed={} up={} down={} sub={}",
            info.has_own_value,
            info.has_delayed_value,
            info.has_up_values,
            info.has_down_values,
            info.has_sub_values
        ),
        format!(
            "messages: {}",
            if info.message_tags.is_empty() {
                "<none>".to_string()
            } else {
                info.message_tags.join(", ")
            }
        ),
    ];

    if let Some(summary) = &info.summary {
        lines.insert(1, format!("summary: {summary}"));
    }

    lines.join("\n")
}

fn render_inspect_markdown(response: &SessionInspectResponse) -> String {
    let info = &response.info;
    let attributes = if info.attributes.is_empty() {
        "<none>".to_string()
    } else {
        info.attributes.join(", ")
    };
    let messages = if info.message_tags.is_empty() {
        "<none>".to_string()
    } else {
        info.message_tags.join(", ")
    };

    format!(
        "### `{}`\n\n- Context: `{}`\n- Attributes: {}\n- Definitions: own=`{}`, delayed=`{}`, up=`{}`, down=`{}`, sub=`{}`\n- Messages: {}",
        info.qualified_name,
        info.context,
        attributes,
        info.has_own_value,
        info.has_delayed_value,
        info.has_up_values,
        info.has_down_values,
        info.has_sub_values,
        messages
    )
}

fn render_trace_event(event: &TraceEvent) -> String {
    match event {
        TraceEvent::EvaluationPass { before, after } => {
            format!("evaluation-pass: {before} => {after}")
        }
        TraceEvent::DefinitionStored {
            symbol,
            kind,
            value,
        } => format!("definition-stored: {symbol} [{kind:?}] = {value}"),
        TraceEvent::RuleApplied {
            source,
            input,
            rule,
            result,
        } => format!(
            "rule-applied: {} | input={input} | rule={rule} | result={result}",
            render_rule_application_source(*source)
        ),
    }
}

fn render_rule_application_source(source: RuleApplicationSource) -> &'static str {
    match source {
        RuleApplicationSource::UpValue => "upvalue",
        RuleApplicationSource::DownValue => "downvalue",
        RuleApplicationSource::SubValue => "subvalue",
        RuleApplicationSource::ReplaceAll => "replace-all",
        RuleApplicationSource::ReplaceRepeated => "replace-repeated",
    }
}
