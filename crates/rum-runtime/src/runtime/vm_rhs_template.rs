use super::*;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

const RHS_TEMPLATE_ENV: &str = "RUM_VM_RHS_TEMPLATE";

static RHS_TEMPLATE_ENABLED: AtomicU8 = AtomicU8::new(0);
static RHS_TEMPLATE_COMPILED_DISTINCT: AtomicU64 = AtomicU64::new(0);
static RHS_TEMPLATE_FALLBACK_DISTINCT: AtomicU64 = AtomicU64::new(0);

type RuntimeStateRef = Arc<Mutex<RuntimeState>>;
type CompiledClosure =
    Arc<dyn Fn(&mut Session, RuntimeStateRef) -> Result<Expr, EvalError> + Send + Sync + 'static>;

#[derive(Clone)]
struct CodeObject {
    source: Expr,
    body: CompiledClosure,
    source_template: Option<TemplateNode>,
    slots: Vec<Symbol>,
    const_symbols: Vec<Symbol>,
    local_specs: Vec<LocalSpec>,
    shape_count: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LocalKind {
    Module,
    With,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LocalSpec {
    symbol: Symbol,
    kind: LocalKind,
}

#[derive(Clone, Debug)]
struct RuntimeState {
    slot_values: Vec<Expr>,
    locals: Vec<LocalRuntime>,
    option_scope: Option<OptionValueScope>,
}

#[derive(Clone, Debug)]
enum LocalRuntime {
    Module { fresh: Symbol, value: Option<Expr> },
    With { value: Option<Expr> },
}

#[derive(Clone)]
pub(super) enum PreparedRhsTemplate {
    Substituted(Expr),
    Compiled(PreparedCompiledRhsTemplate),
}

#[derive(Clone)]
pub(super) struct PreparedCompiledRhsTemplate {
    code: Arc<CodeObject>,
    slot_values: Vec<Expr>,
}

#[derive(Clone)]
enum CacheEntry {
    Compiled(Arc<CodeObject>),
    NonCompilable(Expr),
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum RhsTemplateKey {
    Atom(Expr),
    Call { head: usize, args: usize },
}

impl Hash for RhsTemplateKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Atom(expr) => {
                0_u8.hash(state);
                expr.hash(state);
            }
            Self::Call { head, args } => {
                1_u8.hash(state);
                head.hash(state);
                args.hash(state);
            }
        }
    }
}

#[derive(Default)]
struct TemplateCache {
    epoch: u64,
    entries: HashMap<RhsTemplateKey, CacheEntry>,
    reported_compiled: HashSet<RhsTemplateKey>,
    reported_fallback: HashSet<RhsTemplateKey>,
}

#[derive(Clone)]
enum TemplateNode {
    Const(SharedExpr),
    Binding(usize),
    Local(usize),
    Call {
        head: Box<TemplateNode>,
        args: Vec<TemplateNode>,
    },
}

#[derive(Clone, Copy)]
struct LocalBinding {
    symbol: Symbol,
    slot: usize,
}

pub(super) fn prepare_rule_bindings_for_rhs_with_template(
    expr: &Expr,
    bindings: &SymbolExprMap,
    may_splice_without_bindings: bool,
    evaluation_epoch: u64,
) -> PreparedRhsTemplate {
    if !rhs_template_enabled() {
        return PreparedRhsTemplate::Substituted(substitute_rule_bindings_for_rhs(
            expr,
            bindings,
            may_splice_without_bindings,
        ));
    }

    prepare_rule_bindings_for_rhs_with_template_slow(
        expr,
        bindings,
        may_splice_without_bindings,
        evaluation_epoch,
    )
}

pub(super) fn trace_rhs_for_prepared_template(
    prepared: &PreparedRhsTemplate,
    original: &Expr,
    bindings: &SymbolExprMap,
    may_splice_without_bindings: bool,
    include_payload: bool,
) -> Expr {
    match prepared {
        PreparedRhsTemplate::Substituted(expr) => expr.clone(),
        PreparedRhsTemplate::Compiled(_) if include_payload => {
            substitute_rule_bindings_for_rhs(original, bindings, may_splice_without_bindings)
        }
        PreparedRhsTemplate::Compiled(_) => original.clone(),
    }
}

pub(super) fn evaluate_prepared_rhs_template(
    session: &mut Session,
    prepared: PreparedRhsTemplate,
    option_scope: Option<OptionValueScope>,
) -> Result<Expr, EvalError> {
    match prepared {
        PreparedRhsTemplate::Substituted(rhs) => session.evaluate_definition_rhs(option_scope, rhs),
        PreparedRhsTemplate::Compiled(prepared) => {
            let result = execute_compiled_body(session, prepared, option_scope);
            let outcome = session.suspend_with_result_frame_lazy(result, || {
                move |_, outcome| finish_compiled_definition_rhs_result(outcome)
            });

            match outcome {
                Err(EvalError::InputRequired { request }) => {
                    Err(EvalError::InputRequired { request })
                }
                outcome => finish_compiled_definition_rhs_result(outcome),
            }
        }
    }
}

#[inline]
fn rhs_template_enabled() -> bool {
    match RHS_TEMPLATE_ENABLED.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_rhs_template_enabled(),
    }
}

#[cold]
fn initialize_rhs_template_enabled() -> bool {
    let enabled =
        std::env::var_os(RHS_TEMPLATE_ENV).is_some_and(|value| !value.is_empty() && value != "0");
    RHS_TEMPLATE_ENABLED.store(if enabled { 1 } else { 2 }, Ordering::Relaxed);
    enabled
}

#[cold]
#[inline(never)]
fn prepare_rule_bindings_for_rhs_with_template_slow(
    expr: &Expr,
    bindings: &SymbolExprMap,
    may_splice_without_bindings: bool,
    evaluation_epoch: u64,
) -> PreparedRhsTemplate {
    if let Some(code) = cached_rhs_template(expr, bindings, evaluation_epoch) {
        if let Some(slot_values) = slot_values_for_code(&code, bindings) {
            #[cfg(debug_assertions)]
            debug_assert_template_matches_tree_walk(
                &code,
                &slot_values,
                expr,
                bindings,
                may_splice_without_bindings,
            );
            return PreparedRhsTemplate::Compiled(PreparedCompiledRhsTemplate {
                code,
                slot_values,
            });
        }
    }

    PreparedRhsTemplate::Substituted(substitute_rule_bindings_for_rhs(
        expr,
        bindings,
        may_splice_without_bindings,
    ))
}

fn finish_compiled_definition_rhs_result(
    outcome: Result<Expr, EvalError>,
) -> Result<Expr, EvalError> {
    match outcome {
        Ok(value) => Ok(value),
        Err(EvalError::Return { value }) => Ok(value),
        Err(error) => Err(error),
    }
}

fn cached_rhs_template(
    expr: &Expr,
    bindings: &SymbolExprMap,
    evaluation_epoch: u64,
) -> Option<Arc<CodeObject>> {
    let key = rhs_template_key(expr);
    let mut cache = rhs_template_cache()
        .lock()
        .expect("RHS template cache lock should not be poisoned");

    if cache.epoch != evaluation_epoch {
        cache.epoch = evaluation_epoch;
        cache.entries.clear();
    }

    if let Some(entry) = cache.entries.get(&key) {
        match entry {
            CacheEntry::Compiled(code)
                if code.source == *expr && code_compatible_with_bindings(code, bindings) =>
            {
                return Some(code.clone());
            }
            CacheEntry::NonCompilable(source) if source == expr => return None,
            _ => {}
        }
    }

    let compiled = compile_body(expr, bindings).map(Arc::new);
    match compiled {
        Some(code) => {
            cache
                .entries
                .insert(key.clone(), CacheEntry::Compiled(code.clone()));
            record_distinct_template_compile(&mut cache, key, code.shape_count);
            Some(code)
        }
        None => {
            cache
                .entries
                .insert(key.clone(), CacheEntry::NonCompilable(expr.clone()));
            record_distinct_template_fallback(&mut cache, key);
            None
        }
    }
}

fn rhs_template_cache() -> &'static Mutex<TemplateCache> {
    static CACHE: OnceLock<Mutex<TemplateCache>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(TemplateCache::default()))
}

fn rhs_template_key(expr: &Expr) -> RhsTemplateKey {
    match expr {
        Expr::Atom(_) => RhsTemplateKey::Atom(expr.clone()),
        Expr::Call { head, args } => RhsTemplateKey::Call {
            head: head.ptr_key(),
            args: args.ptr_key(),
        },
    }
}

fn record_distinct_template_compile(
    cache: &mut TemplateCache,
    key: RhsTemplateKey,
    shape_count: usize,
) {
    if cache.reported_compiled.insert(key) {
        let compiled = RHS_TEMPLATE_COMPILED_DISTINCT.fetch_add(1, Ordering::Relaxed) + 1;
        let fallback = RHS_TEMPLATE_FALLBACK_DISTINCT.load(Ordering::Relaxed);
        eprintln!(
            "rum-vm-rhs-template coverage closure_compiled_distinct={compiled} fallback_distinct={fallback} last_shape_nodes={shape_count}"
        );
    }
}

fn record_distinct_template_fallback(cache: &mut TemplateCache, key: RhsTemplateKey) {
    if cache.reported_fallback.insert(key) {
        let fallback = RHS_TEMPLATE_FALLBACK_DISTINCT.fetch_add(1, Ordering::Relaxed) + 1;
        let compiled = RHS_TEMPLATE_COMPILED_DISTINCT.load(Ordering::Relaxed);
        eprintln!(
            "rum-vm-rhs-template coverage closure_compiled_distinct={compiled} fallback_distinct={fallback}"
        );
    }
}

fn compile_body(expr: &Expr, bindings: &SymbolExprMap) -> Option<CodeObject> {
    let mut compiler = ClosureBodyCompiler::new(bindings);
    let body = compiler.compile_eval(expr)?;
    let source_template = if compiler.local_specs.is_empty() {
        Some(compiler.compile_template(expr)?)
    } else {
        None
    };
    Some(CodeObject {
        source: expr.clone(),
        body,
        source_template,
        slots: compiler.slots,
        const_symbols: compiler.const_symbols,
        local_specs: compiler.local_specs,
        shape_count: compiler.shape_count,
    })
}

#[cfg(debug_assertions)]
fn debug_assert_template_matches_tree_walk(
    code: &CodeObject,
    slot_values: &[Expr],
    expr: &Expr,
    bindings: &SymbolExprMap,
    may_splice_without_bindings: bool,
) {
    let Some(source_template) = &code.source_template else {
        return;
    };
    let state = Arc::new(Mutex::new(RuntimeState {
        slot_values: slot_values.to_vec(),
        locals: Vec::new(),
        option_scope: None,
    }));
    let result = instantiate_template(source_template, &state)
        .expect("compiled source template should instantiate with prepared bindings");
    let expected = substitute_rule_bindings_for_rhs(expr, bindings, may_splice_without_bindings);
    debug_assert_eq!(
        result, expected,
        "RHS closure template diverged from tree substitution"
    );
}

struct ClosureBodyCompiler<'a> {
    bindings: &'a SymbolExprMap,
    slots: Vec<Symbol>,
    const_symbols: Vec<Symbol>,
    local_specs: Vec<LocalSpec>,
    locals: Vec<LocalBinding>,
    shape_count: usize,
}

impl<'a> ClosureBodyCompiler<'a> {
    fn new(bindings: &'a SymbolExprMap) -> Self {
        Self {
            bindings,
            slots: Vec::new(),
            const_symbols: Vec::new(),
            local_specs: Vec::new(),
            locals: Vec::new(),
            shape_count: 0,
        }
    }

    fn compile_eval(&mut self, expr: &Expr) -> Option<CompiledClosure> {
        self.shape_count += 1;
        match expr {
            Expr::Call { head, args } if is_system_symbol_name(head, "If") => self.compile_if(args),
            Expr::Call { head, args } if is_system_symbol_name(head, "CompoundExpression") => {
                self.compile_compound(args)
            }
            Expr::Call { head, args } if is_system_symbol_name(head, "Return") => {
                self.compile_return(args)
            }
            Expr::Call { head, args } if is_system_symbol_name(head, "Set") => {
                self.compile_set(args).or_else(|| self.compile_leaf(expr))
            }
            Expr::Call { head, args }
                if is_system_symbol_name(head, "Module") && args.len() == 2 =>
            {
                self.compile_module(args)
            }
            Expr::Call { head, args } if is_system_symbol_name(head, "With") && args.len() == 2 => {
                self.compile_with(args)
            }
            Expr::Call { head, .. } if is_system_symbol_name(head, "Function") => None,
            _ => self.compile_leaf(expr),
        }
    }

    fn compile_if(&mut self, args: &SharedArgs) -> Option<CompiledClosure> {
        if !(2..=4).contains(&args.len()) {
            return None;
        }
        let condition = self.compile_eval(&args[0])?;
        let true_branch = self.compile_eval(&args[1])?;
        let false_branch = if args.len() >= 3 {
            Some(self.compile_eval(&args[2])?)
        } else {
            None
        };
        let unknown_branch = if args.len() == 4 {
            Some(self.compile_eval(&args[3])?)
        } else {
            None
        };
        let residual_templates = args
            .iter()
            .map(|arg| self.compile_template(arg))
            .collect::<Option<Vec<_>>>()?;

        Some(Arc::new(move |session, state| {
            let result = condition(session, state.clone());
            let value = session.suspend_with_frame_lazy(result, || {
                let true_branch = true_branch.clone();
                let false_branch = false_branch.clone();
                let unknown_branch = unknown_branch.clone();
                let residual_templates = residual_templates.clone();
                let state = state.clone();
                move |session, condition| {
                    resume_compiled_if(
                        session,
                        state,
                        condition,
                        true_branch,
                        false_branch,
                        unknown_branch,
                        residual_templates,
                    )
                }
            })?;
            resume_compiled_if(
                session,
                state,
                value,
                true_branch.clone(),
                false_branch.clone(),
                unknown_branch.clone(),
                residual_templates.clone(),
            )
        }))
    }

    fn compile_compound(&mut self, args: &SharedArgs) -> Option<CompiledClosure> {
        let items = args
            .iter()
            .map(|arg| self.compile_eval(arg))
            .collect::<Option<Vec<_>>>()?;
        let items = Arc::new(items);
        Some(Arc::new(move |session, state| {
            resume_compiled_compound(session, items.clone(), state, 0)
        }))
    }

    fn compile_return(&mut self, args: &SharedArgs) -> Option<CompiledClosure> {
        match args.as_slice() {
            [] => Some(Arc::new(|_, _| {
                Err(EvalError::Return {
                    value: system_expr("Null"),
                })
            })),
            [value] => {
                let value = self.compile_eval(value)?;
                Some(Arc::new(move |session, state| {
                    let result = value(session, state.clone());
                    let value = session.suspend_with_frame_lazy(result, || {
                        move |_, value| Err(EvalError::Return { value })
                    })?;
                    Err(EvalError::Return { value })
                }))
            }
            _ => None,
        }
    }

    fn compile_set(&mut self, args: &SharedArgs) -> Option<CompiledClosure> {
        let [lhs, rhs] = args.as_slice() else {
            return None;
        };
        let symbol = expr_as_symbol(lhs)?;
        let local = self.visible_local(*symbol)?;
        let rhs = self.compile_eval(rhs)?;
        Some(Arc::new(move |session, state| {
            let result = rhs(session, state.clone());
            let value = session.suspend_with_frame_lazy(result, || {
                let state = state.clone();
                move |_, value| {
                    set_local_value(&state, local.slot, value.clone());
                    Ok(value)
                }
            })?;
            set_local_value(&state, local.slot, value.clone());
            Ok(value)
        }))
    }

    fn compile_module(&mut self, args: &SharedArgs) -> Option<CompiledClosure> {
        let items = parse_module_binding_list(&args[0])?;
        if items.is_empty() || duplicate_scoped_symbols(items.iter().map(|(symbol, _)| *symbol)) {
            return None;
        }
        let local_symbols = items.iter().map(|(symbol, _)| *symbol).collect::<Vec<_>>();
        if !locals_are_safe_in_compiled_expr(&args[1], &local_symbols) {
            return None;
        }

        let local_base = self.locals.len();
        let mut local_slots = Vec::with_capacity(items.len());
        for (symbol, _) in &items {
            let slot = self.push_local(*symbol, LocalKind::Module);
            local_slots.push((slot, *symbol));
        }

        let mut initializers = Vec::new();
        for (index, (_, init)) in items.iter().enumerate() {
            if let Some(init) = init {
                initializers.push((local_slots[index].0, self.compile_eval(init)?));
            }
        }
        let body = self.compile_eval(&args[1])?;
        self.locals.truncate(local_base);

        let initializers = Arc::new(initializers);
        Some(Arc::new(move |session, state| {
            initialize_module_locals(session, &state, &local_slots);
            resume_module_initializers(session, initializers.clone(), body.clone(), state, 0)
        }))
    }

    fn compile_with(&mut self, args: &SharedArgs) -> Option<CompiledClosure> {
        let items = parse_with_binding_list(&args[0])?;
        if items.is_empty() || duplicate_scoped_symbols(items.iter().map(|(symbol, _)| *symbol)) {
            return None;
        }

        let local_base = self.locals.len();
        let mut initializers = Vec::with_capacity(items.len());
        for (symbol, rhs) in &items {
            let rhs = self.compile_eval(rhs)?;
            let slot = self.push_local(*symbol, LocalKind::With);
            initializers.push((slot, rhs));
        }
        let body = self.compile_eval(&args[1])?;
        self.locals.truncate(local_base);

        let initializers = Arc::new(initializers);
        Some(Arc::new(move |session, state| {
            initialize_with_locals(&state, &initializers);
            resume_with_initializers(session, initializers.clone(), body.clone(), state, 0)
        }))
    }

    fn compile_leaf(&mut self, expr: &Expr) -> Option<CompiledClosure> {
        if leaf_contains_uncompiled_scope(expr) {
            return None;
        }
        let template = self.compile_template(expr)?;
        Some(Arc::new(move |session, state| {
            let expr = instantiate_template(&template, &state)
                .expect("compiled leaf template should instantiate with prepared bindings");
            evaluate_instantiated_expr(session, state, expr)
        }))
    }

    fn compile_template(&mut self, expr: &Expr) -> Option<TemplateNode> {
        match expr {
            Expr::Atom(Atom::Symbol(symbol)) if self.visible_local(*symbol).is_some() => {
                Some(TemplateNode::Local(
                    self.visible_local(*symbol)
                        .expect("checked local should remain visible")
                        .slot,
                ))
            }
            Expr::Atom(Atom::Symbol(symbol)) if self.bindings.contains_key(symbol) => {
                Some(TemplateNode::Binding(self.binding_slot(*symbol)))
            }
            Expr::Atom(Atom::Symbol(symbol)) => {
                self.record_const_symbol(*symbol);
                Some(TemplateNode::Const(SharedExpr::from(expr.clone())))
            }
            Expr::Atom(_) => Some(TemplateNode::Const(SharedExpr::from(expr.clone()))),
            Expr::Call { head, args } => {
                let head = Box::new(self.compile_template(head)?);
                let args = args
                    .iter()
                    .map(|arg| self.compile_template(arg))
                    .collect::<Option<Vec<_>>>()?;
                Some(TemplateNode::Call { head, args })
            }
        }
    }

    fn push_local(&mut self, symbol: Symbol, kind: LocalKind) -> usize {
        let slot = self.local_specs.len();
        self.local_specs.push(LocalSpec { symbol, kind });
        self.locals.push(LocalBinding { symbol, slot });
        slot
    }

    fn visible_local(&self, symbol: Symbol) -> Option<LocalBinding> {
        self.locals
            .iter()
            .rev()
            .find(|local| local.symbol == symbol)
            .copied()
    }

    fn binding_slot(&mut self, symbol: Symbol) -> usize {
        if let Some(index) = self.slots.iter().position(|slot| *slot == symbol) {
            index
        } else {
            let index = self.slots.len();
            self.slots.push(symbol);
            index
        }
    }

    fn record_const_symbol(&mut self, symbol: Symbol) {
        if !self.const_symbols.contains(&symbol) {
            self.const_symbols.push(symbol);
        }
    }
}

fn code_compatible_with_bindings(code: &CodeObject, bindings: &SymbolExprMap) -> bool {
    code.slots
        .iter()
        .all(|symbol| bindings.contains_key(symbol))
        && code
            .const_symbols
            .iter()
            .all(|symbol| !bindings.contains_key(symbol))
}

fn slot_values_for_code(code: &CodeObject, bindings: &SymbolExprMap) -> Option<Vec<Expr>> {
    if code
        .const_symbols
        .iter()
        .any(|symbol| bindings.contains_key(symbol))
    {
        return None;
    }

    let mut slot_values = Vec::with_capacity(code.slots.len());
    for symbol in &code.slots {
        slot_values.push(bindings.get(symbol)?.clone());
    }
    Some(slot_values)
}

fn execute_compiled_body(
    session: &mut Session,
    prepared: PreparedCompiledRhsTemplate,
    option_scope: Option<OptionValueScope>,
) -> Result<Expr, EvalError> {
    let state = runtime_state(session, &prepared.code, prepared.slot_values, option_scope);
    (prepared.code.body)(session, state)
}

fn runtime_state(
    session: &mut Session,
    code: &CodeObject,
    slot_values: Vec<Expr>,
    option_scope: Option<OptionValueScope>,
) -> RuntimeStateRef {
    let locals = code
        .local_specs
        .iter()
        .map(|spec| match spec.kind {
            LocalKind::Module => LocalRuntime::Module {
                fresh: session.allocate_fresh_symbol(&spec.symbol),
                value: None,
            },
            LocalKind::With => LocalRuntime::With { value: None },
        })
        .collect();
    Arc::new(Mutex::new(RuntimeState {
        slot_values,
        locals,
        option_scope,
    }))
}

fn instantiate_template(template: &TemplateNode, state: &RuntimeStateRef) -> Option<Expr> {
    match template {
        TemplateNode::Const(expr) => Some(expr.as_ref().clone()),
        TemplateNode::Binding(slot) => {
            let state = state
                .lock()
                .expect("compiled RHS state lock should not poison");
            state.slot_values.get(*slot).cloned()
        }
        TemplateNode::Local(slot) => local_expr(state, *slot),
        TemplateNode::Call { head, args } => {
            let head = instantiate_template(head, state)?;
            let mut output_args = Vec::with_capacity(args.len());
            for arg in args {
                let value = instantiate_template(arg, state)?;
                if let Some(items) = sequence_items(&value) {
                    output_args.extend(items.iter().cloned());
                } else {
                    output_args.push(value);
                }
            }
            Some(Expr::call(head, output_args))
        }
    }
}

fn local_expr(state: &RuntimeStateRef, slot: usize) -> Option<Expr> {
    let state = state
        .lock()
        .expect("compiled RHS state lock should not poison");
    match state.locals.get(slot)? {
        LocalRuntime::Module { fresh, .. } => Some(Expr::Atom(Atom::Symbol(*fresh))),
        LocalRuntime::With { value } => value.clone(),
    }
}

fn set_local_value(state: &RuntimeStateRef, slot: usize, value: Expr) {
    let mut state = state
        .lock()
        .expect("compiled RHS state lock should not poison");
    match state
        .locals
        .get_mut(slot)
        .expect("compiled RHS local slot should exist")
    {
        LocalRuntime::Module {
            value: local_value, ..
        }
        | LocalRuntime::With { value: local_value } => *local_value = Some(value),
    }
}

fn evaluate_instantiated_expr(
    session: &mut Session,
    state: RuntimeStateRef,
    expr: Expr,
) -> Result<Expr, EvalError> {
    if expr_is_self_evaluating_atom(&expr) {
        return Ok(expr);
    }

    let option_scope = {
        let state = state
            .lock()
            .expect("compiled RHS state lock should not poison");
        state.option_scope.clone()
    };
    let dynamic_depth = push_module_local_bindings(session, &state);
    let result = session.evaluate_with_option_value_scope(option_scope, expr);
    match result {
        Ok(value) => {
            truncate_dynamic_locals(session, dynamic_depth);
            Ok(value)
        }
        Err(EvalError::InputRequired { request }) => {
            session.push_suspension_frame(move |session, outcome| {
                truncate_dynamic_locals(session, dynamic_depth);
                outcome
            });
            Err(EvalError::InputRequired { request })
        }
        Err(error) => {
            truncate_dynamic_locals(session, dynamic_depth);
            Err(error)
        }
    }
}

fn push_module_local_bindings(session: &mut Session, state: &RuntimeStateRef) -> Option<usize> {
    let locals = {
        let state = state
            .lock()
            .expect("compiled RHS state lock should not poison");
        state
            .locals
            .iter()
            .filter_map(|local| match local {
                LocalRuntime::Module { fresh, value } => Some((*fresh, value.clone())),
                LocalRuntime::With { .. } => None,
            })
            .collect::<Vec<_>>()
    };

    let mut depth = None;
    for (symbol, value) in locals {
        let pushed_depth = session.push_dynamic_own_value_binding(symbol, value);
        depth.get_or_insert(pushed_depth);
    }
    depth
}

fn truncate_dynamic_locals(session: &mut Session, depth: Option<usize>) {
    if let Some(depth) = depth {
        session.truncate_dynamic_own_value_bindings(depth);
    }
}

fn resume_compiled_if(
    session: &mut Session,
    state: RuntimeStateRef,
    condition: Expr,
    true_branch: CompiledClosure,
    false_branch: Option<CompiledClosure>,
    unknown_branch: Option<CompiledClosure>,
    residual_templates: Vec<TemplateNode>,
) -> Result<Expr, EvalError> {
    match truthiness_symbol(&condition) {
        Some(true) => true_branch(session, state),
        Some(false) => {
            if let Some(false_branch) = false_branch {
                false_branch(session, state)
            } else {
                Ok(system_expr("Null"))
            }
        }
        None => {
            if let Some(unknown_branch) = unknown_branch {
                return unknown_branch(session, state);
            }
            let mut args = Vec::with_capacity(residual_templates.len());
            args.push(condition);
            for template in residual_templates.iter().skip(1) {
                args.push(instantiate_template(template, &state).expect(
                    "compiled If residual templates should instantiate with current bindings",
                ));
            }
            Ok(Expr::call(system_expr("If"), args))
        }
    }
}

fn resume_compiled_compound(
    session: &mut Session,
    items: Arc<Vec<CompiledClosure>>,
    state: RuntimeStateRef,
    index: usize,
) -> Result<Expr, EvalError> {
    let Some(item) = items.get(index).cloned() else {
        return Ok(system_expr("Null"));
    };
    let result = item(session, state.clone());
    let value = session.suspend_with_frame_lazy(result, || {
        let items = items.clone();
        let state = state.clone();
        move |session, value| {
            if index + 1 >= items.len() {
                Ok(value)
            } else {
                resume_compiled_compound(session, items, state, index + 1)
            }
        }
    })?;

    if index + 1 >= items.len() {
        Ok(value)
    } else {
        resume_compiled_compound(session, items, state, index + 1)
    }
}

fn initialize_module_locals(
    session: &mut Session,
    state: &RuntimeStateRef,
    local_slots: &[(usize, Symbol)],
) {
    let mut state = state
        .lock()
        .expect("compiled RHS state lock should not poison");
    for (slot, symbol) in local_slots {
        state.locals[*slot] = LocalRuntime::Module {
            fresh: session.allocate_fresh_symbol(symbol),
            value: None,
        };
    }
}

fn resume_module_initializers(
    session: &mut Session,
    initializers: Arc<Vec<(usize, CompiledClosure)>>,
    body: CompiledClosure,
    state: RuntimeStateRef,
    index: usize,
) -> Result<Expr, EvalError> {
    let Some((slot, init)) = initializers.get(index).cloned() else {
        return body(session, state);
    };
    let result = init(session, state.clone());
    let value = session.suspend_with_frame_lazy(result, || {
        let initializers = initializers.clone();
        let body = body.clone();
        let state = state.clone();
        move |session, value| {
            set_local_value(&state, slot, value);
            resume_module_initializers(session, initializers, body, state, index + 1)
        }
    })?;
    set_local_value(&state, slot, value);
    resume_module_initializers(session, initializers, body, state, index + 1)
}

fn initialize_with_locals(
    state: &RuntimeStateRef,
    initializers: &Arc<Vec<(usize, CompiledClosure)>>,
) {
    let mut state = state
        .lock()
        .expect("compiled RHS state lock should not poison");
    for (slot, _) in initializers.iter() {
        state.locals[*slot] = LocalRuntime::With { value: None };
    }
}

fn resume_with_initializers(
    session: &mut Session,
    initializers: Arc<Vec<(usize, CompiledClosure)>>,
    body: CompiledClosure,
    state: RuntimeStateRef,
    index: usize,
) -> Result<Expr, EvalError> {
    let Some((slot, init)) = initializers.get(index).cloned() else {
        return body(session, state);
    };
    let result = init(session, state.clone());
    let value = session.suspend_with_frame_lazy(result, || {
        let initializers = initializers.clone();
        let body = body.clone();
        let state = state.clone();
        move |session, value| {
            set_local_value(&state, slot, value);
            resume_with_initializers(session, initializers, body, state, index + 1)
        }
    })?;
    set_local_value(&state, slot, value);
    resume_with_initializers(session, initializers, body, state, index + 1)
}

fn duplicate_scoped_symbols(symbols: impl Iterator<Item = Symbol>) -> bool {
    let mut seen = HashSet::new();
    symbols.into_iter().any(|symbol| !seen.insert(symbol))
}

fn leaf_contains_uncompiled_scope(expr: &Expr) -> bool {
    match expr {
        Expr::Call { head, args } => {
            is_system_symbol_name(head, "Module")
                || is_system_symbol_name(head, "With")
                || is_system_symbol_name(head, "Function")
                || leaf_contains_uncompiled_scope(head)
                || args.iter().any(leaf_contains_uncompiled_scope)
        }
        Expr::Atom(_) => false,
    }
}

fn locals_are_safe_in_compiled_expr(expr: &Expr, locals: &[Symbol]) -> bool {
    match expr {
        Expr::Atom(_) => true,
        Expr::Call { head, args } if is_system_symbol_name(head, "If") => args
            .iter()
            .all(|arg| locals_are_safe_in_compiled_expr(arg, locals)),
        Expr::Call { head, args } if is_system_symbol_name(head, "CompoundExpression") => {
            if args
                .last()
                .and_then(expr_as_symbol)
                .is_some_and(|symbol| locals.contains(symbol))
            {
                return false;
            }
            args.iter()
                .all(|arg| locals_are_safe_in_compiled_expr(arg, locals))
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Return") => args
            .iter()
            .all(|arg| locals_are_safe_in_compiled_expr(arg, locals)),
        Expr::Call { head, args } if is_system_symbol_name(head, "Set") && args.len() == 2 => {
            if expr_as_symbol(&args[0]).is_some_and(|symbol| locals.contains(symbol)) {
                locals_are_safe_in_compiled_expr(&args[1], locals)
            } else {
                local_reads_are_leaf_safe(expr, locals)
            }
        }
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Module") || is_system_symbol_name(head, "With"))
                && args.len() == 2 =>
        {
            !expr_contains_any_symbol(expr, locals)
        }
        Expr::Call { head, .. } if is_system_symbol_name(head, "Function") => false,
        Expr::Call { .. } => local_reads_are_leaf_safe(expr, locals),
    }
}

fn local_reads_are_leaf_safe(expr: &Expr, locals: &[Symbol]) -> bool {
    match expr {
        Expr::Atom(_) => true,
        Expr::Call { head, args } => {
            if expr_as_symbol(head).is_some_and(|symbol| locals.contains(symbol)) {
                return false;
            }
            if head_blocks_registerized_locals(head)
                && args.iter().any(|arg| expr_contains_any_symbol(arg, locals))
            {
                return false;
            }
            local_reads_are_leaf_safe(head, locals)
                && args
                    .iter()
                    .all(|arg| local_reads_are_leaf_safe(arg, locals))
        }
    }
}

fn head_blocks_registerized_locals(head: &Expr) -> bool {
    [
        "Condition",
        "Defer",
        "Function",
        "Hold",
        "HoldComplete",
        "HoldForm",
        "Inactive",
        "Optional",
        "Pattern",
        "PatternTest",
        "Rule",
        "RuleDelayed",
        "Set",
        "SetDelayed",
        "Unevaluated",
        "Verbatim",
    ]
    .into_iter()
    .any(|name| is_system_symbol_name(head, name))
}

fn expr_contains_any_symbol(expr: &Expr, symbols: &[Symbol]) -> bool {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => symbols.contains(symbol),
        Expr::Atom(_) => false,
        Expr::Call { head, args } => {
            expr_contains_any_symbol(head, symbols)
                || args
                    .iter()
                    .any(|arg| expr_contains_any_symbol(arg, symbols))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn closure_template_matches_plain_rule_rhs() {
        let x = Symbol::from_name("x");
        let y = Symbol::from_name("y");
        let rhs = Expr::call(
            system_expr("f"),
            vec![
                Expr::Atom(Atom::Symbol(x)),
                Expr::call(system_expr("g"), vec![Expr::Atom(Atom::Symbol(y))]),
            ],
        );
        let mut bindings = SymbolExprMap::default();
        bindings.insert(x, Expr::integer(BigInt::from(1_u8)));
        bindings.insert(y, Expr::integer(BigInt::from(2_u8)));

        assert_compiled_body_matches_tree_walk(&rhs, &bindings);
    }

    #[test]
    fn closure_template_splices_bound_and_existing_sequences() {
        let x = Symbol::from_name("x");
        let rhs = Expr::call(
            system_expr("f"),
            vec![
                Expr::Atom(Atom::Symbol(x)),
                Expr::call(
                    system_expr("g"),
                    vec![sequence_expr(vec![
                        Expr::integer(BigInt::from(3_u8)),
                        Expr::integer(BigInt::from(4_u8)),
                    ])],
                ),
            ],
        );
        let mut bindings = SymbolExprMap::default();
        bindings.insert(
            x,
            sequence_expr(vec![
                Expr::integer(BigInt::from(1_u8)),
                Expr::integer(BigInt::from(2_u8)),
            ]),
        );

        assert_compiled_body_matches_tree_walk(&rhs, &bindings);
    }

    #[test]
    fn closure_template_keeps_nested_scopes_on_fallback_path() {
        let bindings = SymbolExprMap::default();
        let module_rhs = Expr::call(
            system_expr("f"),
            vec![Expr::call(
                system_expr("Module"),
                vec![
                    Expr::call(system_expr("List"), vec![Expr::symbol("x")]),
                    Expr::symbol("x"),
                ],
            )],
        );
        let with_rhs = Expr::call(
            system_expr("f"),
            vec![Expr::call(
                system_expr("With"),
                vec![
                    Expr::call(
                        system_expr("List"),
                        vec![Expr::call(
                            system_expr("Set"),
                            vec![Expr::symbol("x"), Expr::integer(BigInt::from(1_u8))],
                        )],
                    ),
                    Expr::symbol("x"),
                ],
            )],
        );
        let function_rhs = Expr::call(
            system_expr("Function"),
            vec![
                Expr::call(system_expr("List"), vec![Expr::symbol("x")]),
                Expr::symbol("x"),
            ],
        );

        assert!(compile_body(&module_rhs, &bindings).is_none());
        assert!(compile_body(&with_rhs, &bindings).is_none());
        assert!(compile_body(&function_rhs, &bindings).is_none());
    }

    #[test]
    fn closure_module_if_sameq_matches_tree_walk_when_branch_not_taken() {
        let x = Symbol::from_name("x");
        let rhs = module_if_sameq_rhs(
            Expr::call(
                system_expr("If"),
                vec![system_expr("True"), symbol_expr(&x), Expr::symbol("unused")],
            ),
            Expr::symbol("failed"),
            Expr::call(
                system_expr("If"),
                vec![
                    system_expr("True"),
                    Expr::symbol("fallback"),
                    Expr::symbol("unused"),
                ],
            ),
        );
        let mut bindings = SymbolExprMap::default();
        bindings.insert(x, Expr::symbol("ok"));

        assert_compiled_module_matches_tree_walk(&rhs, &bindings, "ok");
    }

    #[test]
    fn closure_module_if_sameq_matches_tree_walk_when_branch_taken() {
        let x = Symbol::from_name("x");
        let rhs = module_if_sameq_rhs(
            Expr::call(
                system_expr("If"),
                vec![system_expr("True"), symbol_expr(&x), Expr::symbol("unused")],
            ),
            Expr::symbol("failed"),
            Expr::call(
                system_expr("If"),
                vec![
                    system_expr("True"),
                    Expr::symbol("fallback"),
                    Expr::symbol("unused"),
                ],
            ),
        );
        let mut bindings = SymbolExprMap::default();
        bindings.insert(x, Expr::symbol("failed"));

        assert_compiled_module_matches_tree_walk(&rhs, &bindings, "fallback");
    }

    #[test]
    fn closure_module_rejects_held_local_escape() {
        let x = Symbol::from_name("x");
        let res = Symbol::from_name("res");
        let rhs = module_if_sameq_rhs(
            Expr::call(system_expr("Hold"), vec![symbol_expr(&res)]),
            Expr::symbol("failed"),
            Expr::call(Expr::symbol("fallback"), vec![symbol_expr(&x)]),
        );
        let mut bindings = SymbolExprMap::default();
        bindings.insert(x, Expr::symbol("value"));

        assert!(compile_body(&rhs, &bindings).is_none());
    }

    #[test]
    fn closure_repeatc_matches_tree_walk_when_failed_return_taken() {
        let (rhs, mut bindings) = repeatc_rhs_and_bindings("failConv", "start");
        assert_compiled_repeatc_matches_tree_walk(&rhs, &bindings, "refl[start]");

        bindings.insert(Symbol::from_name("c"), Expr::symbol("stableConv"));
        assert_compiled_repeatc_matches_tree_walk(&rhs, &bindings, "thm[start, start]");
    }

    #[test]
    fn closure_repeatc_matches_tree_walk_when_stable_return_taken() {
        let (rhs, bindings) = repeatc_rhs_and_bindings("stableConv", "start");
        assert_compiled_repeatc_matches_tree_walk(&rhs, &bindings, "thm[start, start]");
    }

    #[test]
    fn closure_repeatc_matches_tree_walk_when_recursive_fallthrough_runs() {
        let (rhs, bindings) = repeatc_rhs_and_bindings("stepConv", "start");
        assert_compiled_repeatc_matches_tree_walk(
            &rhs,
            &bindings,
            "transResult[thm[start, mid], refl[mid]]",
        );
    }

    #[test]
    fn closure_repeatc_rejects_held_local_read() {
        let c = Symbol::from_name("c");
        let t = Symbol::from_name("t");
        let mut bindings = SymbolExprMap::default();
        bindings.insert(c, Expr::symbol("stepConv"));
        bindings.insert(t, Expr::symbol("start"));
        let rhs = repeatc_rhs_with_final(
            &c,
            &t,
            Expr::call(system_expr("Hold"), vec![Expr::symbol("res")]),
        );

        assert!(compile_body(&rhs, &bindings).is_none());
    }

    #[test]
    fn closure_compiles_general_control_skeleton() {
        let x = Symbol::from_name("x");
        let rhs = Expr::call(
            system_expr("CompoundExpression"),
            vec![
                Expr::call(
                    system_expr("If"),
                    vec![
                        Expr::call(
                            system_expr("SameQ"),
                            vec![symbol_expr(&x), Expr::symbol("a")],
                        ),
                        Expr::symbol("Null"),
                    ],
                ),
                Expr::call(Expr::symbol("h"), vec![symbol_expr(&x)]),
            ],
        );
        let mut bindings = SymbolExprMap::default();
        bindings.insert(x, Expr::symbol("a"));

        assert_compiled_body_matches_tree_walk(&rhs, &bindings);
    }

    fn assert_compiled_body_matches_tree_walk(rhs: &Expr, bindings: &SymbolExprMap) {
        let code = Arc::new(compile_body(rhs, bindings).expect("RHS should compile"));
        let slot_values = slot_values_for_code(&code, bindings).expect("slots should prepare");
        let mut actual_session = vm_test_session();
        let actual = finish_compiled_definition_rhs_result(execute_compiled_body(
            &mut actual_session,
            PreparedCompiledRhsTemplate { code, slot_values },
            None,
        ))
        .expect("compiled body should evaluate");

        let substituted = substitute_rule_bindings_for_rhs(
            rhs,
            bindings,
            rule_rhs_may_splice_without_bindings(rhs),
        );
        let mut expected_session = vm_test_session();
        let expected = expected_session
            .evaluate_definition_rhs(None, substituted)
            .expect("tree-walk RHS should evaluate");

        assert_eq!(actual, expected);
    }

    fn assert_compiled_module_matches_tree_walk(
        rhs: &Expr,
        bindings: &SymbolExprMap,
        expected_text: &str,
    ) {
        assert_compiled_body_matches_tree_walk(rhs, bindings);
        let actual = eval_compiled_to_string(rhs, bindings);
        assert_eq!(actual, expected_text);
    }

    fn assert_compiled_repeatc_matches_tree_walk(
        rhs: &Expr,
        bindings: &SymbolExprMap,
        expected_text: &str,
    ) {
        let actual = eval_compiled_repeatc_to_string(rhs, bindings);
        let substituted = substitute_rule_bindings_for_rhs(
            rhs,
            bindings,
            rule_rhs_may_splice_without_bindings(rhs),
        );
        let mut expected_session = repeatc_test_session();
        let expected = expected_session
            .evaluate_definition_rhs(None, substituted)
            .expect("tree-walk REPEATC Module should evaluate");

        assert_eq!(actual, expected.to_string());
        assert_eq!(actual, expected_text);
    }

    fn eval_compiled_to_string(rhs: &Expr, bindings: &SymbolExprMap) -> String {
        let code = Arc::new(compile_body(rhs, bindings).expect("RHS should compile"));
        let slot_values = slot_values_for_code(&code, bindings).expect("slots should prepare");
        let mut session = vm_test_session();
        finish_compiled_definition_rhs_result(execute_compiled_body(
            &mut session,
            PreparedCompiledRhsTemplate { code, slot_values },
            None,
        ))
        .expect("compiled body should evaluate")
        .to_string()
    }

    fn eval_compiled_repeatc_to_string(rhs: &Expr, bindings: &SymbolExprMap) -> String {
        let code = Arc::new(compile_body(rhs, bindings).expect("REPEATC RHS should compile"));
        let slot_values = slot_values_for_code(&code, bindings).expect("slots should prepare");
        let mut session = repeatc_test_session();
        finish_compiled_definition_rhs_result(execute_compiled_body(
            &mut session,
            PreparedCompiledRhsTemplate { code, slot_values },
            None,
        ))
        .expect("compiled REPEATC body should evaluate")
        .to_string()
    }

    fn module_if_sameq_rhs(eval_expr: Expr, const_expr: Expr, alt_expr: Expr) -> Expr {
        let res = Symbol::from_name("res");
        Expr::call(
            system_expr("Module"),
            vec![
                Expr::call(system_expr("List"), vec![symbol_expr(&res)]),
                Expr::call(
                    system_expr("CompoundExpression"),
                    vec![
                        Expr::call(system_expr("Set"), vec![symbol_expr(&res), eval_expr]),
                        Expr::call(
                            system_expr("If"),
                            vec![
                                Expr::call(
                                    system_expr("SameQ"),
                                    vec![symbol_expr(&res), const_expr],
                                ),
                                alt_expr,
                                symbol_expr(&res),
                            ],
                        ),
                    ],
                ),
            ],
        )
    }

    fn repeatc_rhs_and_bindings(conv: &str, term: &str) -> (Expr, SymbolExprMap) {
        let c = Symbol::from_name("c");
        let t = Symbol::from_name("t");
        let mut bindings = SymbolExprMap::default();
        bindings.insert(c, Expr::symbol(conv));
        bindings.insert(t, Expr::symbol(term));
        (repeatc_rhs(&c, &t), bindings)
    }

    fn repeatc_rhs(c: &Symbol, t: &Symbol) -> Expr {
        let res = Symbol::from_name("res");
        let rhs = Symbol::from_name("rhs");
        let final_expr = Expr::call(
            Expr::symbol("trans"),
            vec![
                symbol_expr(&res),
                Expr::call(
                    Expr::call(Expr::symbol("repeatc"), vec![symbol_expr(c)]),
                    vec![symbol_expr(&rhs)],
                ),
            ],
        );
        repeatc_rhs_with_final(c, t, final_expr)
    }

    fn repeatc_rhs_with_final(c: &Symbol, t: &Symbol, final_expr: Expr) -> Expr {
        let res = Symbol::from_name("res");
        let rhs = Symbol::from_name("rhs");
        Expr::call(
            system_expr("Module"),
            vec![
                Expr::call(
                    system_expr("List"),
                    vec![symbol_expr(&res), symbol_expr(&rhs)],
                ),
                Expr::call(
                    system_expr("CompoundExpression"),
                    vec![
                        Expr::call(
                            system_expr("Set"),
                            vec![
                                symbol_expr(&res),
                                Expr::call(
                                    Expr::symbol("try"),
                                    vec![Expr::call(symbol_expr(c), vec![symbol_expr(t)])],
                                ),
                            ],
                        ),
                        Expr::call(
                            system_expr("If"),
                            vec![
                                Expr::call(
                                    system_expr("SameQ"),
                                    vec![symbol_expr(&res), Expr::symbol("failed")],
                                ),
                                Expr::call(
                                    system_expr("Return"),
                                    vec![Expr::call(Expr::symbol("refl"), vec![symbol_expr(t)])],
                                ),
                            ],
                        ),
                        Expr::call(
                            system_expr("Set"),
                            vec![
                                symbol_expr(&rhs),
                                Expr::call(Expr::symbol("rhsOf"), vec![symbol_expr(&res)]),
                            ],
                        ),
                        Expr::call(
                            system_expr("If"),
                            vec![
                                Expr::call(
                                    system_expr("SameQ"),
                                    vec![symbol_expr(&rhs), symbol_expr(t)],
                                ),
                                Expr::call(system_expr("Return"), vec![symbol_expr(&res)]),
                            ],
                        ),
                        final_expr,
                    ],
                ),
            ],
        )
    }

    fn repeatc_test_session() -> Session {
        let mut session = vm_test_session();
        session
            .eval_text(
                "\
                ClearAll[try, failConv, stableConv, stepConv, rhsOf, trans, repeatc];\
                try[x_] := x;\
                failConv[x_] := failed;\
                stableConv[x_] := thm[x, x];\
                stepConv[start] := thm[start, mid];\
                stepConv[mid] := failed;\
                rhsOf[thm[_, r_]] := r;\
                trans[a_, b_] := transResult[a, b];\
                repeatc[c_][t_] := Module[{res, rhs},\
                  res = try[c[t]];\
                  If[res === failed, Return[refl[t]]];\
                  rhs = rhsOf[res];\
                  If[SameQ[rhs, t], Return[res]];\
                  trans[res, repeatc[c][rhs]]\
                ]",
            )
            .expect("repeatc test definitions should install");
        session
    }

    fn vm_test_session() -> Session {
        let mut session = Session::new();
        session.register_builtin(BuiltinSpec::ordinary(
            "SameQ",
            "Tests whether all arguments are exactly identical after evaluation.",
            test_eval_same_q,
        ));
        session
    }

    fn test_eval_same_q(
        _session: &mut Session,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        Ok(system_expr(
            if args.windows(2).all(|pair| pair[0] == pair[1]) {
                heads::structure::TRUE
            } else {
                heads::structure::FALSE
            },
        ))
    }

    fn symbol_expr(symbol: &Symbol) -> Expr {
        Expr::Atom(Atom::Symbol(*symbol))
    }
}
