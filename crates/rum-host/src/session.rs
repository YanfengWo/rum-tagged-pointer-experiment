use rum_runtime::Session;
use std::any::Any;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use crate::config::HostConfig;
use crate::cursor::{completion_prefix, symbol_at_cursor};
use crate::model::{
    HostCompletionRequest, HostCompletionResponse, HostDefinitionSummary, HostEvalOutcome,
    HostEvalRequest, HostFailure, HostFailureCode, HostInspectRequest, HostInspectResponse,
    HostIsCompleteRequest, HostIsCompleteResponse, HostParseRequest, HostParseResponse,
    completeness_status, eval_failure_from_runtime, eval_success_from_runtime,
    host_failure_from_runtime, inspect_response_from_runtime, synthesized_host_failure,
};

#[derive(Clone, Debug)]
pub struct HostSessionBuilder {
    config: HostConfig,
}

impl Default for HostSessionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl HostSessionBuilder {
    pub fn new() -> Self {
        Self {
            config: HostConfig::default(),
        }
    }

    pub fn with_config(mut self, config: HostConfig) -> Self {
        self.config = config;
        self
    }

    pub fn with_tracing(mut self, enabled: bool) -> Self {
        self.config.tracing_enabled = enabled;
        self
    }

    pub fn with_core_builtins(mut self, enabled: bool) -> Self {
        self.config.install_core_builtins = enabled;
        self
    }

    pub fn build(self) -> HostSession {
        let session = HostSession::build_inner_session(&self.config);
        let install_core_builtins = self.config.install_core_builtins;

        HostSession {
            session,
            config: self.config,
            install_core_builtins,
            poisoned: false,
        }
    }
}

pub struct HostSession {
    session: Session,
    config: HostConfig,
    install_core_builtins: bool,
    poisoned: bool,
}

#[derive(Clone, Debug)]
pub struct HostCancelHandle {
    flag: Arc<AtomicBool>,
}

impl HostCancelHandle {
    pub fn cancel(&self) {
        self.flag.store(true, Ordering::Relaxed);
    }
}

impl Default for HostSession {
    fn default() -> Self {
        Self::new()
    }
}

impl HostSession {
    pub fn new() -> Self {
        Self::builder().build()
    }

    pub fn builder() -> HostSessionBuilder {
        HostSessionBuilder::new()
    }

    fn build_inner_session(config: &HostConfig) -> Session {
        let mut session = Session::with_config(config.runtime_config());

        if config.install_core_builtins {
            rum_stdlib::install_core(&mut session);
        }

        session
    }

    /// Rebuilds this session to its original post-construction state.
    ///
    /// This drops all user state, including definitions, attributes, contexts,
    /// loaded packages, messages, and gensyms. It also invalidates any
    /// previously returned [`HostCancelHandle`], because the rebuilt session has
    /// a fresh interrupt flag; obtain a new handle after reset. If the session
    /// was poisoned by a caught panic, reset clears the poisoned flag.
    pub fn reset(&mut self) {
        self.session = Self::build_inner_session(&self.config);
        self.poisoned = false;
    }

    pub fn configuration(&self) -> HostConfig {
        let config = self.session.configuration();

        HostConfig {
            resource_limits: config.resource_limits.into(),
            current_context: config.current_context,
            context_path: config.context_path,
            package_search_paths: config.package_search_paths,
            tracing_enabled: config.tracing_enabled,
            asserts_enabled: config.asserts_enabled,
            install_core_builtins: self.install_core_builtins,
        }
    }

    pub fn parse(
        &mut self,
        request: impl Into<HostParseRequest>,
    ) -> Result<HostParseResponse, HostFailure> {
        if self.poisoned {
            return Err(poisoned_failure());
        }

        let request = request.into();
        self.guard(
            |session| {
                session
                    .session
                    .parse_request(&request.source)
                    .map(|response| HostParseResponse {
                        source: response.parsed.source().to_string(),
                        syntax: response.parsed.syntax().to_string(),
                    })
                    .map_err(|failure| host_failure_from_runtime(failure))
            },
            |message| Err(panic_failure(message)),
        )
    }

    pub fn eval(&mut self, request: impl Into<HostEvalRequest>) -> HostEvalOutcome {
        if self.poisoned {
            return poisoned_eval_outcome();
        }

        let request = request.into();
        self.guard(
            |session| session.eval_request_unprotected(request),
            panic_eval_outcome,
        )
    }

    fn eval_request_unprotected(&mut self, request: HostEvalRequest) -> HostEvalOutcome {
        match self.session.eval_request(&request.source) {
            Ok(response) => HostEvalOutcome::Success(eval_success_from_runtime(response)),
            Err(failure) => eval_failure_from_runtime(failure),
        }
    }

    pub fn cancel_handle(&self) -> HostCancelHandle {
        HostCancelHandle {
            flag: self.session.interrupt_handle(),
        }
    }

    pub fn definitions(&self) -> Vec<HostDefinitionSummary> {
        self.session
            .definition_summaries()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    pub fn contexts(&self) -> Vec<String> {
        self.session.context_names()
    }

    pub fn eval_with_timeout(
        &mut self,
        request: impl Into<HostEvalRequest>,
        timeout: Duration,
    ) -> HostEvalOutcome {
        if self.poisoned {
            return poisoned_eval_outcome();
        }

        let request = request.into();
        self.guard(
            |session| {
                session.session.set_eval_timeout(Some(timeout));
                let outcome = session.eval_request_unprotected(request);
                session.session.set_eval_timeout(None);
                outcome
            },
            panic_eval_outcome,
        )
    }

    pub fn resume_with_input(&mut self, input: impl Into<String>) -> HostEvalOutcome {
        if self.poisoned {
            return poisoned_eval_outcome();
        }

        self.guard(
            |session| match session.session.resume_eval_request_with_input(input) {
                Ok(response) => HostEvalOutcome::Success(eval_success_from_runtime(response)),
                Err(failure) => eval_failure_from_runtime(failure),
            },
            panic_eval_outcome,
        )
    }

    pub fn inspect(&mut self, request: HostInspectRequest) -> HostInspectResponse {
        if self.poisoned {
            return unavailable_inspect_response(None);
        }

        self.guard(
            |session| {
                let target = symbol_at_cursor(&request.source, request.cursor_offset);
                let response = target
                    .as_deref()
                    .and_then(|symbol| session.session.inspect_request(symbol));

                inspect_response_from_runtime(target, response)
            },
            |_| unavailable_inspect_response(None),
        )
    }

    pub fn inspect_symbol(&mut self, symbol: &str) -> HostInspectResponse {
        if self.poisoned {
            return unavailable_inspect_response(Some(symbol.to_string()));
        }

        self.guard(
            |session| {
                inspect_response_from_runtime(
                    Some(symbol.to_string()),
                    session.session.inspect_request(symbol),
                )
            },
            |_| unavailable_inspect_response(Some(symbol.to_string())),
        )
    }

    pub fn complete(&mut self, request: HostCompletionRequest) -> HostCompletionResponse {
        if self.poisoned {
            return unavailable_completion_response();
        }

        self.guard(
            |session| {
                let (replacement_start, replacement_end, prefix) =
                    completion_prefix(&request.source, request.cursor_offset);
                let completion = session.session.complete_request(&prefix);

                HostCompletionResponse {
                    prefix: completion.prefix,
                    matches: completion.matches,
                    replacement_start,
                    replacement_end,
                }
            },
            |_| unavailable_completion_response(),
        )
    }

    pub fn is_complete(
        &mut self,
        request: impl Into<HostIsCompleteRequest>,
    ) -> HostIsCompleteResponse {
        if self.poisoned {
            return poisoned_is_complete_response();
        }

        let request = request.into();
        self.guard(
            |session| {
                let response = session.session.is_complete_request(&request.source);

                HostIsCompleteResponse {
                    status: completeness_status(response.status),
                    indent: response.indent,
                    diagnostic: response.diagnostic.map(|diagnostic| diagnostic.message),
                }
            },
            |_| panic_is_complete_response(),
        )
    }

    pub fn pending_input_request(&self) -> Option<crate::HostInputRequest> {
        self.session
            .pending_input_request()
            .cloned()
            .map(Into::into)
    }

    fn guard<T>(
        &mut self,
        body: impl FnOnce(&mut Self) -> T,
        on_panic: impl FnOnce(String) -> T,
    ) -> T {
        match catch_unwind(AssertUnwindSafe(|| body(self))) {
            Ok(outcome) => outcome,
            Err(payload) => {
                self.poisoned = true;
                on_panic(panic_payload_message(&*payload))
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn eval_with_test_panic(&mut self) -> HostEvalOutcome {
        if self.poisoned {
            return poisoned_eval_outcome();
        }

        self.guard(
            |_session| panic!("host panic boundary test"),
            panic_eval_outcome,
        )
    }

    #[cfg(test)]
    pub(crate) fn poisoned_for_test(&self) -> bool {
        self.poisoned
    }
}

fn panic_payload_message(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        return (*message).to_string();
    }
    if let Some(message) = payload.downcast_ref::<String>() {
        return message.clone();
    }
    "panic".to_string()
}

fn panic_failure(message: String) -> HostFailure {
    synthesized_host_failure(
        HostFailureCode::Panic,
        format!("host session caught panic: {message}"),
    )
}

fn poisoned_failure() -> HostFailure {
    synthesized_host_failure(
        HostFailureCode::Poisoned,
        "host session is poisoned after a previously caught panic",
    )
}

fn panic_eval_outcome(message: String) -> HostEvalOutcome {
    HostEvalOutcome::Failure(panic_failure(message))
}

fn poisoned_eval_outcome() -> HostEvalOutcome {
    HostEvalOutcome::Failure(poisoned_failure())
}

fn unavailable_inspect_response(target: Option<String>) -> HostInspectResponse {
    HostInspectResponse {
        target,
        found: false,
        info: None,
        definitions: Vec::new(),
        message_catalog: Vec::new(),
    }
}

fn unavailable_completion_response() -> HostCompletionResponse {
    HostCompletionResponse {
        prefix: String::new(),
        matches: Vec::new(),
        replacement_start: 0,
        replacement_end: 0,
    }
}

fn panic_is_complete_response() -> HostIsCompleteResponse {
    HostIsCompleteResponse {
        status: crate::HostCompletenessStatus::Invalid,
        indent: None,
        diagnostic: Some("host session caught panic".to_string()),
    }
}

fn poisoned_is_complete_response() -> HostIsCompleteResponse {
    HostIsCompleteResponse {
        status: crate::HostCompletenessStatus::Invalid,
        indent: None,
        diagnostic: Some("host session is poisoned after a previously caught panic".to_string()),
    }
}
