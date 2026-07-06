use rum_runtime::{CapabilityPolicy, Session};
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
    HostEvalRequest, HostFailure, HostFailureCode, HostInputSuspension, HostInspectRequest,
    HostInspectResponse, HostIsCompleteRequest, HostIsCompleteResponse, HostParseRequest,
    HostParseResponse, completeness_status, eval_failure_from_runtime, eval_success_from_runtime,
    host_failure_from_runtime, inspect_response_from_runtime, synthesized_host_failure,
};

/// Builder for [`HostSession`].
///
/// Use this when the default session configuration needs tracing, sandboxing,
/// or core-builtin installation changes before construction.
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
    /// Creates a builder with [`HostConfig::default`].
    pub fn new() -> Self {
        Self {
            config: HostConfig::default(),
        }
    }

    /// Replaces the full host configuration used by the builder.
    pub fn with_config(mut self, config: HostConfig) -> Self {
        self.config = config;
        self
    }

    /// Enables or disables evaluation trace collection.
    pub fn with_tracing(mut self, enabled: bool) -> Self {
        self.config.tracing_enabled = enabled;
        self
    }

    /// Replaces the capability policy for the built session.
    pub fn with_capability_policy(mut self, policy: CapabilityPolicy) -> Self {
        self.config.capability_policy = policy;
        self
    }

    /// Enables or disables installation of the standard core builtins.
    pub fn with_core_builtins(mut self, enabled: bool) -> Self {
        self.config.install_core_builtins = enabled;
        self
    }

    /// Builds a new host session from the builder configuration.
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

/// Stateful embedding session for evaluating `rum` source.
///
/// A session owns runtime definitions, contexts, pending input continuations,
/// tracing state, timeout/cancel configuration, and the poison boundary used
/// after a caught panic.
pub struct HostSession {
    session: Session,
    config: HostConfig,
    install_core_builtins: bool,
    poisoned: bool,
}

/// Cloneable cancellation handle for a [`HostSession`].
///
/// A handle interrupts the session instance that produced it. Handles returned
/// before [`HostSession::reset`] are invalidated by that reset.
#[derive(Clone, Debug)]
pub struct HostCancelHandle {
    flag: Arc<AtomicBool>,
}

impl HostCancelHandle {
    /// Requests cancellation of the associated session's current evaluation.
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
    /// Creates a session with the default trusted-host configuration.
    ///
    /// The default installs the standard core builtins and uses the default
    /// unrestricted capability policy.
    pub fn new() -> Self {
        Self::builder().build()
    }

    /// Creates a builder for configuring a host session before construction.
    pub fn builder() -> HostSessionBuilder {
        HostSessionBuilder::new()
    }

    fn build_inner_session(config: &HostConfig) -> Session {
        let mut session = Session::with_config(config.runtime_config());
        session.set_file_io(config.file_io.clone());
        session.set_entropy_source(config.entropy_source.clone());

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

    /// Serializes this session's WL-level user definitions to a text image.
    ///
    /// The image captures own values, delayed own values, down values, up values,
    /// sub values, and attributes for symbols that have definitions. It does not
    /// capture attribute-only symbols, `$` variables, options, messages, the
    /// gensym counter, PRNG state, or contexts beyond what the generated source
    /// re-establishes. This is a text image, not a binary `.mx` dump.
    pub fn snapshot_to_bytes(&self) -> Vec<u8> {
        self.session.snapshot_definitions_source().into_bytes()
    }

    /// Replaces this session's user state by restoring a text snapshot image.
    ///
    /// The snapshot format is the UTF-8 source emitted by [`Self::snapshot_to_bytes`].
    /// Restore decodes the bytes, resets the session to its original
    /// post-construction state, then evaluates the snapshot source. The image
    /// captures WL-level definitions and attributes for defined symbols; it does
    /// not capture attribute-only symbols, `$` variables, options, messages, the
    /// gensym counter, PRNG state, or contexts beyond what the source re-establishes.
    pub fn restore_from_bytes(&mut self, bytes: &[u8]) -> Result<(), HostFailure> {
        let source = std::str::from_utf8(bytes).map_err(snapshot_decode_failure)?;
        self.reset();
        if source.trim().is_empty() {
            return Ok(());
        }

        match self.eval(source) {
            HostEvalOutcome::Success(_) => Ok(()),
            HostEvalOutcome::Failure(failure) => Err(failure),
            HostEvalOutcome::InputRequired(suspension) => Err(snapshot_input_failure(suspension)),
        }
    }

    /// Returns the current session configuration.
    ///
    /// The returned value reflects runtime configuration plus the host-level
    /// core-builtin installation flag.
    pub fn configuration(&self) -> HostConfig {
        let config = self.session.configuration();

        HostConfig {
            resource_limits: config.resource_limits.into(),
            capability_policy: config.capability_policy,
            current_context: config.current_context,
            context_path: config.context_path,
            package_search_paths: config.package_search_paths,
            file_io: self.config.file_io.clone(),
            entropy_source: self.config.entropy_source.clone(),
            tracing_enabled: config.tracing_enabled,
            asserts_enabled: config.asserts_enabled,
            install_core_builtins: self.install_core_builtins,
        }
    }

    /// Parses source and returns a host-facing syntax rendering.
    ///
    /// A poisoned session returns [`HostFailureCode::Poisoned`].
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

    /// Evaluates source and returns a structured host outcome.
    ///
    /// Input suspension is reported as [`HostEvalOutcome::InputRequired`] and
    /// should be resumed with [`Self::resume_with_input`].
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

    /// Returns a cancellation handle for this session.
    ///
    /// The handle may be cloned and used from another thread to request
    /// interruption of the current evaluation.
    pub fn cancel_handle(&self) -> HostCancelHandle {
        HostCancelHandle {
            flag: self.session.interrupt_handle(),
        }
    }

    /// Returns summaries of symbols with user-visible definitions.
    pub fn definitions(&self) -> Vec<HostDefinitionSummary> {
        self.session
            .definition_summaries()
            .into_iter()
            .map(Into::into)
            .collect()
    }

    /// Returns the names of contexts known to this session.
    pub fn contexts(&self) -> Vec<String> {
        self.session.context_names()
    }

    /// Evaluates source with a wall-clock timeout for this request.
    ///
    /// The timeout is cleared after the request completes or fails.
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

    /// Resumes a pending `Input` or `InputString` continuation.
    ///
    /// The host should call this only after receiving
    /// [`HostEvalOutcome::InputRequired`] or observing
    /// [`Self::pending_input_request`].
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

    /// Inspects the symbol located at a source cursor offset.
    ///
    /// If no symbol is found at the cursor, the response reports `found: false`.
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

    /// Inspects a symbol by name.
    ///
    /// The name may be qualified or unqualified according to the runtime's
    /// symbol resolution rules.
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

    /// Returns completion candidates and replacement bounds for a cursor.
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

    /// Checks whether source is complete enough to submit for evaluation.
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

    /// Returns the input request currently suspended in this session, if any.
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

fn snapshot_decode_failure(error: std::str::Utf8Error) -> HostFailure {
    synthesized_host_failure(
        HostFailureCode::Other,
        format!("snapshot bytes are not valid UTF-8: {error}"),
    )
}

fn snapshot_input_failure(suspension: HostInputSuspension) -> HostFailure {
    HostFailure {
        code: HostFailureCode::Other,
        message: "snapshot restore unexpectedly requested input".to_string(),
        messages: suspension.messages,
        prints: suspension.prints,
        trace: suspension.trace,
        outputs: suspension.outputs,
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::HostCompletenessStatus;

    #[test]
    fn panic_is_complete_response_marks_session_poisoned_and_invalid() {
        let mut session = HostSession::new();

        let response = session.guard(
            |_session| panic!("host completeness panic boundary test"),
            |_| panic_is_complete_response(),
        );

        assert_eq!(response.status, HostCompletenessStatus::Invalid);
        assert!(response.indent.is_none());
        assert_eq!(
            response.diagnostic.as_deref(),
            Some("host session caught panic")
        );
        assert!(session.poisoned_for_test());

        let poisoned = session.is_complete("1 + 1");
        assert_eq!(poisoned.status, HostCompletenessStatus::Invalid);
        assert!(
            poisoned
                .diagnostic
                .as_deref()
                .is_some_and(|text| text.contains("poisoned"))
        );
    }
}
