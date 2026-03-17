use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use rum_core::{Atom, BigInt, Expr, GLOBAL_CONTEXT, SYSTEM_CONTEXT, Symbol};
use rum_syntax::{
    BinaryOp, BlankKind, PostfixOp, PrefixOp, SlotSpec, SyntaxError, SyntaxExpr, parse_expr,
};
use thiserror::Error;

const DEFAULT_REPLACE_REPEATED_LIMIT: usize = 256;
const DEFAULT_RECURSION_LIMIT: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoldAttribute {
    None,
    HoldFirst,
    HoldRest,
    HoldAll,
}

impl HoldAttribute {
    fn holds_argument(self, index: usize) -> bool {
        match self {
            Self::None => false,
            Self::HoldFirst => index == 0,
            Self::HoldRest => index > 0,
            Self::HoldAll => true,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BuiltinSpec {
    pub name: &'static str,
    pub summary: &'static str,
    pub hold: HoldAttribute,
}

impl BuiltinSpec {
    pub const fn new(name: &'static str, summary: &'static str) -> Self {
        Self {
            name,
            summary,
            hold: HoldAttribute::None,
        }
    }

    pub const fn with_hold(name: &'static str, summary: &'static str, hold: HoldAttribute) -> Self {
        Self {
            name,
            summary,
            hold,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceLimits {
    pub recursion_limit: usize,
    pub replace_repeated_limit: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            recursion_limit: DEFAULT_RECURSION_LIMIT,
            replace_repeated_limit: DEFAULT_REPLACE_REPEATED_LIMIT,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DefinitionKind {
    OwnValue,
    DelayedOwnValue,
    DownValue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RuleApplicationSource {
    DownValue,
    ReplaceAll,
    ReplaceRepeated,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TraceEvent {
    EvaluationPass {
        before: String,
        after: String,
    },
    DefinitionStored {
        symbol: String,
        kind: DefinitionKind,
        value: String,
    },
    RuleApplied {
        source: RuleApplicationSource,
        input: String,
        rule: String,
        result: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RuntimeMessage {
    pub symbol: String,
    pub tag: String,
    pub text: String,
}

impl fmt::Display for RuntimeMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}::{}: {}", self.symbol, self.tag, self.text)
    }
}

#[derive(Clone, Debug)]
pub struct Session {
    builtins: BTreeMap<String, BuiltinSpec>,
    current_context: String,
    context_path: Vec<String>,
    context_stack: Vec<ContextFrame>,
    package_search_paths: Vec<PathBuf>,
    loaded_packages: Vec<String>,
    load_stack: Vec<PathBuf>,
    definitions: BTreeMap<Symbol, SymbolDefinition>,
    next_generated_symbol_id: u64,
    resource_limits: ResourceLimits,
    evaluation_depth: usize,
    evaluation_stack: Vec<Expr>,
    tracing_enabled: bool,
    trace_events: Vec<TraceEvent>,
    messages: Vec<RuntimeMessage>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EvalOutcome {
    pub value: Expr,
    pub messages: Vec<RuntimeMessage>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SymbolInfo {
    pub qualified_name: String,
    pub summary: Option<String>,
    pub is_builtin: bool,
    pub has_own_value: bool,
    pub has_delayed_value: bool,
    pub has_down_values: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SymbolDefinition {
    pub own_value: Option<Expr>,
    pub delayed_own_value: Option<Expr>,
    pub down_values: Vec<DownValue>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DownValue {
    pub lhs: Expr,
    pub rhs: Expr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct RewriteRule {
    lhs: Expr,
    rhs: Expr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CasesSpec {
    Pattern(Expr),
    Rule(RewriteRule),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ContextFrameKind {
    Begin,
    BeginPackage,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ContextFrame {
    kind: ContextFrameKind,
    previous_context: String,
    previous_context_path: Vec<String>,
    opened_context: String,
}

#[derive(Clone, Debug)]
struct LoaderSnapshot {
    current_context: String,
    context_path: Vec<String>,
    context_stack: Vec<ContextFrame>,
    loaded_packages: Vec<String>,
}

#[derive(Clone, Copy, Debug)]
struct SequencePatternSpec<'a> {
    min_len: usize,
    head_constraint: Option<&'a Expr>,
    name: Option<&'a Symbol>,
    condition: Option<&'a Expr>,
}

#[derive(Debug, Error)]
pub enum EvalError {
    #[error(transparent)]
    Syntax(#[from] SyntaxError),
    #[error("invalid integer literal `{literal}`")]
    InvalidInteger { literal: String },
    #[error("could not read `{path}`: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
    #[error("could not resolve package or file `{target}`")]
    LoadPathNotFound {
        target: String,
        searched: Vec<String>,
    },
    #[error("evaluation cycle detected while re-entering `{expr}`")]
    EvaluationCycleDetected { expr: String },
    #[error("evaluation exceeded recursion limit ({limit})")]
    RecursionLimitExceeded { limit: usize },
    #[error("ReplaceRepeated exceeded iteration limit ({limit})")]
    IterationLimitExceeded { limit: usize },
}

impl Default for Session {
    fn default() -> Self {
        Self {
            builtins: BTreeMap::new(),
            current_context: GLOBAL_CONTEXT.to_string(),
            context_path: vec![SYSTEM_CONTEXT.to_string(), GLOBAL_CONTEXT.to_string()],
            context_stack: Vec::new(),
            package_search_paths: vec![PathBuf::from(".")],
            loaded_packages: Vec::new(),
            load_stack: Vec::new(),
            definitions: BTreeMap::new(),
            next_generated_symbol_id: 0,
            resource_limits: ResourceLimits::default(),
            evaluation_depth: 0,
            evaluation_stack: Vec::new(),
            tracing_enabled: false,
            trace_events: Vec::new(),
            messages: Vec::new(),
        }
    }
}

impl Session {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_recursion_limit(&mut self, limit: usize) {
        self.resource_limits.recursion_limit = limit;
    }

    pub fn set_replace_repeated_limit(&mut self, limit: usize) {
        self.resource_limits.replace_repeated_limit = limit;
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

    pub fn register_builtin(&mut self, spec: BuiltinSpec) {
        self.builtins.insert(spec.name.to_string(), spec);
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

    pub fn loaded_packages(&self) -> &[String] {
        &self.loaded_packages
    }

    fn mark_package_loaded(&mut self, context: &str) {
        let context = normalize_context_name(context);

        if !self.loaded_packages.iter().any(|loaded| loaded == &context) {
            self.loaded_packages.push(context);
        }
    }

    fn begin_context(&mut self, context: String) -> Expr {
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

    fn end_context(&mut self) -> Option<Expr> {
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

    fn begin_package_context(&mut self, context: String) -> Expr {
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

    fn end_package_context(&mut self) -> Option<Expr> {
        let frame = self.context_stack.pop()?;

        if frame.kind != ContextFrameKind::BeginPackage {
            self.context_stack.push(frame);
            return None;
        }

        self.current_context = frame.previous_context;
        self.context_path = prepend_context(&frame.opened_context, &frame.previous_context_path);
        Some(system_expr("Null"))
    }

    pub fn inspect(&self, name: &str) -> Option<SymbolInfo> {
        let symbol = self.resolve_symbol_name(name, false)?;
        let builtin = self.builtin_spec_for_symbol(&symbol);
        let definition = self.definitions.get(&symbol);

        if builtin.is_none() && definition.is_none() && !is_known_system_symbol(&symbol) {
            return None;
        }

        Some(SymbolInfo {
            qualified_name: symbol.fully_qualified_name(),
            summary: builtin.map(|builtin| builtin.summary.to_string()),
            is_builtin: builtin.is_some() || is_known_system_symbol(&symbol),
            has_own_value: definition.and_then(|def| def.own_value.as_ref()).is_some(),
            has_delayed_value: definition
                .and_then(|def| def.delayed_own_value.as_ref())
                .is_some(),
            has_down_values: definition
                .map(|definition| !definition.down_values.is_empty())
                .unwrap_or(false),
        })
    }

    pub fn eval_text(&mut self, input: &str) -> Result<EvalOutcome, EvalError> {
        let message_checkpoint = self.messages.len();
        let syntax = parse_expr(input)?;
        let expr = self.lower_syntax_expr(&syntax)?;
        let value = self.evaluate(expr)?;
        Ok(EvalOutcome {
            value,
            messages: self.messages.split_off(message_checkpoint),
        })
    }

    pub fn eval_script(&mut self, source: &str) -> Result<EvalOutcome, EvalError> {
        let forms = split_script_into_forms(source)?;
        let mut value = system_expr("Null");
        let mut messages = Vec::new();

        for form in forms {
            let outcome = self.eval_text(&form)?;
            value = outcome.value;
            messages.extend(outcome.messages);
        }

        Ok(EvalOutcome { value, messages })
    }

    pub fn eval_file(&mut self, path: impl AsRef<Path>) -> Result<EvalOutcome, EvalError> {
        let path = path.as_ref().to_path_buf();
        let source = fs::read_to_string(&path).map_err(|source| EvalError::Io {
            path: path.display().to_string(),
            source,
        })?;
        let snapshot = self.capture_loader_snapshot();

        self.load_stack.push(path);
        let result = self.eval_script(&source);
        self.load_stack.pop();

        if result.is_err() {
            self.restore_loader_snapshot(snapshot);
        }

        result
    }

    fn capture_loader_snapshot(&self) -> LoaderSnapshot {
        LoaderSnapshot {
            current_context: self.current_context.clone(),
            context_path: self.context_path.clone(),
            context_stack: self.context_stack.clone(),
            loaded_packages: self.loaded_packages.clone(),
        }
    }

    fn restore_loader_snapshot(&mut self, snapshot: LoaderSnapshot) {
        self.current_context = snapshot.current_context;
        self.context_path = snapshot.context_path;
        self.context_stack = snapshot.context_stack;
        self.loaded_packages = snapshot.loaded_packages;
    }

    fn push_message(&mut self, symbol: &str, tag: &str, text: impl Into<String>) {
        self.messages.push(RuntimeMessage {
            symbol: symbol.to_string(),
            tag: tag.to_string(),
            text: text.into(),
        });
    }

    fn emit_loader_message(&mut self, symbol: &str, error: &EvalError) {
        match error {
            EvalError::LoadPathNotFound { target, searched } => {
                let detail = if searched.is_empty() {
                    format!("could not locate `{target}`")
                } else {
                    format!("could not locate `{target}` in {}", searched.join(", "))
                };
                self.push_message(symbol, "notfound", detail);
            }
            EvalError::Io { path, source } => {
                self.push_message(symbol, "io", format!("could not read `{path}`: {source}"));
            }
            EvalError::Syntax(err) => {
                self.push_message(symbol, "syntax", err.to_string());
            }
            other => {
                self.push_message(symbol, "load", other.to_string());
            }
        }
    }

    fn invalid_call(&self, symbol: &str, args: &[Expr]) -> Expr {
        Expr::call(system_expr(symbol), args.to_vec())
    }

    fn invalid_call_with_message(
        &mut self,
        symbol: &str,
        args: &[Expr],
        tag: &str,
        text: impl Into<String>,
    ) -> Expr {
        self.push_message(symbol, tag, text);
        self.invalid_call(symbol, args)
    }

    fn invalid_call_with_exact_arg_count(
        &mut self,
        symbol: &str,
        args: &[Expr],
        expected: usize,
    ) -> Expr {
        self.invalid_call_with_message(
            symbol,
            args,
            "argx",
            format!(
                "expected exactly {expected} argument(s), got {}",
                args.len()
            ),
        )
    }

    fn invalid_call_with_arg_range(
        &mut self,
        symbol: &str,
        args: &[Expr],
        min: usize,
        max: usize,
    ) -> Expr {
        self.invalid_call_with_message(
            symbol,
            args,
            "argtu",
            format!(
                "expected between {min} and {max} argument(s), got {}",
                args.len()
            ),
        )
    }

    fn invalid_call_with_arg_form(
        &mut self,
        symbol: &str,
        args: &[Expr],
        detail: impl Into<String>,
    ) -> Expr {
        self.invalid_call_with_message(symbol, args, "argform", detail)
    }

    fn lower_syntax_expr(&self, syntax: &SyntaxExpr) -> Result<Expr, EvalError> {
        lower_syntax_expr_with(syntax, &mut |raw| self.resolve_symbol_name(raw, true))
    }

    fn builtin_spec_for_symbol(&self, symbol: &Symbol) -> Option<&BuiltinSpec> {
        (symbol.context() == SYSTEM_CONTEXT)
            .then(|| self.builtins.get(symbol.name()))
            .flatten()
    }

    fn resolve_symbol_name(&self, raw: &str, create_if_missing: bool) -> Option<Symbol> {
        if let Some((context, name)) = split_symbol_name(raw) {
            return Some(Symbol::new(normalize_context_name(context), name));
        }

        let mut seen_contexts = BTreeSet::new();

        for context in self
            .context_path
            .iter()
            .map(String::as_str)
            .chain(std::iter::once(self.current_context.as_str()))
        {
            if !seen_contexts.insert(context) {
                continue;
            }

            let symbol = Symbol::new(context.to_string(), raw);

            if self.symbol_exists(&symbol) {
                return Some(symbol);
            }
        }

        create_if_missing.then(|| Symbol::new(self.current_context.clone(), raw))
    }

    fn symbol_exists(&self, symbol: &Symbol) -> bool {
        self.definitions.contains_key(symbol)
            || self.builtin_spec_for_symbol(symbol).is_some()
            || is_known_system_symbol(symbol)
    }

    fn resolve_load_target(&self, spec: &str) -> Result<PathBuf, EvalError> {
        self.find_load_target(spec)
            .map(|(path, _)| path)
            .ok_or_else(|| {
                let (_, searched) = self.load_target_candidates(spec);
                EvalError::LoadPathNotFound {
                    target: spec.to_string(),
                    searched,
                }
            })
    }

    fn load_search_roots(&self) -> Vec<PathBuf> {
        let mut roots = Vec::new();

        if let Some(parent) = self
            .load_stack
            .last()
            .and_then(|path| path.parent().map(Path::to_path_buf))
        {
            roots.push(parent);
        }

        roots.extend(self.package_search_paths.iter().cloned());
        dedup_paths(roots)
    }

    fn find_load_target(&self, spec: &str) -> Option<(PathBuf, Vec<String>)> {
        let (candidates, searched) = self.load_target_candidates(spec);

        for candidate in candidates {
            if candidate.is_file() {
                return Some((normalize_found_path(&candidate), searched));
            }
        }

        None
    }

    fn load_target_candidates(&self, spec: &str) -> (Vec<PathBuf>, Vec<String>) {
        let specs = if spec.ends_with('`') {
            package_candidates_for_context(spec)
        } else {
            vec![PathBuf::from(spec)]
        };

        let mut candidates = Vec::new();
        let mut searched = Vec::new();

        for spec in specs {
            if spec.is_absolute() {
                searched.push(spec.display().to_string());
                candidates.push(spec);
                continue;
            }

            for root in self.load_search_roots() {
                let path = root.join(&spec);
                searched.push(path.display().to_string());
                candidates.push(path);
            }
        }

        (candidates, searched)
    }

    pub fn evaluate(&mut self, expr: Expr) -> Result<Expr, EvalError> {
        let mut current = expr;
        let mut fixed_point_seen = Vec::new();

        loop {
            if fixed_point_seen.iter().any(|previous| previous == &current) {
                return Err(EvalError::EvaluationCycleDetected {
                    expr: current.to_string(),
                });
            }

            fixed_point_seen.push(current.clone());
            let next = self.evaluate_once(current.clone())?;

            if next == current {
                return Ok(next);
            }

            self.push_trace(TraceEvent::EvaluationPass {
                before: current.to_string(),
                after: next.to_string(),
            });

            current = next;
        }
    }

    fn evaluate_once(&mut self, expr: Expr) -> Result<Expr, EvalError> {
        if self.evaluation_stack.iter().any(|active| active == &expr) {
            return Err(EvalError::EvaluationCycleDetected {
                expr: expr.to_string(),
            });
        }

        if self.evaluation_depth >= self.resource_limits.recursion_limit {
            return Err(EvalError::RecursionLimitExceeded {
                limit: self.resource_limits.recursion_limit,
            });
        }

        self.evaluation_stack.push(expr.clone());
        self.evaluation_depth += 1;
        let result = self.evaluate_inner(expr);
        self.evaluation_depth -= 1;
        self.evaluation_stack.pop();
        result
    }

    fn evaluate_inner(&mut self, expr: Expr) -> Result<Expr, EvalError> {
        match expr {
            Expr::Atom(Atom::Symbol(symbol)) => self.evaluate_symbol(symbol),
            Expr::Call { head, args } => self.evaluate_call(*head, args),
            other => Ok(other),
        }
    }

    fn evaluate_symbol(&mut self, symbol: Symbol) -> Result<Expr, EvalError> {
        if symbol.context() == SYSTEM_CONTEXT {
            match symbol.name() {
                "$Context" => return Ok(Expr::string(self.current_context.clone())),
                "$ContextPath" => {
                    let contexts = self
                        .context_path
                        .iter()
                        .cloned()
                        .map(Expr::string)
                        .collect::<Vec<_>>();
                    return Ok(Expr::call(system_expr("List"), contexts));
                }
                "$Packages" => {
                    let packages = self
                        .loaded_packages
                        .iter()
                        .cloned()
                        .map(Expr::string)
                        .collect::<Vec<_>>();
                    return Ok(Expr::call(system_expr("List"), packages));
                }
                "$Path" => {
                    let paths = self
                        .package_search_paths
                        .iter()
                        .map(|path| Expr::string(path.display().to_string()))
                        .collect::<Vec<_>>();
                    return Ok(Expr::call(system_expr("List"), paths));
                }
                _ => {}
            }
        }

        let definition = self.definitions.get(&symbol).cloned();

        match definition {
            Some(SymbolDefinition {
                own_value: Some(value),
                ..
            }) => self.evaluate(value),
            Some(SymbolDefinition {
                delayed_own_value: Some(value),
                ..
            }) => self.evaluate(value),
            _ => Ok(Expr::Atom(Atom::Symbol(symbol))),
        }
    }

    fn evaluate_call(&mut self, head: Expr, args: Vec<Expr>) -> Result<Expr, EvalError> {
        let evaluated_head = self.evaluate(head)?;

        if let Some(result) = self.try_evaluate_special_form(&evaluated_head, args.as_slice())? {
            return Ok(result);
        }

        let evaluated_args = self.evaluate_call_args(&evaluated_head, args)?;

        if let Some(result) =
            self.try_apply_down_values(&evaluated_head, evaluated_args.as_slice())?
        {
            return Ok(result);
        }

        if let Some(result) =
            self.try_evaluate_non_held_builtin(&evaluated_head, evaluated_args.as_slice())?
        {
            return Ok(result);
        }

        if let Some(result) = self.try_apply_function(&evaluated_head, evaluated_args.as_slice())? {
            return Ok(result);
        }

        Ok(Expr::call(evaluated_head, evaluated_args))
    }

    fn evaluate_call_args(&mut self, head: &Expr, args: Vec<Expr>) -> Result<Vec<Expr>, EvalError> {
        let hold = self.hold_attribute_for_head(head);
        let mut evaluated_args = Vec::new();

        for (index, arg) in args.into_iter().enumerate() {
            let (evaluated_arg, allow_sequence_splice) = if hold.holds_argument(index) {
                self.evaluate_held_argument(arg)?
            } else {
                (self.evaluate(arg)?, true)
            };

            if allow_sequence_splice {
                splice_sequence_argument(&mut evaluated_args, evaluated_arg);
            } else {
                evaluated_args.push(evaluated_arg);
            }
        }

        Ok(evaluated_args)
    }

    fn evaluate_held_argument(&mut self, arg: Expr) -> Result<(Expr, bool), EvalError> {
        if let Some(inner) = evaluate_wrapper_arg(&arg) {
            return self.evaluate(inner.clone()).map(|value| (value, true));
        }

        Ok((arg, false))
    }

    fn hold_attribute_for_head(&self, head: &Expr) -> HoldAttribute {
        let Some(symbol) = expr_as_symbol(head) else {
            return HoldAttribute::None;
        };

        if symbol.context() != SYSTEM_CONTEXT {
            return HoldAttribute::None;
        }

        self.builtin_spec_for_symbol(symbol)
            .map(|spec| spec.hold)
            .unwrap_or_else(|| fallback_hold_attribute(symbol.name()))
    }

    fn try_evaluate_special_form(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Option<Expr>, EvalError> {
        let Some(symbol) = expr_as_symbol(head) else {
            return Ok(None);
        };

        if symbol.context() != SYSTEM_CONTEXT {
            return Ok(None);
        }

        match symbol.name() {
            "Set" => self.eval_set(args).map(Some),
            "SetDelayed" => self.eval_set_delayed(args).map(Some),
            "Rule" => self.eval_rule(args).map(Some),
            "RuleDelayed" => self.eval_rule_delayed(args).map(Some),
            "MatchQ" => self.eval_match_q(args).map(Some),
            "Cases" => self.eval_cases(args).map(Some),
            "Module" => self.eval_module(args).map(Some),
            "Block" => self.eval_block(args).map(Some),
            "ReplaceAll" => self.eval_replace_all(args).map(Some),
            "ReplaceRepeated" => self.eval_replace_repeated(args).map(Some),
            _ => Ok(None),
        }
    }

    fn try_evaluate_non_held_builtin(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Option<Expr>, EvalError> {
        let Some(symbol) = expr_as_symbol(head) else {
            return Ok(None);
        };

        if symbol.context() != SYSTEM_CONTEXT {
            return Ok(None);
        }

        match symbol.name() {
            "Identity" if args.len() == 1 => Ok(Some(args[0].clone())),
            "FindFile" => self.eval_find_file(args).map(Some),
            "Get" => self.eval_get(args).map(Some),
            "Needs" => self.eval_needs(args).map(Some),
            "Context" => self.eval_context(args).map(Some),
            "Begin" => self.eval_begin(args).map(Some),
            "End" => self.eval_end(args).map(Some),
            "BeginPackage" => self.eval_begin_package(args).map(Some),
            "EndPackage" => self.eval_end_package(args).map(Some),
            "Check" => self.eval_check(args).map(Some),
            "Quiet" => self.eval_quiet(args).map(Some),
            "If" => self.eval_if(args).map(Some),
            "Which" => self.eval_which(args).map(Some),
            "With" => self.eval_with(args).map(Some),
            "Hold" | "HoldFirst" | "HoldRest" | "HoldAll" => {
                Ok(Some(Expr::call(head.clone(), args.to_vec())))
            }
            "Evaluate" if args.len() == 1 => self.evaluate(args[0].clone()).map(Some),
            "Sequence" => Ok(Some(sequence_expr(args.to_vec()))),
            _ => Ok(None),
        }
    }

    fn eval_context(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [] => Ok(Expr::string(self.current_context.clone())),
            [Expr::Atom(Atom::Symbol(symbol))] => Ok(Expr::string(symbol.context().to_string())),
            [Expr::Atom(Atom::String(raw_name))] => self
                .resolve_symbol_name(raw_name, false)
                .map(|symbol| Expr::string(symbol.context().to_string()))
                .map_or_else(|| Ok(Expr::call(system_expr("Context"), args.to_vec())), Ok),
            [_] => Ok(self.invalid_call_with_arg_form(
                "Context",
                args,
                "expected a symbol or string argument",
            )),
            _ => Ok(self.invalid_call_with_message(
                "Context",
                args,
                "argx",
                format!("expected 0 or 1 argument(s), got {}", args.len()),
            )),
        }
    }

    fn eval_find_file(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [spec] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("FindFile", args, 1));
        };

        let Some(spec) = load_spec_from_expr(spec) else {
            return Ok(self.invalid_call_with_arg_form(
                "FindFile",
                args,
                "expected a string or File[\"...\"] load target",
            ));
        };

        Ok(match self.find_load_target(&spec) {
            Some((path, _)) => Expr::string(path.display().to_string()),
            None => {
                self.emit_loader_message(
                    "FindFile",
                    &EvalError::LoadPathNotFound {
                        target: spec.clone(),
                        searched: self.load_target_candidates(&spec).1,
                    },
                );
                system_expr("$Failed")
            }
        })
    }

    fn eval_get(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [spec] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Get", args, 1));
        };

        let Some(spec) = load_spec_from_expr(spec) else {
            return Ok(self.invalid_call_with_arg_form(
                "Get",
                args,
                "expected a string or File[\"...\"] load target",
            ));
        };

        let path = match self.resolve_load_target(&spec) {
            Ok(path) => path,
            Err(err) => {
                self.emit_loader_message("Get", &err);
                return Ok(system_expr("$Failed"));
            }
        };

        match self.eval_file(path) {
            Ok(outcome) => {
                self.messages.extend(outcome.messages);
                Ok(outcome.value)
            }
            Err(err) => {
                self.emit_loader_message("Get", &err);
                Ok(system_expr("$Failed"))
            }
        }
    }

    fn eval_needs(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let (context, load_spec) = match args {
            [Expr::Atom(Atom::String(context))] => (
                normalize_context_name(context),
                normalize_context_name(context),
            ),
            [Expr::Atom(Atom::String(context)), spec] => {
                let Some(spec) = load_spec_from_expr(spec) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "Needs",
                        args,
                        "expected a string or File[\"...\"] as the explicit load target",
                    ));
                };

                (normalize_context_name(context), spec)
            }
            [_, ..] if args.len() <= 2 => {
                return Ok(self.invalid_call_with_arg_form(
                    "Needs",
                    args,
                    "expected a string context name, optionally followed by a string or File[\"...\"] load target",
                ));
            }
            _ => {
                return Ok(self.invalid_call_with_message(
                    "Needs",
                    args,
                    "argx",
                    format!("expected 1 or 2 argument(s), got {}", args.len()),
                ));
            }
        };

        if self.loaded_packages.iter().any(|loaded| loaded == &context) {
            return Ok(system_expr("Null"));
        }

        let result = self.eval_get(&[Expr::string(load_spec)])?;

        if is_system_symbol_name(&result, "$Failed") {
            self.push_message(
                "Needs",
                "load",
                format!("context `{context}` could not be loaded"),
            );
            return Ok(system_expr("$Failed"));
        }

        self.mark_package_loaded(&context);
        self.context_path = prepend_context(&context, &self.context_path);
        Ok(system_expr("Null"))
    }

    fn eval_begin(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [context] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Begin", args, 1));
        };

        let Expr::Atom(Atom::String(context)) = context else {
            return Ok(self.invalid_call_with_arg_form(
                "Begin",
                args,
                "expected a string context name",
            ));
        };

        let target = expand_context_spec(context, &self.current_context);
        Ok(self.begin_context(target))
    }

    fn eval_end(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !args.is_empty() {
            return Ok(self.invalid_call_with_exact_arg_count("End", args, 0));
        }

        Ok(self.end_context().unwrap_or_else(|| {
            self.invalid_call_with_message(
                "End",
                &[],
                "context",
                "no matching Begin[] frame is active",
            )
        }))
    }

    fn eval_begin_package(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let (context, needs) = match args {
            [Expr::Atom(Atom::String(context))] => (context, Vec::new()),
            [Expr::Atom(Atom::String(context)), needs] => {
                let Some(needs) = expr_as_string_list(needs) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "BeginPackage",
                        args,
                        "expected the dependency list to be a list of strings",
                    ));
                };

                (context, needs)
            }
            [_, ..] if args.len() <= 2 => {
                return Ok(self.invalid_call_with_arg_form(
                    "BeginPackage",
                    args,
                    "expected a string context name, optionally followed by a list of string dependencies",
                ));
            }
            _ => {
                return Ok(self.invalid_call_with_message(
                    "BeginPackage",
                    args,
                    "argx",
                    format!("expected 1 or 2 argument(s), got {}", args.len()),
                ));
            }
        };

        let target = expand_context_spec(context, &self.current_context);
        let result = self.begin_package_context(target.clone());

        for need in needs {
            let context = normalize_context_name(need);
            self.eval_needs(&[Expr::string(context)])?;
        }

        self.context_path = prepend_context(&target, &self.context_path);

        Ok(result)
    }

    fn eval_end_package(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !args.is_empty() {
            return Ok(self.invalid_call_with_exact_arg_count("EndPackage", args, 0));
        }

        Ok(self.end_package_context().unwrap_or_else(|| {
            self.invalid_call_with_message(
                "EndPackage",
                &[],
                "context",
                "no matching BeginPackage[] frame is active",
            )
        }))
    }

    fn eval_check(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Check", args, 2));
        }

        let message_checkpoint = self.messages.len();
        let value = self.evaluate(args[0].clone())?;

        if self.messages.len() > message_checkpoint {
            self.evaluate(args[1].clone())
        } else {
            Ok(value)
        }
    }

    fn eval_quiet(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("Quiet", args, 1));
        }

        let message_checkpoint = self.messages.len();
        let result = self.evaluate(args[0].clone());
        self.messages.truncate(message_checkpoint);
        result
    }

    fn eval_if(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(2..=4).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("If", args, 2, 4));
        }

        match truthiness_symbol(&args[0]) {
            Some(true) => self.evaluate(args[1].clone()),
            Some(false) => {
                if args.len() >= 3 {
                    self.evaluate(args[2].clone())
                } else {
                    Ok(system_expr("Null"))
                }
            }
            None if args.len() == 4 => self.evaluate(args[3].clone()),
            None => Ok(Expr::call(system_expr("If"), args.to_vec())),
        }
    }

    fn eval_which(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.is_empty() || args.len() % 2 != 0 {
            return Ok(self.invalid_call_with_arg_form(
                "Which",
                args,
                "expected one or more condition/result pairs",
            ));
        }

        let mut index = 0;

        while index < args.len() {
            let condition = self.evaluate(args[index].clone())?;

            match truthiness_symbol(&condition) {
                Some(true) => return self.evaluate(args[index + 1].clone()),
                Some(false) => index += 2,
                None => {
                    let mut residual = args.to_vec();
                    residual[index] = condition;
                    return Ok(Expr::call(system_expr("Which"), residual));
                }
            }
        }

        Ok(system_expr("Null"))
    }

    fn eval_with(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("With", args, 2));
        }

        let Some(bindings) = self.evaluate_with_bindings(&args[0])? else {
            return Ok(self.invalid_call_with_arg_form(
                "With",
                args,
                "expected a list of symbol assignments or rules as the first argument",
            ));
        };

        let body = substitute_lexical_bindings(&args[1], &bindings, &BTreeSet::new());
        self.evaluate(body)
    }

    fn eval_module(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Module", args, 2));
        }

        let Some(items) = parse_module_binding_list(&args[0]) else {
            return Ok(self.invalid_call_with_arg_form(
                "Module",
                args,
                "expected a list of symbols or symbol initializers as the first argument",
            ));
        };

        let mut renames = BTreeMap::new();

        for (symbol, _) in &items {
            renames.insert(
                symbol.clone(),
                Expr::Atom(Atom::Symbol(self.allocate_fresh_symbol(symbol))),
            );
        }

        for (symbol, init) in items {
            let Some(fresh_expr) = renames.get(&symbol) else {
                continue;
            };
            let Some(fresh_symbol) = expr_as_symbol(fresh_expr).cloned() else {
                continue;
            };

            let definition = self.definitions.entry(fresh_symbol.clone()).or_default();
            definition.own_value = None;
            definition.delayed_own_value = None;

            if let Some(init) = init {
                let renamed_init = substitute_lexical_bindings(&init, &renames, &BTreeSet::new());
                let value = self.evaluate(renamed_init)?;
                let definition = self.definitions.entry(fresh_symbol).or_default();
                definition.own_value = Some(value);
                definition.delayed_own_value = None;
            }
        }

        let body = substitute_lexical_bindings(&args[1], &renames, &BTreeSet::new());
        self.evaluate(body)
    }

    fn eval_block(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Block", args, 2));
        }

        let Some(items) = parse_module_binding_list(&args[0]) else {
            return Ok(self.invalid_call_with_arg_form(
                "Block",
                args,
                "expected a list of symbols or symbol initializers as the first argument",
            ));
        };

        let mut saved_definitions = BTreeMap::new();

        for (symbol, _) in &items {
            saved_definitions
                .entry(symbol.clone())
                .or_insert_with(|| self.definitions.get(symbol).cloned());
        }

        let result = (|| -> Result<Expr, EvalError> {
            for (symbol, _) in &items {
                let definition = self.definitions.entry(symbol.clone()).or_default();
                definition.own_value = None;
                definition.delayed_own_value = None;
            }

            for (symbol, init) in &items {
                if let Some(init) = init {
                    let value = self.evaluate(init.clone())?;
                    let definition = self.definitions.entry(symbol.clone()).or_default();
                    definition.own_value = Some(value);
                    definition.delayed_own_value = None;
                }
            }

            self.evaluate(args[1].clone())
        })();

        for (symbol, prior_definition) in saved_definitions {
            match prior_definition {
                Some(definition) => {
                    self.definitions.insert(symbol, definition);
                }
                None => {
                    self.definitions.remove(&symbol);
                }
            }
        }

        result
    }

    fn try_apply_function(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Option<Expr>, EvalError> {
        let Some(function) = parse_function(head) else {
            return Ok(None);
        };

        let applied = match function {
            FunctionSpec::SlotBased(body) => substitute_slots(body, args),
            FunctionSpec::Explicit { params, body } => {
                if params.len() != args.len() {
                    self.push_message(
                        "Function",
                        "argx",
                        format!(
                            "expected exactly {} argument(s), got {}",
                            params.len(),
                            args.len()
                        ),
                    );
                    return Ok(Some(Expr::call(head.clone(), args.to_vec())));
                }

                let bindings = params.into_iter().zip(args.iter().cloned()).collect();
                substitute_lexical_bindings(body, &bindings, &BTreeSet::new())
            }
        };

        self.evaluate(applied).map(Some)
    }

    fn evaluate_with_bindings(
        &mut self,
        expr: &Expr,
    ) -> Result<Option<BTreeMap<Symbol, Expr>>, EvalError> {
        let Expr::Call { head, args } = expr else {
            return Ok(None);
        };

        if !is_system_symbol_name(head, "List") {
            return Ok(None);
        }

        let mut bindings = BTreeMap::new();

        for item in args {
            let Some((symbol, rhs)) = parse_with_binding_item(item) else {
                return Ok(None);
            };

            let substituted_rhs = substitute_lexical_bindings(&rhs, &bindings, &BTreeSet::new());
            let value = self.evaluate(substituted_rhs)?;
            bindings.insert(symbol, value);
        }

        Ok(Some(bindings))
    }

    fn allocate_fresh_symbol(&mut self, symbol: &Symbol) -> Symbol {
        self.next_generated_symbol_id += 1;
        Symbol::new(
            symbol.context().to_string(),
            format!("{}${}", symbol.name(), self.next_generated_symbol_id),
        )
    }

    fn eval_set(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Set", args, 2));
        }

        let lhs = args[0].clone();
        let rhs = self.evaluate(args[1].clone())?;

        if let Some(symbol) = expr_as_symbol(&lhs).cloned() {
            let definition = self.definitions.entry(symbol).or_default();
            definition.own_value = Some(rhs.clone());
            definition.delayed_own_value = None;
            self.push_trace(TraceEvent::DefinitionStored {
                symbol: lhs.to_string(),
                kind: DefinitionKind::OwnValue,
                value: rhs.to_string(),
            });
            Ok(rhs)
        } else {
            Ok(Expr::call(system_expr("Set"), vec![lhs, rhs]))
        }
    }

    fn eval_set_delayed(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("SetDelayed", args, 2));
        }

        let lhs = args[0].clone();
        let rhs = args[1].clone();

        if let Some(symbol) = expr_as_symbol(&lhs).cloned() {
            let definition = self.definitions.entry(symbol).or_default();
            definition.own_value = None;
            definition.delayed_own_value = Some(rhs.clone());
            self.push_trace(TraceEvent::DefinitionStored {
                symbol: lhs.to_string(),
                kind: DefinitionKind::DelayedOwnValue,
                value: rhs.to_string(),
            });
            Ok(system_expr("Null"))
        } else if let Some(head_symbol) = downvalue_head_symbol(&lhs).cloned() {
            let trace_symbol = head_symbol.to_string();
            let definition = self.definitions.entry(head_symbol).or_default();
            definition.down_values.push(DownValue {
                lhs: lhs.clone(),
                rhs: rhs.clone(),
            });
            self.push_trace(TraceEvent::DefinitionStored {
                symbol: trace_symbol,
                kind: DefinitionKind::DownValue,
                value: format!("{lhs} := {rhs}"),
            });
            Ok(system_expr("Null"))
        } else {
            Ok(Expr::call(system_expr("SetDelayed"), vec![lhs, rhs]))
        }
    }

    fn eval_rule(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Rule", args, 2));
        }

        Ok(Expr::call(
            system_expr("Rule"),
            vec![args[0].clone(), self.evaluate(args[1].clone())?],
        ))
    }

    fn eval_rule_delayed(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("RuleDelayed", args, 2));
        }

        Ok(Expr::call(system_expr("RuleDelayed"), args.to_vec()))
    }

    fn eval_match_q(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("MatchQ", args, 2));
        }

        let value = self.evaluate(args[0].clone())?;
        let pattern = self.normalize_pattern_expr(&args[1])?;
        let mut bindings = BTreeMap::new();

        Ok(system_expr(
            if self.match_expr(&pattern, &value, &mut bindings)? {
                "True"
            } else {
                "False"
            },
        ))
    }

    fn eval_cases(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("Cases", args, 2));
        }

        let target = self.evaluate(args[0].clone())?;
        let spec = self.normalize_cases_spec(&args[1])?;
        let mut matches = Vec::new();

        self.collect_cases(&target, &spec, &mut matches)?;

        Ok(Expr::call(system_expr("List"), matches))
    }

    fn eval_replace_all(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("ReplaceAll", args, 2));
        }

        let target = self.evaluate(args[0].clone())?;
        let rules_expr = self.evaluate(args[1].clone())?;

        let Some(rules) = normalize_rewrite_rules(&rules_expr) else {
            return Ok(self.invalid_call_with_arg_form(
                "ReplaceAll",
                &[target, rules_expr],
                "expected a rule or list of rules as the second argument",
            ));
        };

        let replaced = self.apply_replace_all(target, &rules, RuleApplicationSource::ReplaceAll)?;
        self.evaluate(replaced)
    }

    fn eval_replace_repeated(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("ReplaceRepeated", args, 2));
        }

        let mut current = self.evaluate(args[0].clone())?;
        let rules_expr = self.evaluate(args[1].clone())?;

        let Some(rules) = normalize_rewrite_rules(&rules_expr) else {
            return Ok(self.invalid_call_with_arg_form(
                "ReplaceRepeated",
                &[current, rules_expr],
                "expected a rule or list of rules as the second argument",
            ));
        };

        for _ in 0..self.resource_limits.replace_repeated_limit {
            let replaced = self.apply_replace_all(
                current.clone(),
                &rules,
                RuleApplicationSource::ReplaceRepeated,
            )?;
            let next = self.evaluate(replaced)?;

            if next == current {
                return Ok(next);
            }

            current = next;
        }

        Err(EvalError::IterationLimitExceeded {
            limit: self.resource_limits.replace_repeated_limit,
        })
    }

    fn try_apply_down_values(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Option<Expr>, EvalError> {
        let Some(symbol) = expr_as_symbol(head) else {
            return Ok(None);
        };

        let Some(definition) = self.definitions.get(symbol).cloned() else {
            return Ok(None);
        };

        let candidate = Expr::call(head.clone(), args.to_vec());

        for down_value in definition.down_values {
            let mut bindings = BTreeMap::new();

            if self.match_expr(&down_value.lhs, &candidate, &mut bindings)? {
                let substituted = substitute_rule_bindings(&down_value.rhs, &bindings);
                self.push_trace(TraceEvent::RuleApplied {
                    source: RuleApplicationSource::DownValue,
                    input: candidate.to_string(),
                    rule: down_value.lhs.to_string(),
                    result: substituted.to_string(),
                });
                return self.evaluate(substituted).map(Some);
            }
        }

        Ok(None)
    }

    fn match_expr(
        &mut self,
        pattern: &Expr,
        value: &Expr,
        bindings: &mut BTreeMap<Symbol, Expr>,
    ) -> Result<bool, EvalError> {
        let snapshot = bindings.clone();
        let matched = self.match_expr_inner(pattern, value, bindings)?;

        if !matched {
            *bindings = snapshot;
        }

        Ok(matched)
    }

    fn match_expr_inner(
        &mut self,
        pattern: &Expr,
        value: &Expr,
        bindings: &mut BTreeMap<Symbol, Expr>,
    ) -> Result<bool, EvalError> {
        if let Some((name, inner)) = pattern_named_pattern(pattern) {
            if !self.match_expr(inner, value, bindings)? {
                return Ok(false);
            }

            return Ok(bind_pattern_name(name, value.clone(), bindings));
        }

        if let Some((inner, condition)) = pattern_condition(pattern) {
            if !self.match_expr(inner, value, bindings)? {
                return Ok(false);
            }

            let condition_expr = substitute_bindings(condition, bindings);
            let result = self.evaluate(condition_expr)?;
            return Ok(is_system_symbol_name(&result, "True"));
        }

        if let Some(blank_head) = pattern_blank_head(pattern) {
            return Ok(match_blank(blank_head, value));
        }

        match (pattern, value) {
            (Expr::Atom(lhs), Expr::Atom(rhs)) => Ok(lhs == rhs),
            (
                Expr::Call {
                    head: lhs_head,
                    args: lhs_args,
                },
                Expr::Call {
                    head: rhs_head,
                    args: rhs_args,
                },
            ) => {
                if !self.match_expr(lhs_head, rhs_head, bindings)? {
                    return Ok(false);
                }

                self.match_call_args(lhs_args, rhs_args, bindings)
            }
            _ => Ok(false),
        }
    }

    fn match_call_args(
        &mut self,
        pattern_args: &[Expr],
        value_args: &[Expr],
        bindings: &mut BTreeMap<Symbol, Expr>,
    ) -> Result<bool, EvalError> {
        if pattern_args.is_empty() {
            return Ok(value_args.is_empty());
        }

        if let Some(spec) = decompose_sequence_pattern(&pattern_args[0]) {
            let rest = &pattern_args[1..];
            let min_rest = min_required_arg_count(rest);

            if value_args.len() < spec.min_len + min_rest {
                return Ok(false);
            }

            let max_len = value_args.len() - min_rest;

            for consume in spec.min_len..=max_len {
                let snapshot = bindings.clone();

                if self.match_sequence_pattern(spec, &value_args[..consume], bindings)?
                    && self.match_call_args(rest, &value_args[consume..], bindings)?
                {
                    return Ok(true);
                }

                *bindings = snapshot;
            }

            return Ok(false);
        }

        let Some((value, rest)) = value_args.split_first() else {
            return Ok(false);
        };

        if !self.match_expr(&pattern_args[0], value, bindings)? {
            return Ok(false);
        }

        self.match_call_args(&pattern_args[1..], rest, bindings)
    }

    fn match_sequence_pattern(
        &mut self,
        spec: SequencePatternSpec<'_>,
        values: &[Expr],
        bindings: &mut BTreeMap<Symbol, Expr>,
    ) -> Result<bool, EvalError> {
        if values.len() < spec.min_len {
            return Ok(false);
        }

        if !values
            .iter()
            .all(|value| match_blank(spec.head_constraint, value))
        {
            return Ok(false);
        }

        if let Some(name) = spec.name {
            let bound_value = sequence_expr(values.to_vec());

            if !bind_pattern_name(name, bound_value, bindings) {
                return Ok(false);
            }
        }

        if let Some(condition) = spec.condition {
            let condition_expr = substitute_bindings(condition, bindings);
            let result = self.evaluate(condition_expr)?;
            return Ok(is_system_symbol_name(&result, "True"));
        }

        Ok(true)
    }

    fn normalize_pattern_expr(&mut self, expr: &Expr) -> Result<Expr, EvalError> {
        match expr {
            Expr::Atom(Atom::Symbol(_)) => self.evaluate(expr.clone()),
            _ => Ok(expr.clone()),
        }
    }

    fn normalize_cases_spec(&mut self, expr: &Expr) -> Result<CasesSpec, EvalError> {
        let resolved = match expr {
            Expr::Atom(Atom::Symbol(_)) => self.evaluate(expr.clone())?,
            Expr::Call { head, .. }
                if is_system_symbol_name(head, "Rule")
                    || is_system_symbol_name(head, "RuleDelayed") =>
            {
                self.evaluate(expr.clone())?
            }
            _ => expr.clone(),
        };

        Ok(match normalize_single_rewrite_rule(&resolved) {
            Some(rule) => CasesSpec::Rule(rule),
            None => CasesSpec::Pattern(resolved),
        })
    }

    fn collect_cases(
        &mut self,
        expr: &Expr,
        spec: &CasesSpec,
        matches: &mut Vec<Expr>,
    ) -> Result<(), EvalError> {
        if let Some(case) = self.try_collect_case(expr, spec)? {
            matches.push(case);
        }

        if let Expr::Call { args, .. } = expr {
            for arg in args {
                self.collect_cases(arg, spec, matches)?;
            }
        }

        Ok(())
    }

    fn try_collect_case(
        &mut self,
        expr: &Expr,
        spec: &CasesSpec,
    ) -> Result<Option<Expr>, EvalError> {
        let mut bindings = BTreeMap::new();

        match spec {
            CasesSpec::Pattern(pattern) => {
                if self.match_expr(pattern, expr, &mut bindings)? {
                    Ok(Some(expr.clone()))
                } else {
                    Ok(None)
                }
            }
            CasesSpec::Rule(rule) => {
                if self.match_expr(&rule.lhs, expr, &mut bindings)? {
                    let transformed = substitute_rule_bindings(&rule.rhs, &bindings);
                    self.evaluate(transformed).map(Some)
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn apply_replace_all(
        &mut self,
        expr: Expr,
        rules: &[RewriteRule],
        source: RuleApplicationSource,
    ) -> Result<Expr, EvalError> {
        if let Some(replaced) = self.apply_rules_once(&expr, rules, source)? {
            return Ok(replaced);
        }

        match expr {
            Expr::Atom(_) => Ok(expr),
            Expr::Call { head, args } => Ok(Expr::call(
                self.apply_replace_all(*head, rules, source)?,
                args.into_iter()
                    .map(|arg| self.apply_replace_all(arg, rules, source))
                    .collect::<Result<Vec<_>, _>>()?,
            )),
        }
    }

    fn apply_rules_once(
        &mut self,
        expr: &Expr,
        rules: &[RewriteRule],
        source: RuleApplicationSource,
    ) -> Result<Option<Expr>, EvalError> {
        for rule in rules {
            let mut bindings = BTreeMap::new();

            if self.match_expr(&rule.lhs, expr, &mut bindings)? {
                let replaced = substitute_rule_bindings(&rule.rhs, &bindings);
                self.push_trace(TraceEvent::RuleApplied {
                    source,
                    input: expr.to_string(),
                    rule: rule.lhs.to_string(),
                    result: replaced.to_string(),
                });
                return Ok(Some(replaced));
            }
        }

        Ok(None)
    }

    fn push_trace(&mut self, event: TraceEvent) {
        if self.tracing_enabled {
            self.trace_events.push(event);
        }
    }
}

pub fn lower_syntax_expr(expr: &SyntaxExpr) -> Result<Expr, EvalError> {
    lower_syntax_expr_with(expr, &mut |raw| {
        resolve_unqualified_symbol(raw).or_else(|| Some(symbol_from_raw_name(raw)))
    })
}

fn lower_syntax_expr_with<F>(expr: &SyntaxExpr, resolve_symbol: &mut F) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    match expr {
        SyntaxExpr::Integer(value) => {
            let bigint = BigInt::parse_bytes(value.as_bytes(), 10).ok_or_else(|| {
                EvalError::InvalidInteger {
                    literal: value.clone(),
                }
            })?;

            Ok(Expr::integer(bigint))
        }
        SyntaxExpr::String(value) => Ok(Expr::string(value.clone())),
        SyntaxExpr::Symbol(value) => Ok(lower_symbol(value, resolve_symbol)),
        SyntaxExpr::List(items) => {
            let lowered_items = items
                .iter()
                .map(|item| lower_syntax_expr_with(item, resolve_symbol))
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Expr::call(system_expr("List"), lowered_items))
        }
        SyntaxExpr::Slot(spec) => lower_slot(spec),
        SyntaxExpr::Blank { kind, head } => lower_blank(*kind, head.as_deref(), resolve_symbol),
        SyntaxExpr::Pattern { name, pattern } => Ok(Expr::call(
            system_expr("Pattern"),
            vec![
                lower_syntax_expr_with(name, resolve_symbol)?,
                lower_syntax_expr_with(pattern, resolve_symbol)?,
            ],
        )),
        SyntaxExpr::Prefix { op, expr } => lower_prefix(*op, expr, resolve_symbol),
        SyntaxExpr::Binary { op, lhs, rhs } => lower_binary(*op, lhs, rhs, resolve_symbol),
        SyntaxExpr::Postfix { op, expr } => lower_postfix(*op, expr, resolve_symbol),
        SyntaxExpr::Call { head, args } => {
            let lowered_head = lower_syntax_expr_with(head, resolve_symbol)?;
            let lowered_args = args
                .iter()
                .map(|arg| lower_syntax_expr_with(arg, resolve_symbol))
                .collect::<Result<Vec<_>, _>>()?;

            Ok(Expr::call(lowered_head, lowered_args))
        }
    }
}

fn lower_slot(spec: &SlotSpec) -> Result<Expr, EvalError> {
    let (head, arg) = match spec {
        SlotSpec::Anonymous => ("Slot", Expr::integer(BigInt::from(1_u8))),
        SlotSpec::Number(value) => {
            let bigint = BigInt::parse_bytes(value.as_bytes(), 10).ok_or_else(|| {
                EvalError::InvalidInteger {
                    literal: value.clone(),
                }
            })?;

            ("Slot", Expr::integer(bigint))
        }
        SlotSpec::Named(value) => ("Slot", Expr::string(value.clone())),
        SlotSpec::Sequence => ("SlotSequence", Expr::integer(BigInt::from(1_u8))),
        SlotSpec::SequenceNumber(value) => {
            let bigint = BigInt::parse_bytes(value.as_bytes(), 10).ok_or_else(|| {
                EvalError::InvalidInteger {
                    literal: value.clone(),
                }
            })?;

            ("SlotSequence", Expr::integer(bigint))
        }
    };

    Ok(Expr::call(system_expr(head), vec![arg]))
}

fn lower_blank<F>(
    kind: BlankKind,
    head: Option<&SyntaxExpr>,
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    let head_symbol = match kind {
        BlankKind::Blank => "Blank",
        BlankKind::BlankSequence => "BlankSequence",
        BlankKind::BlankNullSequence => "BlankNullSequence",
    };

    let args = match head {
        Some(head) => vec![lower_syntax_expr_with(head, resolve_symbol)?],
        None => Vec::new(),
    };

    Ok(Expr::call(system_expr(head_symbol), args))
}

fn lower_prefix<F>(
    op: PrefixOp,
    expr: &SyntaxExpr,
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    let lowered = lower_syntax_expr_with(expr, resolve_symbol)?;

    match op {
        PrefixOp::Plus => Ok(lowered),
        PrefixOp::Minus => Ok(Expr::call(
            system_expr("Times"),
            vec![Expr::integer(BigInt::from(-1)), lowered],
        )),
    }
}

fn lower_binary<F>(
    op: BinaryOp,
    lhs: &SyntaxExpr,
    rhs: &SyntaxExpr,
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    let lowered_lhs = lower_syntax_expr_with(lhs, resolve_symbol)?;
    let lowered_rhs = lower_syntax_expr_with(rhs, resolve_symbol)?;

    match op {
        BinaryOp::Set => Ok(Expr::call(
            system_expr("Set"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::SetDelayed => Ok(Expr::call(
            system_expr("SetDelayed"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::ReplaceAll => Ok(Expr::call(
            system_expr("ReplaceAll"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::ReplaceRepeated => Ok(Expr::call(
            system_expr("ReplaceRepeated"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Rule => Ok(Expr::call(
            system_expr("Rule"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::RuleDelayed => Ok(Expr::call(
            system_expr("RuleDelayed"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Condition => Ok(Expr::call(
            system_expr("Condition"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Add => Ok(Expr::call(
            system_expr("Plus"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Subtract => Ok(Expr::call(
            system_expr("Plus"),
            vec![
                lowered_lhs,
                Expr::call(
                    system_expr("Times"),
                    vec![Expr::integer(BigInt::from(-1)), lowered_rhs],
                ),
            ],
        )),
        BinaryOp::Times => Ok(Expr::call(
            system_expr("Times"),
            vec![lowered_lhs, lowered_rhs],
        )),
        BinaryOp::Divide => Ok(Expr::call(
            system_expr("Times"),
            vec![
                lowered_lhs,
                Expr::call(
                    system_expr("Power"),
                    vec![lowered_rhs, Expr::integer(BigInt::from(-1))],
                ),
            ],
        )),
        BinaryOp::Power => Ok(Expr::call(
            system_expr("Power"),
            vec![lowered_lhs, lowered_rhs],
        )),
    }
}

fn lower_postfix<F>(
    op: PostfixOp,
    expr: &SyntaxExpr,
    resolve_symbol: &mut F,
) -> Result<Expr, EvalError>
where
    F: FnMut(&str) -> Option<Symbol>,
{
    let lowered = lower_syntax_expr_with(expr, resolve_symbol)?;

    match op {
        PostfixOp::Function => Ok(Expr::call(system_expr("Function"), vec![lowered])),
    }
}

fn lower_symbol<F>(raw: &str, resolve_symbol: &mut F) -> Expr
where
    F: FnMut(&str) -> Option<Symbol>,
{
    Expr::Atom(Atom::Symbol(
        resolve_symbol(raw).unwrap_or_else(|| symbol_from_raw_name(raw)),
    ))
}

fn symbol_from_raw_name(raw: &str) -> Symbol {
    if let Some((context, name)) = split_symbol_name(raw) {
        Symbol::new(context, name)
    } else {
        Symbol::from_name(raw)
    }
}

fn split_symbol_name(raw: &str) -> Option<(&str, &str)> {
    raw.rfind('`')
        .map(|split_at| (&raw[..=split_at], &raw[split_at + 1..]))
}

fn normalize_context_name(context: impl Into<String>) -> String {
    let mut context = context.into();

    if !context.ends_with('`') {
        context.push('`');
    }

    context
}

fn expand_context_spec(spec: &str, current_context: &str) -> String {
    if let Some(suffix) = spec.strip_prefix('`') {
        normalize_context_name(format!("{current_context}{suffix}"))
    } else {
        normalize_context_name(spec)
    }
}

fn prepend_context(context: &str, path: &[String]) -> Vec<String> {
    let mut result = Vec::with_capacity(path.len() + 1);
    result.push(normalize_context_name(context));

    for existing in path {
        if existing != context {
            result.push(existing.clone());
        }
    }

    result
}

fn dedup_paths(paths: Vec<PathBuf>) -> Vec<PathBuf> {
    let mut deduped = Vec::new();

    for path in paths {
        if !deduped.iter().any(|existing| existing == &path) {
            deduped.push(path);
        }
    }

    deduped
}

fn normalize_found_path(path: &Path) -> PathBuf {
    let canonical = fs::canonicalize(path).unwrap_or_else(|_| {
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map(|cwd| cwd.join(path))
                .unwrap_or_else(|_| path.to_path_buf())
        }
    });

    strip_windows_verbatim_prefix(canonical)
}

fn strip_windows_verbatim_prefix(path: PathBuf) -> PathBuf {
    let rendered = path.display().to_string();

    rendered
        .strip_prefix("\\\\?\\")
        .map(PathBuf::from)
        .unwrap_or(path)
}

fn package_candidates_for_context(context: &str) -> Vec<PathBuf> {
    let trimmed = context.trim_end_matches('`');
    let segments = trimmed
        .split('`')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    let Some((leaf, parents)) = segments.split_last() else {
        return Vec::new();
    };

    package_candidates_for_segments(parents, leaf)
}

fn package_candidates_for_segments(parents: &[&str], leaf: &str) -> Vec<PathBuf> {
    let base_dir = parents.iter().fold(PathBuf::new(), |mut path, segment| {
        path.push(segment);
        path
    });
    let mut candidates = Vec::new();

    for extension in ["rum", "wl", "m"] {
        candidates.push(base_dir.join(format!("{leaf}.{extension}")));
        candidates.push(
            base_dir
                .join(leaf)
                .join("Kernel")
                .join(format!("init.{extension}")),
        );
        candidates.push(base_dir.join(leaf).join(format!("init.{extension}")));
    }

    candidates
}

fn split_script_into_forms(source: &str) -> Result<Vec<String>, EvalError> {
    let mut forms = Vec::new();
    let mut current = String::new();

    for raw_line in source.lines() {
        if current.is_empty() && raw_line.trim().is_empty() {
            continue;
        }

        if !current.is_empty() {
            current.push('\n');
        }

        current.push_str(raw_line);

        if current.trim().is_empty() {
            current.clear();
            continue;
        }

        if parse_expr(&current).is_ok() {
            forms.push(current.trim().to_string());
            current.clear();
        }
    }

    if !current.trim().is_empty() {
        parse_expr(&current)?;
        forms.push(current.trim().to_string());
    }

    Ok(forms)
}

fn resolve_unqualified_symbol(raw: &str) -> Option<Symbol> {
    split_symbol_name(raw)
        .map(|(context, name)| Symbol::new(normalize_context_name(context), name))
        .or_else(|| is_known_system_symbol_name(raw).then(|| Symbol::new(SYSTEM_CONTEXT, raw)))
}

fn system_expr(name: &str) -> Expr {
    Expr::system_symbol(name)
}

fn expr_as_symbol(expr: &Expr) -> Option<&Symbol> {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => Some(symbol),
        _ => None,
    }
}

fn load_spec_from_expr(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Atom(Atom::String(spec)) => Some(spec.clone()),
        Expr::Call { head, args } if is_system_symbol_name(head, "File") && args.len() == 1 => {
            match &args[0] {
                Expr::Atom(Atom::String(spec)) => Some(spec.clone()),
                _ => None,
            }
        }
        _ => None,
    }
}

fn expr_as_string_list(expr: &Expr) -> Option<Vec<String>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "List") {
        return None;
    }

    args.iter()
        .map(|arg| match arg {
            Expr::Atom(Atom::String(value)) => Some(value.clone()),
            _ => None,
        })
        .collect()
}

fn truthiness_symbol(expr: &Expr) -> Option<bool> {
    match expr_as_symbol(expr) {
        Some(symbol) if symbol.context() == SYSTEM_CONTEXT && symbol.name() == "True" => Some(true),
        Some(symbol) if symbol.context() == SYSTEM_CONTEXT && symbol.name() == "False" => {
            Some(false)
        }
        _ => None,
    }
}

fn fallback_hold_attribute(name: &str) -> HoldAttribute {
    match name {
        "Hold" | "HoldAll" | "Evaluate" => HoldAttribute::HoldAll,
        "Function" | "Which" | "Module" | "Block" | "Check" | "Quiet" => HoldAttribute::HoldAll,
        "Context" => HoldAttribute::HoldFirst,
        "If" => HoldAttribute::HoldRest,
        "With" => HoldAttribute::HoldAll,
        "HoldFirst" => HoldAttribute::HoldFirst,
        "HoldRest" => HoldAttribute::HoldRest,
        _ => HoldAttribute::None,
    }
}

fn evaluate_wrapper_arg(expr: &Expr) -> Option<&Expr> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Evaluate") && args.len() == 1 => {
            Some(&args[0])
        }
        _ => None,
    }
}

fn splice_sequence_argument(args: &mut Vec<Expr>, expr: Expr) {
    if let Some(items) = sequence_items(&expr) {
        args.extend(items.iter().cloned());
    } else {
        args.push(expr);
    }
}

enum FunctionSpec<'a> {
    SlotBased(&'a Expr),
    Explicit { params: Vec<Symbol>, body: &'a Expr },
}

fn parse_function(expr: &Expr) -> Option<FunctionSpec<'_>> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Function") && args.len() == 1 => {
            Some(FunctionSpec::SlotBased(&args[0]))
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Function") && args.len() >= 2 => {
            Some(FunctionSpec::Explicit {
                params: parse_function_parameters(&args[0])?,
                body: &args[1],
            })
        }
        _ => None,
    }
}

fn parse_function_parameters(expr: &Expr) -> Option<Vec<Symbol>> {
    if let Some(symbol) = expr_as_symbol(expr) {
        return Some(vec![symbol.clone()]);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "List") {
        return None;
    }

    args.iter()
        .map(|arg| expr_as_symbol(arg).cloned())
        .collect::<Option<Vec<_>>>()
}

fn parse_with_binding_item(expr: &Expr) -> Option<(Symbol, Expr)> {
    match expr {
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Set") || is_system_symbol_name(head, "Rule"))
                && args.len() == 2 =>
        {
            Some((expr_as_symbol(&args[0])?.clone(), args[1].clone()))
        }
        _ => None,
    }
}

fn parse_module_binding_list(expr: &Expr) -> Option<Vec<(Symbol, Option<Expr>)>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "List") {
        return None;
    }

    args.iter().map(parse_module_binding_item).collect()
}

fn parse_module_binding_item(expr: &Expr) -> Option<(Symbol, Option<Expr>)> {
    if let Some(symbol) = expr_as_symbol(expr) {
        return Some((symbol.clone(), None));
    }

    match expr {
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Set") || is_system_symbol_name(head, "Rule"))
                && args.len() == 2 =>
        {
            Some((expr_as_symbol(&args[0])?.clone(), Some(args[1].clone())))
        }
        _ => None,
    }
}

fn collect_with_binding_symbols(expr: &Expr, locals: &mut BTreeSet<Symbol>) {
    let Expr::Call { head, args } = expr else {
        return;
    };

    if !is_system_symbol_name(head, "List") {
        return;
    }

    for item in args {
        if let Some((symbol, _)) = parse_module_binding_item(item) {
            locals.insert(symbol);
        }
    }
}

fn downvalue_head_symbol(expr: &Expr) -> Option<&Symbol> {
    match expr {
        Expr::Call { head, args }
            if is_system_symbol_name(head, "Condition") && !args.is_empty() =>
        {
            downvalue_head_symbol(&args[0])
        }
        Expr::Call { head, .. } => expr_as_symbol(head),
        _ => None,
    }
}

fn normalize_rewrite_rules(expr: &Expr) -> Option<Vec<RewriteRule>> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "List") => {
            let mut rules = Vec::new();

            for item in args {
                let rule = normalize_single_rewrite_rule(item)?;
                rules.push(rule);
            }

            Some(rules)
        }
        _ => normalize_single_rewrite_rule(expr).map(|rule| vec![rule]),
    }
}

fn normalize_single_rewrite_rule(expr: &Expr) -> Option<RewriteRule> {
    match expr {
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Rule")
                || is_system_symbol_name(head, "RuleDelayed"))
                && args.len() == 2 =>
        {
            Some(RewriteRule {
                lhs: args[0].clone(),
                rhs: args[1].clone(),
            })
        }
        _ => None,
    }
}

fn pattern_named_pattern(expr: &Expr) -> Option<(&Symbol, &Expr)> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Pattern") && args.len() == 2 => {
            let name = expr_as_symbol(&args[0])?;
            Some((name, &args[1]))
        }
        _ => None,
    }
}

fn pattern_condition(expr: &Expr) -> Option<(&Expr, &Expr)> {
    match expr {
        Expr::Call { head, args }
            if is_system_symbol_name(head, "Condition") && args.len() == 2 =>
        {
            Some((&args[0], &args[1]))
        }
        _ => None,
    }
}

fn decompose_sequence_pattern(expr: &Expr) -> Option<SequencePatternSpec<'_>> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Pattern") && args.len() == 2 => {
            let mut spec = decompose_sequence_pattern(&args[1])?;
            let name = expr_as_symbol(&args[0])?;

            if spec.name.is_some() {
                return None;
            }

            spec.name = Some(name);
            Some(spec)
        }
        Expr::Call { head, args }
            if is_system_symbol_name(head, "Condition") && args.len() == 2 =>
        {
            let mut spec = decompose_sequence_pattern(&args[0])?;

            if spec.condition.is_some() {
                return None;
            }

            spec.condition = Some(&args[1]);
            Some(spec)
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "BlankSequence") => {
            if args.len() > 1 {
                return None;
            }

            Some(SequencePatternSpec {
                min_len: 1,
                head_constraint: args.first(),
                name: None,
                condition: None,
            })
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "BlankNullSequence") => {
            if args.len() > 1 {
                return None;
            }

            Some(SequencePatternSpec {
                min_len: 0,
                head_constraint: args.first(),
                name: None,
                condition: None,
            })
        }
        _ => None,
    }
}

fn min_required_arg_count(patterns: &[Expr]) -> usize {
    patterns
        .iter()
        .map(|pattern| {
            decompose_sequence_pattern(pattern)
                .map(|spec| spec.min_len)
                .unwrap_or(1)
        })
        .sum()
}

fn pattern_blank_head(expr: &Expr) -> Option<Option<&Expr>> {
    match expr {
        Expr::Call { head, args }
            if is_system_symbol_name(head, "Blank")
                || is_system_symbol_name(head, "BlankSequence")
                || is_system_symbol_name(head, "BlankNullSequence") =>
        {
            if args.is_empty() {
                Some(None)
            } else if args.len() == 1 {
                Some(Some(&args[0]))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn match_blank(head_constraint: Option<&Expr>, value: &Expr) -> bool {
    match head_constraint {
        None => true,
        Some(head_constraint) => expr_has_head(value, head_constraint),
    }
}

fn expr_has_head(value: &Expr, head_constraint: &Expr) -> bool {
    let Some(expected_symbol) = expr_as_symbol(head_constraint) else {
        return false;
    };

    match value {
        Expr::Atom(Atom::Integer(_)) => expected_symbol.name() == "Integer",
        Expr::Atom(Atom::String(_)) => expected_symbol.name() == "String",
        Expr::Atom(Atom::Symbol(_)) => expected_symbol.name() == "Symbol",
        Expr::Call { head, .. } => expr_as_symbol(head)
            .map(|actual| actual == expected_symbol)
            .unwrap_or(false),
    }
}

fn bind_pattern_name(name: &Symbol, value: Expr, bindings: &mut BTreeMap<Symbol, Expr>) -> bool {
    match bindings.get(name) {
        Some(existing) => existing == &value,
        None => {
            bindings.insert(name.clone(), value);
            true
        }
    }
}

fn sequence_expr(values: Vec<Expr>) -> Expr {
    Expr::call(system_expr("Sequence"), values)
}

fn substitute_bindings(expr: &Expr, bindings: &BTreeMap<Symbol, Expr>) -> Expr {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => bindings
            .get(symbol)
            .cloned()
            .unwrap_or_else(|| Expr::Atom(Atom::Symbol(symbol.clone()))),
        Expr::Atom(atom) => Expr::Atom(atom.clone()),
        Expr::Call { head, args } => Expr::call(
            substitute_bindings(head, bindings),
            args.iter()
                .map(|arg| substitute_bindings(arg, bindings))
                .collect(),
        ),
    }
}

fn substitute_rule_bindings(expr: &Expr, bindings: &BTreeMap<Symbol, Expr>) -> Expr {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => bindings
            .get(symbol)
            .cloned()
            .unwrap_or_else(|| Expr::Atom(Atom::Symbol(symbol.clone()))),
        Expr::Atom(atom) => Expr::Atom(atom.clone()),
        Expr::Call { head, args } => {
            let substituted_head = substitute_rule_bindings(head, bindings);
            let mut substituted_args = Vec::new();

            for arg in args {
                let substituted_arg = substitute_rule_bindings(arg, bindings);

                if let Some(sequence_items) = sequence_items(&substituted_arg) {
                    substituted_args.extend(sequence_items.iter().cloned());
                } else {
                    substituted_args.push(substituted_arg);
                }
            }

            Expr::call(substituted_head, substituted_args)
        }
    }
}

fn substitute_lexical_bindings(
    expr: &Expr,
    bindings: &BTreeMap<Symbol, Expr>,
    shadowed: &BTreeSet<Symbol>,
) -> Expr {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => {
            if shadowed.contains(symbol) {
                Expr::Atom(Atom::Symbol(symbol.clone()))
            } else {
                bindings
                    .get(symbol)
                    .cloned()
                    .unwrap_or_else(|| Expr::Atom(Atom::Symbol(symbol.clone())))
            }
        }
        Expr::Atom(atom) => Expr::Atom(atom.clone()),
        Expr::Call { head, args } if is_system_symbol_name(head, "Pattern") && args.len() == 2 => {
            Expr::call(
                substitute_lexical_bindings(head, bindings, shadowed),
                vec![
                    args[0].clone(),
                    substitute_lexical_bindings(&args[1], bindings, shadowed),
                ],
            )
        }
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Rule")
                || is_system_symbol_name(head, "RuleDelayed")
                || is_system_symbol_name(head, "SetDelayed"))
                && args.len() == 2 =>
        {
            let substituted_head = substitute_lexical_bindings(head, bindings, shadowed);
            let substituted_lhs = substitute_lexical_bindings(&args[0], bindings, shadowed);
            let mut rhs_shadowed = shadowed.clone();
            collect_pattern_locals(&substituted_lhs, &mut rhs_shadowed);

            Expr::call(
                substituted_head,
                vec![
                    substituted_lhs,
                    substitute_lexical_bindings(&args[1], bindings, &rhs_shadowed),
                ],
            )
        }
        Expr::Call { head, args }
            if is_system_symbol_name(head, "Condition") && args.len() == 2 =>
        {
            let substituted_head = substitute_lexical_bindings(head, bindings, shadowed);
            let substituted_lhs = substitute_lexical_bindings(&args[0], bindings, shadowed);
            let mut condition_shadowed = shadowed.clone();
            collect_pattern_locals(&substituted_lhs, &mut condition_shadowed);

            Expr::call(
                substituted_head,
                vec![
                    substituted_lhs,
                    substitute_lexical_bindings(&args[1], bindings, &condition_shadowed),
                ],
            )
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "With") && args.len() == 2 => {
            let substituted_head = substitute_lexical_bindings(head, bindings, shadowed);
            let substituted_bindings = substitute_with_binding_list(&args[0], bindings, shadowed);
            let mut body_shadowed = shadowed.clone();
            collect_with_binding_symbols(&args[0], &mut body_shadowed);

            Expr::call(
                substituted_head,
                vec![
                    substituted_bindings,
                    substitute_lexical_bindings(&args[1], bindings, &body_shadowed),
                ],
            )
        }
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Module") || is_system_symbol_name(head, "Block"))
                && !args.is_empty() =>
        {
            let substituted_head = substitute_lexical_bindings(head, bindings, shadowed);
            let substituted_bindings = substitute_with_binding_list(&args[0], bindings, shadowed);
            let mut body_shadowed = shadowed.clone();
            collect_with_binding_symbols(&args[0], &mut body_shadowed);
            let mut substituted_args = vec![substituted_bindings];

            substituted_args.extend(
                args[1..]
                    .iter()
                    .map(|arg| substitute_lexical_bindings(arg, bindings, &body_shadowed)),
            );

            Expr::call(substituted_head, substituted_args)
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Function") && args.len() >= 2 => {
            let substituted_head = substitute_lexical_bindings(head, bindings, shadowed);
            let Some(params) = parse_function_parameters(&args[0]) else {
                return Expr::call(
                    substituted_head,
                    args.iter()
                        .map(|arg| substitute_lexical_bindings(arg, bindings, shadowed))
                        .collect(),
                );
            };

            let mut body_shadowed = shadowed.clone();
            body_shadowed.extend(params);

            let mut substituted_args = vec![args[0].clone()];
            substituted_args.push(substitute_lexical_bindings(
                &args[1],
                bindings,
                &body_shadowed,
            ));
            substituted_args.extend(
                args[2..]
                    .iter()
                    .map(|arg| substitute_lexical_bindings(arg, bindings, shadowed)),
            );

            Expr::call(substituted_head, substituted_args)
        }
        Expr::Call { head, args } => Expr::call(
            substitute_lexical_bindings(head, bindings, shadowed),
            args.iter()
                .map(|arg| substitute_lexical_bindings(arg, bindings, shadowed))
                .collect(),
        ),
    }
}

fn substitute_with_binding_list(
    expr: &Expr,
    bindings: &BTreeMap<Symbol, Expr>,
    shadowed: &BTreeSet<Symbol>,
) -> Expr {
    let Expr::Call { head, args } = expr else {
        return substitute_lexical_bindings(expr, bindings, shadowed);
    };

    if !is_system_symbol_name(head, "List") {
        return substitute_lexical_bindings(expr, bindings, shadowed);
    }

    Expr::call(
        substitute_lexical_bindings(head, bindings, shadowed),
        args.iter()
            .map(|item| substitute_with_binding_item(item, bindings, shadowed))
            .collect(),
    )
}

fn substitute_with_binding_item(
    expr: &Expr,
    bindings: &BTreeMap<Symbol, Expr>,
    shadowed: &BTreeSet<Symbol>,
) -> Expr {
    match expr {
        Expr::Atom(Atom::Symbol(_)) => expr.clone(),
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Set") || is_system_symbol_name(head, "Rule"))
                && args.len() == 2
                && expr_as_symbol(&args[0]).is_some() =>
        {
            Expr::call(
                substitute_lexical_bindings(head, bindings, shadowed),
                vec![
                    args[0].clone(),
                    substitute_lexical_bindings(&args[1], bindings, shadowed),
                ],
            )
        }
        _ => substitute_lexical_bindings(expr, bindings, shadowed),
    }
}

fn collect_pattern_locals(expr: &Expr, locals: &mut BTreeSet<Symbol>) {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Pattern") && args.len() == 2 => {
            if let Some(symbol) = expr_as_symbol(&args[0]) {
                locals.insert(symbol.clone());
            }
            collect_pattern_locals(&args[1], locals);
        }
        Expr::Call { args, .. } => {
            for arg in args {
                collect_pattern_locals(arg, locals);
            }
        }
        Expr::Atom(_) => {}
    }
}

fn substitute_slots(expr: &Expr, args: &[Expr]) -> Expr {
    if let Some(substituted) = substitute_single_slot(expr, args) {
        return substituted;
    }

    match expr {
        Expr::Atom(atom) => Expr::Atom(atom.clone()),
        Expr::Call {
            head,
            args: call_args,
        } => {
            let substituted_head = substitute_slots(head, args);
            let mut substituted_args = Vec::new();

            for arg in call_args {
                let substituted_arg = substitute_slots(arg, args);

                if is_slot_sequence_expr(arg) {
                    if let Some(items) = sequence_items(&substituted_arg) {
                        substituted_args.extend(items.iter().cloned());
                    } else {
                        substituted_args.push(substituted_arg);
                    }
                } else {
                    substituted_args.push(substituted_arg);
                }
            }

            Expr::call(substituted_head, substituted_args)
        }
    }
}

fn substitute_single_slot(expr: &Expr, args: &[Expr]) -> Option<Expr> {
    let Expr::Call {
        head,
        args: slot_args,
    } = expr
    else {
        return None;
    };

    if slot_args.len() != 1 {
        return None;
    }

    match (expr_as_symbol(head)?.name(), &slot_args[0]) {
        ("Slot", Expr::Atom(Atom::Integer(index))) => {
            let index = index.to_string().parse::<usize>().ok()?;
            if index == 0 {
                None
            } else {
                args.get(index - 1).cloned()
            }
        }
        ("Slot", Expr::Atom(Atom::String(name))) => lookup_named_slot(args, name),
        ("SlotSequence", Expr::Atom(Atom::Integer(index))) => {
            let index = index.to_string().parse::<usize>().ok()?;
            if index == 0 {
                None
            } else {
                Some(sequence_expr(
                    args.iter().skip(index - 1).cloned().collect(),
                ))
            }
        }
        _ => None,
    }
}

fn lookup_named_slot(args: &[Expr], name: &str) -> Option<Expr> {
    for arg in args {
        let Expr::Call {
            head,
            args: rule_args,
        } = arg
        else {
            continue;
        };

        if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
            || rule_args.len() != 2
        {
            continue;
        }

        let matches_name = match &rule_args[0] {
            Expr::Atom(Atom::Symbol(symbol)) => symbol.name() == name,
            Expr::Atom(Atom::String(value)) => value == name,
            _ => false,
        };

        if matches_name {
            return Some(rule_args[1].clone());
        }
    }

    None
}

fn is_slot_sequence_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Call { head, args } => is_system_symbol_name(head, "SlotSequence") && args.len() == 1,
        _ => false,
    }
}

fn sequence_items(expr: &Expr) -> Option<&[Expr]> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Sequence") => Some(args),
        _ => None,
    }
}

fn is_known_system_symbol(symbol: &Symbol) -> bool {
    symbol.context() == SYSTEM_CONTEXT && is_known_system_symbol_name(symbol.name())
}

fn is_known_system_symbol_name(name: &str) -> bool {
    matches!(
        name,
        "$Context"
            | "$ContextPath"
            | "$Failed"
            | "$Packages"
            | "$Path"
            | "Begin"
            | "BeginPackage"
            | "Blank"
            | "BlankNullSequence"
            | "BlankSequence"
            | "Block"
            | "Cases"
            | "Check"
            | "Condition"
            | "Context"
            | "End"
            | "EndPackage"
            | "Evaluate"
            | "False"
            | "File"
            | "FindFile"
            | "Function"
            | "Get"
            | "Hold"
            | "HoldAll"
            | "HoldFirst"
            | "HoldRest"
            | "Identity"
            | "If"
            | "Integer"
            | "List"
            | "MatchQ"
            | "Module"
            | "Needs"
            | "Null"
            | "Pattern"
            | "Plus"
            | "Power"
            | "ReplaceAll"
            | "ReplaceRepeated"
            | "Rule"
            | "RuleDelayed"
            | "Quiet"
            | "Sequence"
            | "Set"
            | "SetDelayed"
            | "Slot"
            | "SlotSequence"
            | "String"
            | "Symbol"
            | "Times"
            | "True"
            | "Which"
            | "With"
    )
}

fn is_system_symbol_name(expr: &Expr, name: &str) -> bool {
    expr_as_symbol(expr)
        .map(|symbol| symbol.context() == SYSTEM_CONTEXT && symbol.name() == name)
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use super::*;

    struct TempDirGuard {
        path: PathBuf,
    }

    impl TempDirGuard {
        fn new(prefix: &str) -> Self {
            let unique = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos();
            let path = env::temp_dir().join(format!("rum-{prefix}-{unique}"));
            fs::create_dir_all(&path).unwrap();
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }

        fn write(&self, relative: &str, source: &str) -> PathBuf {
            let path = self.path.join(relative);

            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }

            fs::write(&path, source).unwrap();
            path
        }
    }

    impl Drop for TempDirGuard {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    fn rum_path(path: &Path) -> String {
        path.display().to_string().replace('\\', "/")
    }

    #[test]
    fn lowers_big_integer_into_core_expr() {
        let lowered = lower_syntax_expr(&SyntaxExpr::Integer(
            "123456789012345678901234567890".to_string(),
        ))
        .unwrap();

        assert_eq!(lowered.to_string(), "123456789012345678901234567890");
    }

    #[test]
    fn evaluates_identity_call() {
        let mut session = Session::new();
        let outcome = session
            .eval_text("Identity[123456789012345678901234567890]")
            .unwrap();

        assert_eq!(outcome.value.to_string(), "123456789012345678901234567890");
    }

    #[test]
    fn context_variables_and_builtin_inspection_reflect_session_contexts() {
        let mut session = Session::new();
        session.set_current_context("Rules");

        assert_eq!(
            session.eval_text("$Context").unwrap().value.to_string(),
            "\"Rules`\""
        );
        assert_eq!(
            session.eval_text("$ContextPath").unwrap().value.to_string(),
            "List[\"System`\", \"Global`\"]"
        );

        let info = session.inspect("Identity").unwrap();
        assert_eq!(info.qualified_name, "System`Identity");
        assert!(info.is_builtin);
    }

    #[test]
    fn bare_symbols_search_context_path_before_current_context() {
        let mut session = Session::new();

        session.eval_text("Global`x = 10").unwrap();
        session.set_current_context("Rules");
        session.eval_text("Rules`x = 20").unwrap();
        session.eval_text("local = 30").unwrap();

        assert_eq!(session.eval_text("x").unwrap().value.to_string(), "10");

        let info = session.inspect("local").unwrap();
        assert_eq!(info.qualified_name, "Rules`local");
        assert_eq!(
            session.eval_text("Rules`x").unwrap().value.to_string(),
            "20"
        );
    }

    #[test]
    fn context_builtin_reports_session_and_symbol_contexts_without_evaluating_symbols() {
        let mut session = Session::new();

        session.eval_text("x = 1").unwrap();

        assert_eq!(
            session.eval_text("Context[]").unwrap().value.to_string(),
            "\"Global`\""
        );
        assert_eq!(
            session.eval_text("Context[x]").unwrap().value.to_string(),
            "\"Global`\""
        );
        assert_eq!(
            session
                .eval_text("Context[Identity]")
                .unwrap()
                .value
                .to_string(),
            "\"System`\""
        );
        assert_eq!(
            session
                .eval_text("Context[\"Identity\"]")
                .unwrap()
                .value
                .to_string(),
            "\"System`\""
        );
        assert_eq!(
            session
                .eval_text("Context[\"MissingSymbol\"]")
                .unwrap()
                .value
                .to_string(),
            "Context[\"MissingSymbol\"]"
        );
    }

    #[test]
    fn begin_and_end_balance_context_and_context_path() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("Begin[\"Rules`\"]")
                .unwrap()
                .value
                .to_string(),
            "\"Rules`\""
        );
        assert_eq!(
            session.eval_text("$Context").unwrap().value.to_string(),
            "\"Rules`\""
        );
        assert_eq!(
            session.eval_text("$ContextPath").unwrap().value.to_string(),
            "List[\"Rules`\", \"System`\", \"Global`\"]"
        );

        assert_eq!(
            session
                .eval_text("Begin[\"`Private`\"]")
                .unwrap()
                .value
                .to_string(),
            "\"Rules`Private`\""
        );
        assert_eq!(
            session.eval_text("$Context").unwrap().value.to_string(),
            "\"Rules`Private`\""
        );
        assert_eq!(
            session.eval_text("End[]").unwrap().value.to_string(),
            "\"Rules`Private`\""
        );
        assert_eq!(
            session.eval_text("$Context").unwrap().value.to_string(),
            "\"Rules`\""
        );
        assert_eq!(
            session.eval_text("$ContextPath").unwrap().value.to_string(),
            "List[\"Rules`\", \"System`\", \"Global`\"]"
        );
    }

    #[test]
    fn begin_package_and_end_package_restore_outer_context_and_export_package_context() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("BeginPackage[\"Demo`\"]")
                .unwrap()
                .value
                .to_string(),
            "\"Demo`\""
        );
        assert_eq!(
            session.eval_text("$Context").unwrap().value.to_string(),
            "\"Demo`\""
        );
        assert_eq!(
            session.eval_text("$ContextPath").unwrap().value.to_string(),
            "List[\"Demo`\", \"System`\"]"
        );

        session.eval_text("Begin[\"`Private`\"]").unwrap();
        session.eval_text("hidden = 1").unwrap();
        assert_eq!(
            session.eval_text("End[]").unwrap().value.to_string(),
            "\"Demo`Private`\""
        );

        session.eval_text("public = 2").unwrap();

        assert_eq!(
            session.eval_text("EndPackage[]").unwrap().value.to_string(),
            "Null"
        );
        assert_eq!(
            session.eval_text("$Context").unwrap().value.to_string(),
            "\"Global`\""
        );
        assert_eq!(
            session.eval_text("$ContextPath").unwrap().value.to_string(),
            "List[\"Demo`\", \"System`\", \"Global`\"]"
        );
        assert_eq!(session.eval_text("public").unwrap().value.to_string(), "2");
        assert_eq!(
            session.eval_text("hidden").unwrap().value.to_string(),
            "hidden"
        );
    }

    #[test]
    fn get_loads_a_script_file_and_returns_the_last_value() {
        let dir = TempDirGuard::new("get");
        let file = dir.write("script.rum", "x = 1\npair = {\n  x,\n  2\n}\npair");
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text(&format!("Get[\"{}\"]", rum_path(&file)))
                .unwrap()
                .value
                .to_string(),
            "List[1, 2]"
        );
        assert_eq!(session.eval_text("x").unwrap().value.to_string(), "1");
    }

    #[test]
    fn needs_loads_package_once_and_tracks_loaded_packages() {
        let dir = TempDirGuard::new("needs-once");
        dir.write(
            "Demo.rum",
            "BeginPackage[\"Demo`\"]\ntoken = 1\nEndPackage[]",
        );
        let mut session = Session::new();
        session.set_package_search_paths([dir.path().to_path_buf()]);

        assert_eq!(
            session
                .eval_text("Needs[\"Demo`\"]")
                .unwrap()
                .value
                .to_string(),
            "Null"
        );
        assert_eq!(session.eval_text("token").unwrap().value.to_string(), "1");
        assert_eq!(
            session.eval_text("$Packages").unwrap().value.to_string(),
            "List[\"Demo`\"]"
        );

        session.eval_text("Demo`token = 5").unwrap();
        session.eval_text("Needs[\"Demo`\"]").unwrap();

        assert_eq!(
            session.eval_text("Demo`token").unwrap().value.to_string(),
            "5"
        );
    }

    #[test]
    fn findfile_resolves_package_names_and_file_wrappers() {
        let dir = TempDirGuard::new("findfile");
        let file = dir.write(
            "Demo.rum",
            "BeginPackage[\"Demo`\"]\ntoken = 1\nEndPackage[]",
        );
        let mut session = Session::new();
        session.set_package_search_paths([dir.path().to_path_buf()]);

        assert_eq!(
            session
                .eval_text("FindFile[\"Demo`\"]")
                .unwrap()
                .value
                .to_string(),
            format!("\"{}\"", file.display())
        );
        assert_eq!(
            session
                .eval_text(&format!("FindFile[File[\"{}\"]]", rum_path(&file)))
                .unwrap()
                .value
                .to_string(),
            format!("\"{}\"", file.display())
        );
        assert_eq!(
            session
                .eval_text("FindFile[\"Missing`\"]")
                .unwrap()
                .value
                .to_string(),
            "$Failed"
        );
    }

    #[test]
    fn missing_findfile_emits_runtime_message() {
        let mut session = Session::new();

        let outcome = session.eval_text("FindFile[\"Missing`\"]").unwrap();

        assert_eq!(outcome.value.to_string(), "$Failed");
        assert!(
            outcome
                .messages
                .iter()
                .any(|message| message.symbol == "FindFile" && message.tag == "notfound")
        );
    }

    #[test]
    fn needs_supports_explicit_file_targets() {
        let dir = TempDirGuard::new("needs-file");
        let file = dir.write(
            "custom-package.rum",
            "BeginPackage[\"Custom`\"]\nflag = 9\nEndPackage[]",
        );
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text(&format!("Needs[\"Custom`\", \"{}\"]", rum_path(&file)))
                .unwrap()
                .value
                .to_string(),
            "Null"
        );
        assert_eq!(session.eval_text("flag").unwrap().value.to_string(), "9");

        session.eval_text("Custom`flag = 12").unwrap();
        session
            .eval_text(&format!(
                "Needs[\"Custom`\", File[\"{}\"]]",
                rum_path(&file)
            ))
            .unwrap();

        assert_eq!(
            session.eval_text("Custom`flag").unwrap().value.to_string(),
            "12"
        );
    }

    #[test]
    fn missing_needs_returns_failed_and_emits_messages() {
        let mut session = Session::new();

        let outcome = session.eval_text("Needs[\"Missing`\"]").unwrap();

        assert_eq!(outcome.value.to_string(), "$Failed");
        assert!(
            outcome
                .messages
                .iter()
                .any(|message| message.symbol == "Get" && message.tag == "notfound")
        );
        assert!(
            outcome
                .messages
                .iter()
                .any(|message| message.symbol == "Needs" && message.tag == "load")
        );
    }

    #[test]
    fn invalid_builtin_argument_shapes_emit_runtime_messages() {
        let mut session = Session::new();

        let with_outcome = session.eval_text("With[x, x]").unwrap();
        assert_eq!(with_outcome.value.to_string(), "With[x, x]");
        assert!(
            with_outcome
                .messages
                .iter()
                .any(|message| { message.symbol == "With" && message.tag == "argform" })
        );

        let replace_outcome = session.eval_text("7 /. 1").unwrap();
        assert_eq!(replace_outcome.value.to_string(), "ReplaceAll[7, 1]");
        assert!(
            replace_outcome
                .messages
                .iter()
                .any(|message| { message.symbol == "ReplaceAll" && message.tag == "argform" })
        );

        let get_outcome = session.eval_text("Get[1]").unwrap();
        assert_eq!(get_outcome.value.to_string(), "Get[1]");
        assert!(
            get_outcome
                .messages
                .iter()
                .any(|message| { message.symbol == "Get" && message.tag == "argform" })
        );
    }

    #[test]
    fn invalid_builtin_argument_counts_emit_runtime_messages() {
        let mut session = Session::new();

        let if_outcome = session.eval_text("If[True]").unwrap();
        assert_eq!(if_outcome.value.to_string(), "If[True]");
        assert!(
            if_outcome
                .messages
                .iter()
                .any(|message| { message.symbol == "If" && message.tag == "argtu" })
        );

        let set_outcome = session.eval_text("Set[x]").unwrap();
        assert_eq!(set_outcome.value.to_string(), "Set[x]");
        assert!(
            set_outcome
                .messages
                .iter()
                .any(|message| { message.symbol == "Set" && message.tag == "argx" })
        );
    }

    #[test]
    fn unbalanced_context_frames_emit_runtime_messages() {
        let mut session = Session::new();

        let end_outcome = session.eval_text("End[]").unwrap();
        assert_eq!(end_outcome.value.to_string(), "End[]");
        assert!(
            end_outcome
                .messages
                .iter()
                .any(|message| { message.symbol == "End" && message.tag == "context" })
        );

        let end_package_outcome = session.eval_text("EndPackage[]").unwrap();
        assert_eq!(end_package_outcome.value.to_string(), "EndPackage[]");
        assert!(
            end_package_outcome
                .messages
                .iter()
                .any(|message| { message.symbol == "EndPackage" && message.tag == "context" })
        );
    }

    #[test]
    fn explicit_parameter_function_arity_mismatch_emits_runtime_message() {
        let mut session = Session::new();

        let outcome = session.eval_text("Function[{x, y}, x][1]").unwrap();

        assert_eq!(outcome.value.to_string(), "Function[List[x, y], x][1]");
        assert!(
            outcome
                .messages
                .iter()
                .any(|message| { message.symbol == "Function" && message.tag == "argx" })
        );
    }

    #[test]
    fn check_returns_fallback_when_messages_are_emitted() {
        let mut session = Session::new();

        let outcome = session
            .eval_text("Check[FindFile[\"Missing`\"], 99]")
            .unwrap();

        assert_eq!(outcome.value.to_string(), "99");
        assert!(
            outcome
                .messages
                .iter()
                .any(|message| { message.symbol == "FindFile" && message.tag == "notfound" })
        );
    }

    #[test]
    fn check_leaves_value_unchanged_when_no_messages_are_emitted() {
        let mut session = Session::new();

        let outcome = session.eval_text("Check[Identity[7], 99]").unwrap();

        assert_eq!(outcome.value.to_string(), "7");
        assert!(outcome.messages.is_empty());
    }

    #[test]
    fn quiet_suppresses_runtime_messages_from_nested_evaluation() {
        let mut session = Session::new();

        let outcome = session.eval_text("Quiet[FindFile[\"Missing`\"]]").unwrap();

        assert_eq!(outcome.value.to_string(), "$Failed");
        assert!(outcome.messages.is_empty());
    }

    #[test]
    fn quiet_prevents_check_from_observing_suppressed_messages() {
        let mut session = Session::new();

        let outcome = session
            .eval_text("Check[Quiet[FindFile[\"Missing`\"]], 99]")
            .unwrap();

        assert_eq!(outcome.value.to_string(), "$Failed");
        assert!(outcome.messages.is_empty());
    }

    #[test]
    fn beginpackage_can_load_dependencies_via_needs() {
        let dir = TempDirGuard::new("needs-deps");
        dir.write(
            "Dep.rum",
            "BeginPackage[\"Dep`\"]\ndepValue = 7\nEndPackage[]",
        );
        dir.write(
            "Main.rum",
            "BeginPackage[\"Main`\", {\"Dep`\"}]\nvalue = Dep`depValue\nEndPackage[]",
        );
        let mut session = Session::new();
        session.set_package_search_paths([dir.path().to_path_buf()]);

        session.eval_text("Needs[\"Main`\"]").unwrap();

        assert_eq!(session.eval_text("value").unwrap().value.to_string(), "7");
        assert_eq!(
            session.eval_text("$Packages").unwrap().value.to_string(),
            "List[\"Main`\", \"Dep`\"]"
        );
    }

    #[test]
    fn path_builtin_reflects_package_search_roots() {
        let dir = TempDirGuard::new("path");
        let mut session = Session::new();
        session.set_package_search_paths([dir.path().to_path_buf()]);

        assert_eq!(
            session.eval_text("$Path").unwrap().value.to_string(),
            format!("List[\"{}\"]", dir.path().display())
        );
    }

    #[test]
    fn explicit_non_system_heads_do_not_trigger_builtin_dispatch() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("Foo`If[True, Identity[1], Identity[2]]")
                .unwrap()
                .value
                .to_string(),
            "Foo`If[True, 1, 2]"
        );
    }

    #[test]
    fn hold_family_respects_builtin_hold_attributes() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("Hold[Identity[1], Identity[2]]")
                .unwrap()
                .value
                .to_string(),
            "Hold[Identity[1], Identity[2]]"
        );
        assert_eq!(
            session
                .eval_text("HoldFirst[Identity[1], Identity[2]]")
                .unwrap()
                .value
                .to_string(),
            "HoldFirst[Identity[1], 2]"
        );
        assert_eq!(
            session
                .eval_text("HoldRest[Identity[1], Identity[2]]")
                .unwrap()
                .value
                .to_string(),
            "HoldRest[1, Identity[2]]"
        );
        assert_eq!(
            session
                .eval_text("HoldAll[Identity[1], Identity[2]]")
                .unwrap()
                .value
                .to_string(),
            "HoldAll[Identity[1], Identity[2]]"
        );
    }

    #[test]
    fn evaluate_forces_evaluation_inside_held_arguments() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("Evaluate[Identity[1]]")
                .unwrap()
                .value
                .to_string(),
            "1"
        );
        assert_eq!(
            session
                .eval_text("Hold[Evaluate[Identity[1]]]")
                .unwrap()
                .value
                .to_string(),
            "Hold[1]"
        );
    }

    #[test]
    fn sequence_splices_in_non_held_calls_but_not_in_raw_held_ones() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("f[Sequence[1, 2], 3]")
                .unwrap()
                .value
                .to_string(),
            "f[1, 2, 3]"
        );
        assert_eq!(
            session
                .eval_text("Sequence[1, Sequence[2, 3]]")
                .unwrap()
                .value
                .to_string(),
            "Sequence[1, 2, 3]"
        );
        assert_eq!(
            session
                .eval_text("Hold[Sequence[1, 2], 3]")
                .unwrap()
                .value
                .to_string(),
            "Hold[Sequence[1, 2], 3]"
        );
        assert_eq!(
            session
                .eval_text("Hold[Evaluate[Sequence[1, 2]], 3]")
                .unwrap()
                .value
                .to_string(),
            "Hold[1, 2, 3]"
        );
    }

    #[test]
    fn if_evaluates_only_the_selected_branch() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("If[True, Identity[1], x]")
                .unwrap()
                .value
                .to_string(),
            "1"
        );
        assert_eq!(
            session
                .eval_text("If[False, x, Identity[2]]")
                .unwrap()
                .value
                .to_string(),
            "2"
        );
        assert_eq!(
            session.eval_text("If[False, x]").unwrap().value.to_string(),
            "Null"
        );
        assert_eq!(
            session
                .eval_text("If[maybe, Identity[1], Identity[2]]")
                .unwrap()
                .value
                .to_string(),
            "If[maybe, Identity[1], Identity[2]]"
        );
        assert_eq!(
            session
                .eval_text("If[maybe, x, y, Identity[3]]")
                .unwrap()
                .value
                .to_string(),
            "3"
        );
    }

    #[test]
    fn which_evaluates_conditions_in_order() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("Which[False, x, True, Identity[2], True, Identity[3]]")
                .unwrap()
                .value
                .to_string(),
            "2"
        );
        assert_eq!(
            session
                .eval_text("Which[False, x, False, y]")
                .unwrap()
                .value
                .to_string(),
            "Null"
        );
        assert_eq!(
            session
                .eval_text("Which[maybe, Identity[1], True, Identity[2]]")
                .unwrap()
                .value
                .to_string(),
            "Which[maybe, Identity[1], True, Identity[2]]"
        );
    }

    #[test]
    fn function_applies_slots_positionally() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("Function[Slot[1]][Identity[1]]")
                .unwrap()
                .value
                .to_string(),
            "1"
        );
        assert_eq!(
            session
                .eval_text("Function[Plus[Slot[1], Slot[2]]][1, 2]")
                .unwrap()
                .value
                .to_string(),
            "Plus[1, 2]"
        );
        assert_eq!(
            session
                .eval_text("Function[x, x][Identity[1]]")
                .unwrap()
                .value
                .to_string(),
            "1"
        );
        assert_eq!(
            session
                .eval_text("Function[{x, y}, Hold[x, y]][Identity[1], Identity[2]]")
                .unwrap()
                .value
                .to_string(),
            "Hold[1, 2]"
        );
    }

    #[test]
    fn pure_function_syntax_uses_slots_and_evaluated_arguments() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("(#1 &)[Identity[1]]")
                .unwrap()
                .value
                .to_string(),
            "1"
        );
        assert_eq!(
            session
                .eval_text("(#2 &)[10, Identity[2]]")
                .unwrap()
                .value
                .to_string(),
            "2"
        );
        assert_eq!(
            session
                .eval_text("(f[##] &)[1, Identity[2], 3]")
                .unwrap()
                .value
                .to_string(),
            "f[1, 2, 3]"
        );
        assert_eq!(
            session
                .eval_text("(f[##2] &)[10, Identity[20], 30]")
                .unwrap()
                .value
                .to_string(),
            "f[20, 30]"
        );
        assert_eq!(
            session
                .eval_text("(Hold[##] &)[1, 2]")
                .unwrap()
                .value
                .to_string(),
            "Hold[1, 2]"
        );
        assert_eq!(
            session
                .eval_text("(f[#x, #y] &)[x -> 1, y -> Identity[2]]")
                .unwrap()
                .value
                .to_string(),
            "f[1, 2]"
        );
        assert_eq!(
            session
                .eval_text("(f[#x, #1] &)[10, x -> 20]")
                .unwrap()
                .value
                .to_string(),
            "f[20, 10]"
        );
        assert_eq!(
            session
                .eval_text("(Hold[#x] &)[\"x\" -> Identity[3]]")
                .unwrap()
                .value
                .to_string(),
            "Hold[3]"
        );
    }

    #[test]
    fn explicit_parameter_functions_respect_shadowing_and_pattern_locals() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("Function[x, Function[x, x]][1]")
                .unwrap()
                .value
                .to_string(),
            "Function[x, x]"
        );
        assert_eq!(
            session
                .eval_text("Function[x, 7 /. x_Integer :> x][1]")
                .unwrap()
                .value
                .to_string(),
            "7"
        );
        assert_eq!(
            session
                .eval_text("Function[{x, y}, y][1, 2]")
                .unwrap()
                .value
                .to_string(),
            "2"
        );
    }

    #[test]
    fn head_evaluation_can_activate_special_forms_and_held_builtins() {
        let mut session = Session::new();

        session.eval_text("assign = Set").unwrap();
        assert_eq!(
            session
                .eval_text("assign[x, Identity[1]]")
                .unwrap()
                .value
                .to_string(),
            "1"
        );
        assert_eq!(session.eval_text("x").unwrap().value.to_string(), "1");

        session.eval_text("repl = ReplaceAll").unwrap();
        assert_eq!(
            session
                .eval_text("repl[f[1], f[x_Integer] -> x]")
                .unwrap()
                .value
                .to_string(),
            "1"
        );

        session.eval_text("hold = Hold").unwrap();
        assert_eq!(
            session
                .eval_text("hold[Identity[1]]")
                .unwrap()
                .value
                .to_string(),
            "Hold[Identity[1]]"
        );

        session.eval_text("chooser = If").unwrap();
        assert_eq!(
            session
                .eval_text("chooser[True, Identity[1], x]")
                .unwrap()
                .value
                .to_string(),
            "1"
        );
    }

    #[test]
    fn cycle_detection_stops_self_referential_definitions_and_recovers_after_error() {
        let mut session = Session::new();

        session.eval_text("x := x").unwrap();

        let error = session
            .eval_text("x")
            .expect_err("expected cycle detection error");

        assert!(matches!(
            error,
            EvalError::EvaluationCycleDetected { ref expr } if expr == "x"
        ));
        assert_eq!(session.eval_text("1").unwrap().value.to_string(), "1");
    }

    #[test]
    fn cycle_detection_stops_mutually_recursive_downvalues() {
        let mut session = Session::new();

        session.eval_text("f[x_Integer] := g[x]").unwrap();
        session.eval_text("g[x_Integer] := f[x]").unwrap();

        let error = session
            .eval_text("f[1]")
            .expect_err("expected cycle detection error");

        assert!(matches!(
            error,
            EvalError::EvaluationCycleDetected { ref expr } if expr == "f[1]"
        ));
    }

    #[test]
    fn recursion_limit_stops_expanding_recursive_downvalues() {
        let mut session = Session::new();
        session.set_recursion_limit(32);

        session.eval_text("grow[x_] := grow[f[x]]").unwrap();

        let error = session
            .eval_text("grow[1]")
            .expect_err("expected recursion limit error");

        assert!(matches!(
            error,
            EvalError::RecursionLimitExceeded { limit: 32 }
        ));
    }

    #[test]
    fn fixed_point_evaluation_revisits_expressions_after_side_effects_change_head_meaning() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("f[f = Hold, Identity[1]]")
                .unwrap()
                .value
                .to_string(),
            "Hold[Hold, 1]"
        );
    }

    #[test]
    fn tracing_records_definition_rule_and_evaluation_events() {
        let mut session = Session::new();
        session.enable_tracing(true);

        session.eval_text("f[x_Integer] := x").unwrap();
        session.eval_text("f[1]").unwrap();

        let trace = session.take_trace();

        assert!(trace.iter().any(|event| matches!(
            event,
            TraceEvent::DefinitionStored {
                symbol,
                kind: DefinitionKind::DownValue,
                ..
            } if symbol == "f"
        )));
        assert!(trace.iter().any(|event| matches!(
            event,
            TraceEvent::RuleApplied {
                source: RuleApplicationSource::DownValue,
                input,
                ..
            } if input == "f[1]"
        )));
        assert!(trace.iter().any(|event| matches!(
            event,
            TraceEvent::EvaluationPass { before, after }
                if before == "f[1]" && after == "1"
        )));
        assert!(session.take_trace().is_empty());
    }

    #[test]
    fn configurable_replace_repeated_limit_is_respected() {
        let mut session = Session::new();
        session.set_replace_repeated_limit(8);

        let error = session
            .eval_text("1 //. {1 -> 2, 2 -> 1}")
            .expect_err("expected iteration limit error");

        assert!(matches!(
            error,
            EvalError::IterationLimitExceeded { limit: 8 }
        ));
        assert_eq!(session.resource_limits().replace_repeated_limit, 8);
    }

    #[test]
    fn with_evaluates_bindings_and_substitutes_lexically() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("With[{x = Identity[1], y = x}, y]")
                .unwrap()
                .value
                .to_string(),
            "1"
        );
        assert_eq!(
            session
                .eval_text("With[{x = 1}, Hold[x]]")
                .unwrap()
                .value
                .to_string(),
            "Hold[1]"
        );
    }

    #[test]
    fn with_supports_rule_style_bindings_and_nested_shadowing() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("With[{x -> 1}, x]")
                .unwrap()
                .value
                .to_string(),
            "1"
        );
        assert_eq!(
            session
                .eval_text("With[{x = 1}, With[{x = 2}, x]]")
                .unwrap()
                .value
                .to_string(),
            "2"
        );
    }

    #[test]
    fn with_respects_pattern_local_names_but_substitutes_pattern_metadata() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("With[{x = 1}, 7 /. x_Integer :> x]")
                .unwrap()
                .value
                .to_string(),
            "7"
        );
        assert_eq!(
            session
                .eval_text("With[{t = Integer}, MatchQ[7, x_t]]")
                .unwrap()
                .value
                .to_string(),
            "True"
        );
    }

    #[test]
    fn module_generates_fresh_lexical_symbols_and_initializes_sequentially() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("Module[{x = 1, y = x}, y]")
                .unwrap()
                .value
                .to_string(),
            "1"
        );

        let first = session.eval_text("Module[{x}, Hold[x]]").unwrap().value;
        let second = session.eval_text("Module[{x}, Hold[x]]").unwrap().value;

        let first_text = first.to_string();
        let second_text = second.to_string();

        assert!(first_text.starts_with("Hold[x$"));
        assert!(second_text.starts_with("Hold[x$"));
        assert_ne!(first_text, second_text);
    }

    #[test]
    fn module_respects_nested_shadowing_and_pattern_locals() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("Module[{x = 1}, Module[{x = 2}, x]]")
                .unwrap()
                .value
                .to_string(),
            "2"
        );
        let nested = session
            .eval_text("Module[{x = 1}, Module[{x}, Hold[x]]]")
            .unwrap()
            .value
            .to_string();
        assert!(nested.starts_with("Hold[x$"));
        assert_ne!(nested, "Hold[x]");
        assert_ne!(nested, "Hold[1]");
        assert_eq!(
            session
                .eval_text("Module[{x = 1}, 7 /. x_Integer :> x]")
                .unwrap()
                .value
                .to_string(),
            "7"
        );
    }

    #[test]
    fn block_temporarily_rebinds_and_restores_symbol_values() {
        let mut session = Session::new();

        session.eval_text("x = 10").unwrap();

        assert_eq!(
            session
                .eval_text("Block[{x = 1}, x]")
                .unwrap()
                .value
                .to_string(),
            "1"
        );
        assert_eq!(session.eval_text("x").unwrap().value.to_string(), "10");
        assert_eq!(
            session
                .eval_text("Block[{x}, x]")
                .unwrap()
                .value
                .to_string(),
            "10"
        );
        assert_eq!(
            session
                .eval_text("Block[{x}, Hold[x]]")
                .unwrap()
                .value
                .to_string(),
            "Hold[x]"
        );
        assert_eq!(session.eval_text("x").unwrap().value.to_string(), "10");
    }

    #[test]
    fn block_applies_dynamic_scope_across_called_definitions() {
        let mut session = Session::new();

        session.eval_text("x = 10").unwrap();
        session.eval_text("f[] := x").unwrap();

        assert_eq!(
            session
                .eval_text("Block[{x = 1}, f[]]")
                .unwrap()
                .value
                .to_string(),
            "1"
        );
        assert_eq!(session.eval_text("f[]").unwrap().value.to_string(), "10");
        assert_eq!(
            session
                .eval_text("Block[{x}, x = 3]")
                .unwrap()
                .value
                .to_string(),
            "3"
        );
        assert_eq!(session.eval_text("x").unwrap().value.to_string(), "10");
    }

    #[test]
    fn block_supports_sequential_initializers_and_nested_shadowing() {
        let mut session = Session::new();

        session.eval_text("x = 10").unwrap();

        assert_eq!(
            session
                .eval_text("Block[{x = 1, y = x}, y]")
                .unwrap()
                .value
                .to_string(),
            "1"
        );
        assert_eq!(
            session
                .eval_text("Block[{x -> 1}, x]")
                .unwrap()
                .value
                .to_string(),
            "1"
        );
        assert_eq!(
            session
                .eval_text("Block[{x = 1}, Block[{x = 2}, x]]")
                .unwrap()
                .value
                .to_string(),
            "2"
        );
        assert_eq!(session.eval_text("x").unwrap().value.to_string(), "10");
    }

    #[test]
    fn set_assigns_simple_symbol_and_lookup_works() {
        let mut session = Session::new();

        let assigned = session.eval_text("x = 42").unwrap();
        assert_eq!(assigned.value.to_string(), "42");

        let looked_up = session.eval_text("x").unwrap();
        assert_eq!(looked_up.value.to_string(), "42");

        let info = session.inspect("x").unwrap();
        assert!(info.has_own_value);
        assert!(!info.has_delayed_value);
    }

    #[test]
    fn setdelayed_stores_rhs_without_immediate_evaluation() {
        let mut session = Session::new();

        assert_eq!(
            session.eval_text("y := x").unwrap().value.to_string(),
            "Null"
        );
        assert_eq!(session.eval_text("x = 9").unwrap().value.to_string(), "9");
        assert_eq!(session.eval_text("y").unwrap().value.to_string(), "9");

        let info = session.inspect("y").unwrap();
        assert!(!info.has_own_value);
        assert!(info.has_delayed_value);
        assert!(!info.has_down_values);
    }

    #[test]
    fn set_with_non_symbol_lhs_remains_unevaluated_for_now() {
        let mut session = Session::new();
        let outcome = session.eval_text("f[x] = 1").unwrap();

        assert_eq!(outcome.value.to_string(), "Set[f[x], 1]");
    }

    #[test]
    fn lowers_named_pattern_assignment() {
        let syntax = parse_expr("x_Integer := f[x]").unwrap();
        let lowered = lower_syntax_expr(&syntax).unwrap();

        assert_eq!(
            lowered.to_string(),
            "SetDelayed[Pattern[x, Blank[Integer]], f[x]]"
        );
    }

    #[test]
    fn lowers_slots_and_pure_functions() {
        let syntax = parse_expr("#1 + #2 &").unwrap();
        let lowered = lower_syntax_expr(&syntax).unwrap();

        assert_eq!(lowered.to_string(), "Function[Plus[Slot[1], Slot[2]]]");

        let sequence_syntax = parse_expr("## + ##2 &").unwrap();
        let sequence_lowered = lower_syntax_expr(&sequence_syntax).unwrap();

        assert_eq!(
            sequence_lowered.to_string(),
            "Function[Plus[SlotSequence[1], SlotSequence[2]]]"
        );

        let named_syntax = parse_expr("#x &").unwrap();
        let named_lowered = lower_syntax_expr(&named_syntax).unwrap();

        assert_eq!(named_lowered.to_string(), "Function[Slot[\"x\"]]");
    }

    #[test]
    fn setdelayed_stores_and_applies_simple_downvalue() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("f[x_Integer] := x")
                .unwrap()
                .value
                .to_string(),
            "Null"
        );
        assert_eq!(session.eval_text("f[7]").unwrap().value.to_string(), "7");

        let info = session.inspect("f").unwrap();
        assert!(info.has_down_values);
    }

    #[test]
    fn matchq_reports_true_for_matching_pattern() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("MatchQ[7, x_Integer]")
                .unwrap()
                .value
                .to_string(),
            "True"
        );
    }

    #[test]
    fn matchq_reports_false_for_non_matching_pattern() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("MatchQ[\"hi\", x_Integer]")
                .unwrap()
                .value
                .to_string(),
            "False"
        );
    }

    #[test]
    fn matchq_supports_blanksequence_in_argument_lists() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("MatchQ[f[1, 2], f[x__]]")
                .unwrap()
                .value
                .to_string(),
            "True"
        );
        assert_eq!(
            session
                .eval_text("MatchQ[f[], f[x__]]")
                .unwrap()
                .value
                .to_string(),
            "False"
        );
    }

    #[test]
    fn matchq_supports_blanknullsequence_in_argument_lists() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("MatchQ[f[], f[x___]]")
                .unwrap()
                .value
                .to_string(),
            "True"
        );
        assert_eq!(
            session
                .eval_text("MatchQ[f[1, 2, 3], f[x__, y_Integer]]")
                .unwrap()
                .value
                .to_string(),
            "True"
        );
    }

    #[test]
    fn matchq_supports_conditions_on_sequence_patterns() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("MatchQ[f[1, 2], f[x__ /; MatchQ[x, Sequence[1, 2]]]]")
                .unwrap()
                .value
                .to_string(),
            "True"
        );
        assert_eq!(
            session
                .eval_text("MatchQ[f[1, 2], f[x__ /; MatchQ[x, Sequence[2, 1]]]]")
                .unwrap()
                .value
                .to_string(),
            "False"
        );
    }

    #[test]
    fn downvalue_matches_after_argument_evaluation() {
        let mut session = Session::new();

        session.eval_text("f[x_Integer] := x").unwrap();
        session.eval_text("y = 11").unwrap();

        assert_eq!(session.eval_text("f[y]").unwrap().value.to_string(), "11");
    }

    #[test]
    fn downvalue_can_substitute_multiple_named_patterns() {
        let mut session = Session::new();

        session.eval_text("pair[a_, b_] := {a, b}").unwrap();

        assert_eq!(
            session.eval_text("pair[1, 2]").unwrap().value.to_string(),
            "List[1, 2]"
        );
    }

    #[test]
    fn downvalue_splices_named_blanksequence_into_rhs_calls() {
        let mut session = Session::new();

        session.eval_text("echo[x__] := g[x]").unwrap();

        assert_eq!(
            session
                .eval_text("echo[1, 2, 3]")
                .unwrap()
                .value
                .to_string(),
            "g[1, 2, 3]"
        );
    }

    #[test]
    fn downvalue_splices_named_blanknullsequence_into_empty_lists() {
        let mut session = Session::new();

        session.eval_text("pack[x___] := {x}").unwrap();

        assert_eq!(
            session.eval_text("pack[]").unwrap().value.to_string(),
            "List[]"
        );
        assert_eq!(
            session.eval_text("pack[1, 2]").unwrap().value.to_string(),
            "List[1, 2]"
        );
    }

    #[test]
    fn unmatched_downvalue_leaves_call_in_place() {
        let mut session = Session::new();

        session.eval_text("f[x_Integer] := x").unwrap();

        assert_eq!(
            session.eval_text("f[\"hi\"]").unwrap().value.to_string(),
            "f[\"hi\"]"
        );
    }

    #[test]
    fn replaceall_applies_single_rule() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("7 /. x_Integer -> x")
                .unwrap()
                .value
                .to_string(),
            "7"
        );
    }

    #[test]
    fn replaceall_applies_rule_list_in_order() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("7 /. {x_String -> 0, x_Integer -> 1}")
                .unwrap()
                .value
                .to_string(),
            "1"
        );
    }

    #[test]
    fn replaceall_traverses_nested_expressions() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("f[g[1]] /. g[x_Integer] -> h[x]")
                .unwrap()
                .value
                .to_string(),
            "f[h[1]]"
        );
    }

    #[test]
    fn replaceall_splices_named_blanksequence_into_rhs_calls() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("f[1, 2] /. f[x__] -> g[x]")
                .unwrap()
                .value
                .to_string(),
            "g[1, 2]"
        );
    }

    #[test]
    fn replacerepeated_reaches_a_fixed_point() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("f[1] //. {f[x_Integer] -> g[x], g[x_Integer] -> x}")
                .unwrap()
                .value
                .to_string(),
            "1"
        );
    }

    #[test]
    fn replacerepeated_errors_on_non_terminating_rewrite_cycle() {
        let mut session = Session::new();

        let error = session
            .eval_text("1 //. {1 -> 2, 2 -> 1}")
            .expect_err("expected iteration limit error");

        assert!(matches!(
            error,
            EvalError::IterationLimitExceeded {
                limit: DEFAULT_REPLACE_REPEATED_LIMIT
            }
        ));
    }

    #[test]
    fn immediate_rule_captures_current_rhs_value() {
        let mut session = Session::new();

        session.eval_text("n = 1").unwrap();
        session.eval_text("rule = x_Integer -> n").unwrap();
        session.eval_text("n = 2").unwrap();

        assert_eq!(
            session.eval_text("7 /. rule").unwrap().value.to_string(),
            "1"
        );
    }

    #[test]
    fn delayed_rule_uses_current_rhs_value_at_application_time() {
        let mut session = Session::new();

        session.eval_text("n = 1").unwrap();
        session.eval_text("rule = x_Integer :> n").unwrap();
        session.eval_text("n = 2").unwrap();

        assert_eq!(
            session.eval_text("7 /. rule").unwrap().value.to_string(),
            "2"
        );
    }

    #[test]
    fn matchq_honors_condition_patterns() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("MatchQ[7, x_Integer /; MatchQ[x, 7]]")
                .unwrap()
                .value
                .to_string(),
            "True"
        );
        assert_eq!(
            session
                .eval_text("MatchQ[8, x_Integer /; MatchQ[x, 7]]")
                .unwrap()
                .value
                .to_string(),
            "False"
        );
    }

    #[test]
    fn cases_collects_nested_matches() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("Cases[f[g[1], 2, \"hi\"], x_Integer]")
                .unwrap()
                .value
                .to_string(),
            "List[1, 2]"
        );
    }

    #[test]
    fn cases_can_match_the_root_expression() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("Cases[f[1], f[x_Integer]]")
                .unwrap()
                .value
                .to_string(),
            "List[f[1]]"
        );
    }

    #[test]
    fn cases_does_not_treat_heads_as_separate_candidates() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("Cases[f[1, g[2]], x_Symbol]")
                .unwrap()
                .value
                .to_string(),
            "List[]"
        );
    }

    #[test]
    fn cases_applies_rule_transformations() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("Cases[f[g[1], 2], g[x_Integer] :> h[x]]")
                .unwrap()
                .value
                .to_string(),
            "List[h[1]]"
        );
    }

    #[test]
    fn cases_preserves_immediate_and_delayed_rule_behavior() {
        let mut session = Session::new();

        session.eval_text("n = 1").unwrap();
        session.eval_text("immediate = x_Integer -> n").unwrap();
        session.eval_text("delayed = x_Integer :> n").unwrap();
        session.eval_text("n = 2").unwrap();

        assert_eq!(
            session
                .eval_text("Cases[{7}, immediate]")
                .unwrap()
                .value
                .to_string(),
            "List[1]"
        );
        assert_eq!(
            session
                .eval_text("Cases[{7}, delayed]")
                .unwrap()
                .value
                .to_string(),
            "List[2]"
        );
    }

    #[test]
    fn cases_honors_condition_patterns() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("Cases[f[7, 8, 9], x_Integer /; MatchQ[x, 8]]")
                .unwrap()
                .value
                .to_string(),
            "List[8]"
        );
    }

    #[test]
    fn cases_rule_rhs_splices_named_blanksequence_matches() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("Cases[f[g[1, 2], g[3]], g[x__] :> {x}]")
                .unwrap()
                .value
                .to_string(),
            "List[List[1, 2], List[3]]"
        );
    }

    #[test]
    fn replaceall_honors_condition_patterns() {
        let mut session = Session::new();

        assert_eq!(
            session
                .eval_text("7 /. (x_Integer /; MatchQ[x, 7]) -> y")
                .unwrap()
                .value
                .to_string(),
            "y"
        );
        assert_eq!(
            session
                .eval_text("8 /. (x_Integer /; MatchQ[x, 7]) -> y")
                .unwrap()
                .value
                .to_string(),
            "8"
        );
    }

    #[test]
    fn conditioned_downvalue_matches_only_when_test_passes() {
        let mut session = Session::new();

        session
            .eval_text("f[x_Integer /; MatchQ[x, 7]] := x")
            .unwrap();

        assert_eq!(session.eval_text("f[7]").unwrap().value.to_string(), "7");
        assert_eq!(session.eval_text("f[8]").unwrap().value.to_string(), "f[8]");
    }

    #[test]
    fn outer_conditioned_downvalue_is_stored_and_applied() {
        let mut session = Session::new();

        session
            .eval_text("(f[x_Integer] /; MatchQ[x, 7]) := x")
            .unwrap();

        assert_eq!(session.eval_text("f[7]").unwrap().value.to_string(), "7");
        assert_eq!(session.eval_text("f[8]").unwrap().value.to_string(), "f[8]");
    }
}
