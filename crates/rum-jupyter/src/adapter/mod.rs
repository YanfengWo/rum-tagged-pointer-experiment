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

mod cursor;
mod iopub;
mod render;

use self::{
    cursor::{completion_prefix, symbol_at_cursor},
    render::{
        build_execute_response, build_suspended_outputs, complete_metadata,
        input_request_from_outcome, input_request_metadata, inspect_metadata, inspect_mime_bundle,
        kernel_input_request, response_metadata, response_msg_type,
    },
};

pub struct KernelAdapter {
    session: Session,
    kernel_username: String,
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
        Self::with_username(session, "kernel")
    }

    pub fn with_username(session: Session, username: impl Into<String>) -> Self {
        Self {
            session,
            kernel_username: username.into(),
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
        self.execution_state = KernelExecutionState::Busy;
        let mut iopub = vec![self.status_iopub(&parent_header, KernelExecutionState::Busy)];
        self.execution_state = KernelExecutionState::Idle;

        KernelDispatchResult {
            reply: Some(self.response_envelope(channel, &parent_header, response)),
            iopub: {
                iopub.push(self.status_iopub(&parent_header, KernelExecutionState::Idle));
                iopub
            },
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
            &self.kernel_username,
            msg_type,
        );
        header.protocol_version = parent_header.protocol_version.clone();
        header
    }
}
