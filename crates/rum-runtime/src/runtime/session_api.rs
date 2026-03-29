use super::*;

impl Session {
    pub fn new() -> Self {
        Self::with_config(SessionConfig::default())
    }

    pub fn with_config(config: SessionConfig) -> Self {
        let mut session = Self::default();
        session.apply_config(config);
        session
    }

    pub fn configuration(&self) -> SessionConfig {
        SessionConfig {
            resource_limits: self.resource_limits,
            current_context: self.current_context.clone(),
            context_path: self.context_path.clone(),
            package_search_paths: self.package_search_paths.clone(),
            tracing_enabled: self.tracing_enabled,
        }
    }

    pub fn apply_config(&mut self, config: SessionConfig) {
        let config = config.normalized();
        self.resource_limits = config.resource_limits;
        self.current_context = config.current_context;
        self.context_path = config.context_path;
        self.package_search_paths = config.package_search_paths;
        self.tracing_enabled = config.tracing_enabled;
    }

    pub fn set_recursion_limit(&mut self, limit: usize) {
        self.resource_limits.recursion_limit = limit;
    }

    pub fn set_replace_repeated_limit(&mut self, limit: usize) {
        self.resource_limits.replace_repeated_limit = limit;
    }

    pub fn set_evaluation_pass_limit(&mut self, limit: usize) {
        self.resource_limits.evaluation_pass_limit = limit;
    }

    pub fn set_resource_limits(&mut self, limits: ResourceLimits) {
        self.resource_limits = limits;
    }

    pub fn resource_limits(&self) -> ResourceLimits {
        self.resource_limits
    }

    pub fn enable_tracing(&mut self, enabled: bool) {
        self.tracing_enabled = enabled;
    }

    pub fn trace(&self) -> &[TraceEvent] {
        &self.trace_events
    }

    pub fn take_trace(&mut self) -> Vec<TraceEvent> {
        std::mem::take(&mut self.trace_events)
    }

    pub fn messages(&self) -> &[RuntimeMessage] {
        &self.messages
    }

    pub fn take_messages(&mut self) -> Vec<RuntimeMessage> {
        std::mem::take(&mut self.messages)
    }

    pub fn prints(&self) -> &[String] {
        &self.prints
    }

    pub fn take_prints(&mut self) -> Vec<String> {
        std::mem::take(&mut self.prints)
    }

    pub fn pending_input_request(&self) -> Option<&RuntimeInputRequest> {
        self.pending_input_request.as_ref()
    }

    pub fn take_pending_input_request(&mut self) -> Option<RuntimeInputRequest> {
        self.pending_input_request.take()
    }

    pub fn queue_input_response(&mut self, value: impl Into<String>) {
        self.queued_input_responses.push_back(value.into());
        self.pending_input_request = None;
    }

    pub fn register_builtin(&mut self, spec: BuiltinSpec) {
        self.builtins.insert(spec.name.to_string(), spec);

        if !spec.attributes.is_empty() {
            self.attributes
                .insert(Symbol::new(SYSTEM_CONTEXT, spec.name), spec.attributes);
        }
    }

    pub fn current_context(&self) -> &str {
        &self.current_context
    }

    pub fn context_path(&self) -> &[String] {
        &self.context_path
    }

    pub fn set_current_context(&mut self, context: impl Into<String>) {
        self.current_context = normalize_context_name(context.into());
    }

    pub fn set_context_path<I, S>(&mut self, contexts: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.context_path = contexts
            .into_iter()
            .map(|context| normalize_context_name(context.into()))
            .collect();
    }

    pub fn package_search_paths(&self) -> &[PathBuf] {
        &self.package_search_paths
    }

    pub fn set_package_search_paths<I, P>(&mut self, paths: I)
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.package_search_paths = paths.into_iter().map(Into::into).collect();
    }

    pub fn add_package_search_path(&mut self, path: impl Into<PathBuf>) {
        self.package_search_paths.push(path.into());
    }

    pub fn lower_text(&self, input: &str) -> Result<Expr, EvalError> {
        let syntax = parse_expr(input)?;
        self.lower_syntax_expr(&syntax)
    }

    pub fn complete_symbol_prefix(&self, prefix: &str) -> Vec<String> {
        let prefix = prefix.trim();
        let explicit_context = prefix.contains('`');
        let mut candidates = BTreeSet::new();

        for name in self.builtins.keys() {
            let symbol = Symbol::new(SYSTEM_CONTEXT, name);
            self.insert_completion_candidate(&mut candidates, &symbol, prefix, explicit_context);
        }

        for symbol in self.definitions.keys() {
            self.insert_completion_candidate(&mut candidates, symbol, prefix, explicit_context);
        }

        for symbol in self.attributes.keys() {
            self.insert_completion_candidate(&mut candidates, symbol, prefix, explicit_context);
        }

        for key in self.message_definitions.keys() {
            self.insert_completion_candidate(
                &mut candidates,
                &key.symbol,
                prefix,
                explicit_context,
            );
        }

        for context in self
            .context_path
            .iter()
            .map(String::as_str)
            .chain(std::iter::once(self.current_context.as_str()))
            .chain(self.loaded_packages.iter().map(String::as_str))
        {
            let context = normalize_context_name(context);

            if context.starts_with(prefix) {
                candidates.insert(context);
            }
        }

        candidates.into_iter().collect()
    }

    pub fn loaded_packages(&self) -> &[String] {
        &self.loaded_packages
    }

    pub(super) fn mark_package_loaded(&mut self, context: &str) {
        let context = normalize_context_name(context);

        if !self.loaded_packages.iter().any(|loaded| loaded == &context) {
            self.loaded_packages.push(context);
        }
    }

    fn insert_completion_candidate(
        &self,
        candidates: &mut BTreeSet<String>,
        symbol: &Symbol,
        prefix: &str,
        explicit_context: bool,
    ) {
        let candidate = if explicit_context {
            symbol.fully_qualified_name()
        } else {
            symbol.qualified_name()
        };

        if candidate.starts_with(prefix) {
            candidates.insert(candidate);
        }
    }

    pub(super) fn begin_context(&mut self, context: String) -> Expr {
        let previous_context = self.current_context.clone();
        let previous_context_path = self.context_path.clone();
        self.context_stack.push(ContextFrame {
            kind: ContextFrameKind::Begin,
            previous_context,
            previous_context_path,
            opened_context: context.clone(),
        });
        self.current_context = context.clone();
        self.context_path = prepend_context(&context, &self.context_path);
        Expr::string(context)
    }

    pub(super) fn end_context(&mut self) -> Option<Expr> {
        let frame = self.context_stack.pop()?;

        if frame.kind != ContextFrameKind::Begin {
            self.context_stack.push(frame);
            return None;
        }

        let current = self.current_context.clone();
        self.current_context = frame.previous_context;
        self.context_path = frame.previous_context_path;
        Some(Expr::string(current))
    }

    pub(super) fn begin_package_context(&mut self, context: String) -> Expr {
        let previous_context = self.current_context.clone();
        let previous_context_path = self.context_path.clone();
        self.context_stack.push(ContextFrame {
            kind: ContextFrameKind::BeginPackage,
            previous_context,
            previous_context_path,
            opened_context: context.clone(),
        });
        self.current_context = context.clone();
        self.context_path = vec![context.clone(), SYSTEM_CONTEXT.to_string()];
        self.mark_package_loaded(&context);
        Expr::string(context)
    }

    pub(super) fn end_package_context(&mut self) -> Option<Expr> {
        let frame = self.context_stack.pop()?;

        if frame.kind != ContextFrameKind::BeginPackage {
            self.context_stack.push(frame);
            return None;
        }

        self.current_context = frame.previous_context;
        self.context_path = prepend_context(&frame.opened_context, &frame.previous_context_path);
        Some(system_expr("Null"))
    }
}
