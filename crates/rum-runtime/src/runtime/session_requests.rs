use super::*;

#[derive(Clone, Copy, Debug)]
struct RequestCheckpoint {
    message_checkpoint: usize,
    handled_error_checkpoint: usize,
    trace_checkpoint: usize,
}

impl RequestCheckpoint {
    fn new(session: &Session) -> Self {
        Self {
            message_checkpoint: session.messages.len(),
            handled_error_checkpoint: session.handled_non_loader_eval_errors.len(),
            trace_checkpoint: session.trace_events.len(),
        }
    }
}

impl From<RequestCheckpoint> for PendingEvalRequest {
    fn from(checkpoint: RequestCheckpoint) -> Self {
        Self {
            message_checkpoint: checkpoint.message_checkpoint,
            handled_error_checkpoint: checkpoint.handled_error_checkpoint,
            trace_checkpoint: checkpoint.trace_checkpoint,
        }
    }
}

impl From<PendingEvalRequest> for RequestCheckpoint {
    fn from(state: PendingEvalRequest) -> Self {
        Self {
            message_checkpoint: state.message_checkpoint,
            handled_error_checkpoint: state.handled_error_checkpoint,
            trace_checkpoint: state.trace_checkpoint,
        }
    }
}

impl Session {
    pub fn complete_request(&self, prefix: &str) -> SessionCompletionResponse {
        let prefix = prefix.trim().to_string();
        let matches = self.complete_symbol_prefix(&prefix);

        SessionCompletionResponse { prefix, matches }
    }

    pub fn is_complete_request(&self, input: &str) -> SessionIsCompleteResponse {
        if input.trim().is_empty() {
            return SessionIsCompleteResponse {
                status: SessionCompletenessStatus::Complete,
                indent: None,
                diagnostic: None,
            };
        }

        match parse_with_diagnostics(input) {
            Ok(_) => SessionIsCompleteResponse {
                status: SessionCompletenessStatus::Complete,
                indent: None,
                diagnostic: None,
            },
            Err(diagnostic) => {
                let diagnostic = diagnostic.structured();

                if diagnostic.is_incomplete {
                    SessionIsCompleteResponse {
                        status: SessionCompletenessStatus::Incomplete,
                        indent: Some(suggest_continuation_indent(input)),
                        diagnostic: Some(diagnostic),
                    }
                } else {
                    SessionIsCompleteResponse {
                        status: SessionCompletenessStatus::Invalid,
                        indent: None,
                        diagnostic: Some(diagnostic),
                    }
                }
            }
        }
    }

    pub fn parse_request(
        &self,
        input: &str,
    ) -> Result<SessionParseResponse, SessionRequestFailure> {
        let parsed = parse_with_diagnostics(input).map_err(request_syntax_failure)?;
        Ok(SessionParseResponse { parsed })
    }

    pub fn lower_request(
        &mut self,
        input: &str,
    ) -> Result<SessionLowerResponse, SessionRequestFailure> {
        let checkpoint = RequestCheckpoint::new(self);
        let parsed = parse_with_diagnostics(input).map_err(request_syntax_failure)?;
        let expr = self
            .lower_syntax_expr(parsed.syntax())
            .map_err(|err| self.finish_request_failure(checkpoint, err))?;

        Ok(SessionLowerResponse { expr })
    }

    pub fn eval_request(
        &mut self,
        input: &str,
    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
        let checkpoint = RequestCheckpoint::new(self);
        let parsed = parse_with_diagnostics(input).map_err(request_syntax_failure)?;
        let expr = self
            .lower_syntax_expr(parsed.syntax())
            .map_err(|err| self.finish_request_failure(checkpoint, err))?;
        let outcome = self.evaluate(expr);
        let value = match self.finalize_top_level_control_flow(outcome) {
            Ok(value) => value,
            Err(EvalError::InputRequired { .. }) => {
                self.pending_eval_request = Some(checkpoint.into());
                return Err(self.finish_request_failure(
                    checkpoint,
                    EvalError::InputRequired {
                        request: self
                            .pending_input_request()
                            .cloned()
                            .expect("input suspension should record the pending request"),
                    },
                ));
            }
            Err(err) => return Err(self.finish_request_failure(checkpoint, err)),
        };

        let messages = self.messages.split_off(checkpoint.message_checkpoint);
        let trace = self.trace_events[checkpoint.trace_checkpoint..].to_vec();
        self.last_message_list = messages.clone();
        self.handled_non_loader_eval_errors
            .truncate(checkpoint.handled_error_checkpoint);
        self.pending_eval_request = None;

        Ok(SessionEvalResponse {
            value,
            messages,
            trace,
        })
    }

    pub fn resume_eval_request_with_input(
        &mut self,
        input: impl Into<String>,
    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
        let checkpoint = self
            .pending_eval_request
            .take()
            .map(RequestCheckpoint::from)
            .ok_or_else(|| SessionRequestFailure {
                error: SessionRequestError::Other {
                    message: "no suspended evaluation is waiting for host input".to_string(),
                },
                messages: Vec::new(),
                trace: Vec::new(),
            })?;

        let outcome = self.resume_suspended_evaluation(input);
        let value = match self.finalize_top_level_control_flow(outcome) {
            Ok(value) => value,
            Err(EvalError::InputRequired { .. }) => {
                self.pending_eval_request = Some(checkpoint.into());
                return Err(self.finish_request_failure(
                    checkpoint,
                    EvalError::InputRequired {
                        request: self
                            .pending_input_request()
                            .cloned()
                            .expect("nested input suspension should record the pending request"),
                    },
                ));
            }
            Err(err) => return Err(self.finish_request_failure(checkpoint, err)),
        };

        let messages = self.messages.split_off(checkpoint.message_checkpoint);
        let trace = self.trace_events[checkpoint.trace_checkpoint..].to_vec();
        self.last_message_list = messages.clone();
        self.handled_non_loader_eval_errors
            .truncate(checkpoint.handled_error_checkpoint);

        Ok(SessionEvalResponse {
            value,
            messages,
            trace,
        })
    }

    pub fn eval_script_request(
        &mut self,
        source: &str,
    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
        let checkpoint = RequestCheckpoint::new(self);

        match self.eval_script(source) {
            Ok(outcome) => Ok(SessionEvalResponse {
                value: outcome.value,
                messages: outcome.messages,
                trace: self.trace_events[checkpoint.trace_checkpoint..].to_vec(),
            }),
            Err(err) => Err(self.finish_completed_request_failure(checkpoint, Some(source), err)),
        }
    }

    pub fn eval_file_request(
        &mut self,
        path: impl AsRef<Path>,
    ) -> Result<SessionEvalResponse, SessionRequestFailure> {
        let checkpoint = RequestCheckpoint::new(self);
        let path = path.as_ref();

        match self.eval_file(path) {
            Ok(outcome) => Ok(SessionEvalResponse {
                value: outcome.value,
                messages: outcome.messages,
                trace: self.trace_events[checkpoint.trace_checkpoint..].to_vec(),
            }),
            Err(err) => {
                let source = match err {
                    EvalError::Syntax(_) => fs::read_to_string(path).ok(),
                    _ => None,
                };

                Err(self.finish_completed_request_failure(checkpoint, source.as_deref(), err))
            }
        }
    }

    pub fn inspect_request(&self, name: &str) -> Option<SessionInspectResponse> {
        let info = self.inspect(name)?;
        let definitions = self.inspect_definition_catalog(name).unwrap_or_default();
        let message_catalog = self.inspect_message_catalog(Some(name)).unwrap_or_default();

        Some(SessionInspectResponse {
            info,
            definitions,
            message_catalog,
        })
    }

    fn finish_request_failure(
        &mut self,
        checkpoint: RequestCheckpoint,
        err: EvalError,
    ) -> SessionRequestFailure {
        self.emit_non_loader_eval_message(checkpoint.handled_error_checkpoint, &err);

        let messages = self.messages[checkpoint.message_checkpoint..].to_vec();
        let trace = self.trace_events[checkpoint.trace_checkpoint..].to_vec();
        self.last_message_list = messages.clone();
        self.handled_non_loader_eval_errors
            .truncate(checkpoint.handled_error_checkpoint);

        SessionRequestFailure {
            error: session_request_error_from_eval_error(&err),
            messages,
            trace,
        }
    }

    fn finish_completed_request_failure(
        &mut self,
        checkpoint: RequestCheckpoint,
        source: Option<&str>,
        err: EvalError,
    ) -> SessionRequestFailure {
        let messages = self.messages[checkpoint.message_checkpoint..].to_vec();
        let trace = self.trace_events[checkpoint.trace_checkpoint..].to_vec();
        self.last_message_list = messages.clone();
        self.handled_non_loader_eval_errors
            .truncate(checkpoint.handled_error_checkpoint);

        SessionRequestFailure {
            error: session_request_error_from_eval_error_with_source(&err, source),
            messages,
            trace,
        }
    }
}

fn request_syntax_failure(diagnostic: rum_syntax::SourceDiagnostic) -> SessionRequestFailure {
    SessionRequestFailure {
        error: SessionRequestError::Syntax(diagnostic.structured()),
        messages: Vec::new(),
        trace: Vec::new(),
    }
}

fn suggest_continuation_indent(input: &str) -> String {
    let base_indent = input
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty())
        .map(|line| {
            line.chars()
                .take_while(|ch| ch.is_whitespace())
                .collect::<String>()
        })
        .unwrap_or_default();

    format!("{base_indent}  ")
}

fn session_request_error_from_eval_error(err: &EvalError) -> SessionRequestError {
    session_request_error_from_eval_error_with_source(err, None)
}

fn session_request_error_from_eval_error_with_source(
    err: &EvalError,
    source: Option<&str>,
) -> SessionRequestError {
    match err {
        EvalError::Syntax(error) => source
            .map(|source| {
                SessionRequestError::Syntax(
                    rum_syntax::SourceDiagnostic::new(
                        rum_syntax::SourceText::new(source),
                        error.clone(),
                    )
                    .structured(),
                )
            })
            .unwrap_or_else(|| SessionRequestError::Other {
                message: error.to_string(),
            }),
        EvalError::InvalidInteger { literal } => SessionRequestError::InvalidInteger {
            literal: literal.clone(),
        },
        EvalError::Io { path, source } => SessionRequestError::Io {
            path: path.clone(),
            message: source.to_string(),
        },
        EvalError::LoadPathNotFound { target, searched } => SessionRequestError::LoadPathNotFound {
            target: target.clone(),
            searched: searched.clone(),
        },
        EvalError::EvaluationCycleDetected { expr } => {
            SessionRequestError::EvaluationCycleDetected { expr: expr.clone() }
        }
        EvalError::RecursionLimitExceeded { limit } => {
            SessionRequestError::RecursionLimitExceeded { limit: *limit }
        }
        EvalError::IterationLimitExceeded { limit } => {
            SessionRequestError::IterationLimitExceeded { limit: *limit }
        }
        EvalError::EvaluationPassLimitExceeded { limit } => {
            SessionRequestError::EvaluationPassLimitExceeded { limit: *limit }
        }
        EvalError::Return { value } => SessionRequestError::Other {
            message: format!("uncaught top-level Return resolved to {value}"),
        },
        EvalError::Thrown { value, tag } => SessionRequestError::Other {
            message: format!("uncaught throw: {value} with tag {tag}"),
        },
        EvalError::Abort => SessionRequestError::Other {
            message: "uncaught Abort[]".to_string(),
        },
        EvalError::Break => SessionRequestError::Other {
            message: "uncaught Break[]".to_string(),
        },
        EvalError::Continue => SessionRequestError::Other {
            message: "uncaught Continue[]".to_string(),
        },
        EvalError::InputRequired { request } => SessionRequestError::InputRequired {
            request: request.clone(),
        },
    }
}
