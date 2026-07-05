use super::super::*;
use std::sync::{Arc, atomic::AtomicBool};
use std::time::Duration;

impl Session {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(config: SessionConfig) -> Self {
        let mut session = Self::empty();
        session.apply_config(config);
        session
    }

    pub fn configuration(&self) -> SessionConfig {
        SessionConfig {
            resource_limits: self.resource_limits,
            capability_policy: self.capability_policy,
            current_context: self.current_context.clone(),
            context_path: self.context_path.clone(),
            package_search_paths: self.package_search_paths.clone(),
            tracing_enabled: self.tracing_enabled,
            asserts_enabled: self.asserts_enabled,
        }
    }

    pub fn apply_config(&mut self, config: SessionConfig) {
        let config = config.normalized();
        self.resource_limits = config.resource_limits;
        self.capability_policy = config.capability_policy;
        self.current_context = config.current_context;
        self.context_path = config.context_path;
        self.package_search_paths = config.package_search_paths;
        self.tracing_enabled = config.tracing_enabled;
        self.asserts_enabled = config.asserts_enabled;
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

    pub fn set_generated_element_limit(&mut self, limit: usize) {
        self.resource_limits.generated_element_limit = limit;
    }

    pub fn set_integer_work_limit(&mut self, limit: usize) {
        self.resource_limits.integer_work_limit = limit;
    }

    pub fn set_iteration_work_limit(&mut self, limit: usize) {
        self.resource_limits.iteration_work_limit = limit;
    }

    pub fn interrupt_handle(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.interrupt_flag)
    }

    pub fn set_eval_timeout(&mut self, timeout: Option<Duration>) {
        self.eval_timeout = timeout;
        self.eval_deadline = None;
        self.interrupt_armed =
            self.eval_timeout.is_some() || Arc::strong_count(&self.interrupt_flag) > 1;
    }

    pub fn set_resource_limits(&mut self, limits: ResourceLimits) {
        self.resource_limits = limits;
    }

    pub fn resource_limits(&self) -> ResourceLimits {
        self.resource_limits
    }

    /// Returns the active dynamic memory limit, if one is installed.
    pub fn active_memory_limit(&self) -> Option<usize> {
        self.current_memory_limit
    }

    pub fn set_capability_policy(&mut self, policy: CapabilityPolicy) {
        self.capability_policy = policy;
    }

    pub fn capability_policy(&self) -> CapabilityPolicy {
        self.capability_policy
    }

    pub fn enable_tracing(&mut self, enabled: bool) {
        self.tracing_enabled = enabled;
    }

    pub fn enable_asserts(&mut self, enabled: bool) {
        self.asserts_enabled = enabled;
    }

    pub fn asserts_enabled(&self) -> bool {
        self.asserts_enabled
    }

    pub fn trace(&self) -> &[TraceEvent] {
        &self.trace_events
    }

    pub fn take_trace(&mut self) -> Vec<TraceEvent> {
        self.trace_event_depths.clear();
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
        let symbol = Symbol::system(spec.name);
        let spec = self
            .builtins
            .get(&symbol)
            .copied()
            .map(|fallback| spec.with_fallback_dispatch(fallback))
            .unwrap_or(spec);
        self.builtins.insert(symbol, spec);
        self.builtin_dispatch_spec_cache.clear();

        if !spec.attributes.is_empty() {
            self.attributes.insert(symbol, spec.attributes);
            self.bump_attribute_revision();
        } else {
            self.bump_evaluation_epoch();
        }
    }

    #[doc(hidden)]
    pub fn registered_builtin_specs(&self) -> impl Iterator<Item = &BuiltinSpec> {
        self.builtins.values()
    }

    pub fn current_context(&self) -> &str {
        &self.current_context
    }

    pub fn context_path(&self) -> &[String] {
        &self.context_path
    }

    pub fn definition_summaries(&self) -> Vec<SessionDefinitionSummary> {
        let mut summaries = self
            .definitions
            .iter()
            .map(|(symbol, definition)| SessionDefinitionSummary {
                name: symbol.fully_qualified_name(),
                has_own_value: definition.own_value.is_some()
                    || definition.delayed_own_value.is_some(),
                down_value_count: definition.down_values.len(),
                up_value_count: definition.up_values.len(),
                sub_value_count: definition.sub_values.len(),
            })
            .collect::<Vec<_>>();

        summaries.sort_by(|left, right| left.name.cmp(&right.name));
        summaries
    }

    /// Renders this session's WL-level user definitions as textual dump source.
    ///
    /// The image captures own values, delayed own values, down values, up values,
    /// sub values, and attributes for symbols that have definitions. It does not
    /// capture attribute-only symbols, `$` variables, options, messages, the
    /// gensym counter, or contexts beyond what the generated source
    /// re-establishes. This is a text image, not a binary `.mx` dump.
    pub fn snapshot_definitions_source(&self) -> String {
        let mut symbols = self.definitions.keys().cloned().collect::<Vec<_>>();
        symbols.sort();
        self.render_saved_symbols(&symbols, true, true)
    }

    pub fn context_names(&self) -> Vec<String> {
        let mut contexts = self
            .context_path
            .iter()
            .cloned()
            .chain(std::iter::once(self.current_context.clone()))
            .collect::<BTreeSet<_>>();

        contexts.extend(
            self.definitions
                .keys()
                .map(|symbol| symbol.context().to_string()),
        );

        contexts.into_iter().collect()
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

        for symbol in self.builtins.keys() {
            self.insert_completion_candidate(&mut candidates, symbol, prefix, explicit_context);
        }

        for symbol in self.definitions.keys() {
            self.insert_completion_candidate(&mut candidates, symbol, prefix, explicit_context);
        }

        for symbol in self.attributes.keys() {
            self.insert_completion_candidate(&mut candidates, symbol, prefix, explicit_context);
        }

        for symbol in self.syntax_information.keys() {
            self.insert_completion_candidate(&mut candidates, symbol, prefix, explicit_context);
        }

        for symbol in &self.declared_stubs {
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

    pub(in crate::runtime) fn mark_package_loaded(&mut self, context: &str) {
        let context = normalize_context_name(context);

        if !self.loaded_packages.iter().any(|loaded| loaded == &context) {
            self.loaded_packages.push(context);
            self.bump_evaluation_epoch();
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

    pub(in crate::runtime) fn begin_context(&mut self, context: String) -> Expr {
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

    pub(in crate::runtime) fn end_context(&mut self) -> Option<Expr> {
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

    pub(in crate::runtime) fn begin_package_context(&mut self, context: String) -> Expr {
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

    pub(in crate::runtime) fn end_package_context(&mut self) -> Option<Expr> {
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
