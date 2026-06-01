use rum_runtime::Session;

use crate::config::HostConfig;
use crate::cursor::{completion_prefix, symbol_at_cursor};
use crate::model::{
    HostCompletionRequest, HostCompletionResponse, HostEvalOutcome, HostEvalRequest, HostFailure,
    HostInspectRequest, HostInspectResponse, HostIsCompleteRequest, HostIsCompleteResponse,
    HostParseRequest, HostParseResponse, completeness_status, eval_failure_from_runtime,
    eval_success_from_runtime, host_failure_from_runtime, inspect_response_from_runtime,
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
        let mut session = Session::with_config(self.config.runtime_config());

        if self.config.install_core_builtins {
            rum_stdlib::install_core(&mut session);
        }

        HostSession {
            session,
            install_core_builtins: self.config.install_core_builtins,
        }
    }
}

pub struct HostSession {
    session: Session,
    install_core_builtins: bool,
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
        &self,
        request: impl Into<HostParseRequest>,
    ) -> Result<HostParseResponse, HostFailure> {
        let request = request.into();
        self.session
            .parse_request(&request.source)
            .map(|response| HostParseResponse {
                source: response.parsed.source().to_string(),
                syntax: response.parsed.syntax().to_string(),
            })
            .map_err(|failure| host_failure_from_runtime(failure))
    }

    pub fn eval(&mut self, request: impl Into<HostEvalRequest>) -> HostEvalOutcome {
        let request = request.into();

        match self.session.eval_request(&request.source) {
            Ok(response) => HostEvalOutcome::Success(eval_success_from_runtime(response)),
            Err(failure) => eval_failure_from_runtime(failure),
        }
    }

    pub fn resume_with_input(&mut self, input: impl Into<String>) -> HostEvalOutcome {
        match self.session.resume_eval_request_with_input(input) {
            Ok(response) => HostEvalOutcome::Success(eval_success_from_runtime(response)),
            Err(failure) => eval_failure_from_runtime(failure),
        }
    }

    pub fn inspect(&self, request: HostInspectRequest) -> HostInspectResponse {
        let target = symbol_at_cursor(&request.source, request.cursor_offset);
        let response = target
            .as_deref()
            .and_then(|symbol| self.session.inspect_request(symbol));

        inspect_response_from_runtime(target, response)
    }

    pub fn inspect_symbol(&self, symbol: &str) -> HostInspectResponse {
        inspect_response_from_runtime(
            Some(symbol.to_string()),
            self.session.inspect_request(symbol),
        )
    }

    pub fn complete(&self, request: HostCompletionRequest) -> HostCompletionResponse {
        let (replacement_start, replacement_end, prefix) =
            completion_prefix(&request.source, request.cursor_offset);
        let completion = self.session.complete_request(&prefix);

        HostCompletionResponse {
            prefix: completion.prefix,
            matches: completion.matches,
            replacement_start,
            replacement_end,
        }
    }

    pub fn is_complete(&self, request: impl Into<HostIsCompleteRequest>) -> HostIsCompleteResponse {
        let request = request.into();
        let response = self.session.is_complete_request(&request.source);

        HostIsCompleteResponse {
            status: completeness_status(response.status),
            indent: response.indent,
            diagnostic: response.diagnostic.map(|diagnostic| diagnostic.message),
        }
    }

    pub fn pending_input_request(&self) -> Option<crate::HostInputRequest> {
        self.session
            .pending_input_request()
            .cloned()
            .map(Into::into)
    }
}
