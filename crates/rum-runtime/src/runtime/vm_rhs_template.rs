use super::*;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

const RHS_TEMPLATE_ENV: &str = "RUM_VM_RHS_TEMPLATE";

static RHS_TEMPLATE_ENABLED: AtomicU8 = AtomicU8::new(0);
static RHS_TEMPLATE_COMPILED_DISTINCT: AtomicU64 = AtomicU64::new(0);
static RHS_TEMPLATE_FALLBACK_DISTINCT: AtomicU64 = AtomicU64::new(0);
static RHS_TEMPLATE_MODULE_IF_SAMEQ_DISTINCT: AtomicU64 = AtomicU64::new(0);
static RHS_TEMPLATE_MODULE_REPEATC_DISTINCT: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug)]
enum Op {
    PushConst(SharedExpr),
    PushBinding(usize),
    PushLocal(usize),
    Construct { argc: usize },
    Eval,
}

#[derive(Clone, Debug)]
enum CodeProgram {
    Construct {
        ops: Vec<Op>,
    },
    ModuleIfSameQ {
        rhs_ops: Vec<Op>,
        const_ops: Vec<Op>,
        alt_ops: Vec<Op>,
    },
    ModuleRepeatC {
        res_rhs_ops: Vec<Op>,
        failed_const_ops: Vec<Op>,
        failed_return_ops: Vec<Op>,
        rhs_rhs_ops: Vec<Op>,
        stable_condition_ops: Vec<Op>,
        stable_return_ops: Vec<Op>,
        final_ops: Vec<Op>,
    },
}

#[derive(Clone, Debug)]
struct CodeObject {
    source: Expr,
    program: CodeProgram,
    slots: Vec<Symbol>,
    const_symbols: Vec<Symbol>,
}

#[derive(Clone, Debug)]
pub(super) enum PreparedRhsTemplate {
    Substituted(Expr),
    CompiledModule(PreparedCompiledRhsTemplate),
}

#[derive(Clone, Debug)]
pub(super) struct PreparedCompiledRhsTemplate {
    code: CodeObject,
    slot_values: Vec<Expr>,
}

#[derive(Clone, Debug)]
enum CacheEntry {
    Compiled(CodeObject),
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

#[derive(Clone, Copy)]
enum CompiledTemplateKind {
    Construct,
    ModuleIfSameQ,
    ModuleRepeatC,
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
        PreparedRhsTemplate::CompiledModule(_) if include_payload => {
            substitute_rule_bindings_for_rhs(original, bindings, may_splice_without_bindings)
        }
        PreparedRhsTemplate::CompiledModule(_) => original.clone(),
    }
}

pub(super) fn evaluate_prepared_rhs_template(
    session: &mut Session,
    prepared: PreparedRhsTemplate,
    option_scope: Option<OptionValueScope>,
) -> Result<Expr, EvalError> {
    match prepared {
        PreparedRhsTemplate::Substituted(rhs) => session.evaluate_definition_rhs(option_scope, rhs),
        PreparedRhsTemplate::CompiledModule(prepared) => {
            let result = execute_compiled_module_template(session, prepared, option_scope);
            let outcome = session.suspend_with_result_frame(result, |_, outcome| {
                finish_compiled_definition_rhs_result(outcome)
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
        match &code.program {
            CodeProgram::Construct { .. } => {
                if let Some(result) = execute_rhs_template(&code, bindings) {
                    #[cfg(debug_assertions)]
                    {
                        let expected = substitute_rule_bindings_for_rhs(
                            expr,
                            bindings,
                            may_splice_without_bindings,
                        );
                        debug_assert_eq!(
                            result, expected,
                            "RHS template diverged from tree substitution"
                        );
                    }
                    return PreparedRhsTemplate::Substituted(result);
                }
            }
            CodeProgram::ModuleIfSameQ { .. } | CodeProgram::ModuleRepeatC { .. } => {
                if let Some(prepared) = prepare_compiled_module_template(code, bindings) {
                    return PreparedRhsTemplate::CompiledModule(prepared);
                }
            }
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
) -> Option<CodeObject> {
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

    let compiled = compile_rhs_template(expr, bindings);
    match compiled {
        Some(code) => {
            let kind = compiled_template_kind(&code.program);
            cache
                .entries
                .insert(key.clone(), CacheEntry::Compiled(code.clone()));
            record_distinct_template_compile(&mut cache, key, kind);
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
    kind: CompiledTemplateKind,
) {
    if cache.reported_compiled.insert(key) {
        let compiled = RHS_TEMPLATE_COMPILED_DISTINCT.fetch_add(1, Ordering::Relaxed) + 1;
        let fallback = RHS_TEMPLATE_FALLBACK_DISTINCT.load(Ordering::Relaxed);
        let module_if_sameq = if matches!(kind, CompiledTemplateKind::ModuleIfSameQ) {
            RHS_TEMPLATE_MODULE_IF_SAMEQ_DISTINCT.fetch_add(1, Ordering::Relaxed) + 1
        } else {
            RHS_TEMPLATE_MODULE_IF_SAMEQ_DISTINCT.load(Ordering::Relaxed)
        };
        let module_repeatc = if matches!(kind, CompiledTemplateKind::ModuleRepeatC) {
            RHS_TEMPLATE_MODULE_REPEATC_DISTINCT.fetch_add(1, Ordering::Relaxed) + 1
        } else {
            RHS_TEMPLATE_MODULE_REPEATC_DISTINCT.load(Ordering::Relaxed)
        };
        eprintln!(
            "rum-vm-rhs-template coverage compiled_distinct={compiled} fallback_distinct={fallback} module_if_sameq_distinct={module_if_sameq} module_repeatc_distinct={module_repeatc}"
        );
    }
}

fn record_distinct_template_fallback(cache: &mut TemplateCache, key: RhsTemplateKey) {
    if cache.reported_fallback.insert(key) {
        let fallback = RHS_TEMPLATE_FALLBACK_DISTINCT.fetch_add(1, Ordering::Relaxed) + 1;
        let compiled = RHS_TEMPLATE_COMPILED_DISTINCT.load(Ordering::Relaxed);
        let module_if_sameq = RHS_TEMPLATE_MODULE_IF_SAMEQ_DISTINCT.load(Ordering::Relaxed);
        let module_repeatc = RHS_TEMPLATE_MODULE_REPEATC_DISTINCT.load(Ordering::Relaxed);
        eprintln!(
            "rum-vm-rhs-template coverage compiled_distinct={compiled} fallback_distinct={fallback} module_if_sameq_distinct={module_if_sameq} module_repeatc_distinct={module_repeatc}"
        );
    }
}

fn compiled_template_kind(program: &CodeProgram) -> CompiledTemplateKind {
    match program {
        CodeProgram::Construct { .. } => CompiledTemplateKind::Construct,
        CodeProgram::ModuleIfSameQ { .. } => CompiledTemplateKind::ModuleIfSameQ,
        CodeProgram::ModuleRepeatC { .. } => CompiledTemplateKind::ModuleRepeatC,
    }
}

fn compile_rhs_template(expr: &Expr, bindings: &SymbolExprMap) -> Option<CodeObject> {
    let mut module_compiler = RhsTemplateCompiler::new(bindings);
    if let Some(program) = module_compiler.compile_module_repeatc(expr) {
        return Some(CodeObject {
            source: expr.clone(),
            program,
            slots: module_compiler.slots,
            const_symbols: module_compiler.const_symbols,
        });
    }

    let mut module_compiler = RhsTemplateCompiler::new(bindings);
    if let Some(program) = module_compiler.compile_module_if_sameq(expr) {
        return Some(CodeObject {
            source: expr.clone(),
            program,
            slots: module_compiler.slots,
            const_symbols: module_compiler.const_symbols,
        });
    }

    let mut compiler = RhsTemplateCompiler::new(bindings);
    compiler.compile_expr(expr)?;
    Some(CodeObject {
        source: expr.clone(),
        program: CodeProgram::Construct { ops: compiler.ops },
        slots: compiler.slots,
        const_symbols: compiler.const_symbols,
    })
}

struct RhsTemplateCompiler<'a> {
    bindings: &'a SymbolExprMap,
    ops: Vec<Op>,
    slots: Vec<Symbol>,
    const_symbols: Vec<Symbol>,
    local_symbols: Vec<Symbol>,
}

impl<'a> RhsTemplateCompiler<'a> {
    fn new(bindings: &'a SymbolExprMap) -> Self {
        Self {
            bindings,
            ops: Vec::new(),
            slots: Vec::new(),
            const_symbols: Vec::new(),
            local_symbols: Vec::new(),
        }
    }

    fn compile_module_repeatc(&mut self, expr: &Expr) -> Option<CodeProgram> {
        let parts = module_repeatc_parts(expr)?;
        if self.bindings.contains_key(&parts.res_local)
            || self.bindings.contains_key(&parts.rhs_local)
        {
            return None;
        }

        let locals = [parts.res_local, parts.rhs_local];
        if expr_contains_any_symbol(parts.res_rhs, &locals)
            || expr_contains_any_symbol(parts.failed_const, &locals)
            || expr_contains_any_symbol(parts.failed_return, &locals)
            || !local_reads_are_register_safe(parts.rhs_rhs, &locals)
            || !local_reads_are_register_safe(parts.stable_condition, &locals)
            || !local_reads_are_register_safe(parts.stable_return, &locals)
            || !local_reads_are_register_safe(parts.final_expr, &locals)
            || repeatc_part_contains_nested_return(&parts)
        {
            return None;
        }

        self.local_symbols = locals.to_vec();

        let mut res_rhs_ops = self.compile_expr_to_ops(parts.res_rhs)?;
        res_rhs_ops.push(Op::Eval);
        let failed_const_ops = self.compile_expr_to_ops(parts.failed_const)?;
        let mut failed_return_ops = self.compile_expr_to_ops(parts.failed_return)?;
        failed_return_ops.push(Op::Eval);
        let mut rhs_rhs_ops = self.compile_expr_to_ops(parts.rhs_rhs)?;
        rhs_rhs_ops.push(Op::Eval);
        let mut stable_condition_ops = self.compile_expr_to_ops(parts.stable_condition)?;
        stable_condition_ops.push(Op::Eval);
        let mut stable_return_ops = self.compile_expr_to_ops(parts.stable_return)?;
        stable_return_ops.push(Op::Eval);
        let mut final_ops = self.compile_expr_to_ops(parts.final_expr)?;
        final_ops.push(Op::Eval);

        Some(CodeProgram::ModuleRepeatC {
            res_rhs_ops,
            failed_const_ops,
            failed_return_ops,
            rhs_rhs_ops,
            stable_condition_ops,
            stable_return_ops,
            final_ops,
        })
    }

    fn compile_module_if_sameq(&mut self, expr: &Expr) -> Option<CodeProgram> {
        let parts = module_if_sameq_parts(expr)?;
        if self.is_top_level_bound_symbol(parts.rhs)
            || self.is_top_level_bound_symbol(parts.const_expr)
            || self.is_top_level_bound_symbol(parts.alt)
        {
            return None;
        }

        let mut rhs_ops = self.compile_expr_to_ops(parts.rhs)?;
        rhs_ops.push(Op::Eval);
        let const_ops = self.compile_expr_to_ops(parts.const_expr)?;
        let mut alt_ops = self.compile_expr_to_ops(parts.alt)?;
        alt_ops.push(Op::Eval);

        Some(CodeProgram::ModuleIfSameQ {
            rhs_ops,
            const_ops,
            alt_ops,
        })
    }

    fn compile_expr_to_ops(&mut self, expr: &Expr) -> Option<Vec<Op>> {
        let start = self.ops.len();
        self.compile_expr(expr)?;
        Some(self.ops.split_off(start))
    }

    fn compile_expr(&mut self, expr: &Expr) -> Option<()> {
        if rhs_contains_uncompiled_scope(expr) {
            return None;
        }

        if self.can_push_const_subtree(expr) {
            self.collect_const_symbols(expr);
            self.ops.push(Op::PushConst(SharedExpr::from(expr.clone())));
            return Some(());
        }

        match expr {
            Expr::Atom(Atom::Symbol(symbol)) if self.local_slot(*symbol).is_some() => {
                let slot = self
                    .local_slot(*symbol)
                    .expect("checked local slot should still exist");
                self.ops.push(Op::PushLocal(slot));
                Some(())
            }
            Expr::Atom(Atom::Symbol(symbol)) if self.bindings.contains_key(symbol) => {
                let slot = self.binding_slot(*symbol);
                self.ops.push(Op::PushBinding(slot));
                Some(())
            }
            Expr::Atom(Atom::Symbol(symbol)) => {
                self.record_const_symbol(*symbol);
                self.ops.push(Op::PushConst(SharedExpr::from(expr.clone())));
                Some(())
            }
            Expr::Atom(_) => {
                self.ops.push(Op::PushConst(SharedExpr::from(expr.clone())));
                Some(())
            }
            Expr::Call { head, args } => {
                self.compile_expr(head)?;
                for arg in args.iter() {
                    self.compile_expr(arg)?;
                }
                self.ops.push(Op::Construct { argc: args.len() });
                Some(())
            }
        }
    }

    fn is_top_level_bound_symbol(&self, expr: &Expr) -> bool {
        matches!(expr, Expr::Atom(Atom::Symbol(symbol)) if self.bindings.contains_key(symbol))
    }

    fn can_push_const_subtree(&self, expr: &Expr) -> bool {
        !expr_contains_bound_symbol(expr, self.bindings)
            && !expr_contains_any_symbol(expr, &self.local_symbols)
            && !rule_rhs_may_splice_without_bindings(expr)
    }

    fn local_slot(&self, symbol: Symbol) -> Option<usize> {
        self.local_symbols.iter().position(|slot| *slot == symbol)
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

    fn collect_const_symbols(&mut self, expr: &Expr) {
        match expr {
            Expr::Atom(Atom::Symbol(symbol)) => self.record_const_symbol(*symbol),
            Expr::Atom(_) => {}
            Expr::Call { head, args } => {
                self.collect_const_symbols(head);
                for arg in args.iter() {
                    self.collect_const_symbols(arg);
                }
            }
        }
    }

    fn record_const_symbol(&mut self, symbol: Symbol) {
        if !self.const_symbols.contains(&symbol) {
            self.const_symbols.push(symbol);
        }
    }
}

struct ModuleIfSameQParts<'a> {
    rhs: &'a Expr,
    const_expr: &'a Expr,
    alt: &'a Expr,
}

struct ModuleRepeatCParts<'a> {
    res_local: Symbol,
    rhs_local: Symbol,
    res_rhs: &'a Expr,
    failed_const: &'a Expr,
    failed_return: &'a Expr,
    rhs_rhs: &'a Expr,
    stable_condition: &'a Expr,
    stable_return: &'a Expr,
    final_expr: &'a Expr,
}

fn module_repeatc_parts(expr: &Expr) -> Option<ModuleRepeatCParts<'_>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, "Module") || args.len() != 2 {
        return None;
    }

    let locals = parse_module_binding_list(&args[0])?;
    let [(res_local, res_init), (rhs_local, rhs_init)] = locals.as_slice() else {
        return None;
    };
    if res_init.is_some() || rhs_init.is_some() || res_local == rhs_local {
        return None;
    }

    let items = compound_expression_items(&args[1])?;
    let [set_res, failed_if, set_rhs, stable_if, final_expr] = items else {
        return None;
    };

    let (set_res_lhs, res_rhs) = set_assignment_parts(set_res)?;
    if set_res_lhs != res_local {
        return None;
    }

    let (failed_condition, failed_return_call) = if_two_arg_parts(failed_if)?;
    let (failed_lhs, failed_const) = sameq_two_arg_parts(failed_condition)?;
    if failed_lhs != res_local {
        return None;
    }
    let failed_return = return_one_arg_parts(failed_return_call)?;

    let (set_rhs_lhs, rhs_rhs) = set_assignment_parts(set_rhs)?;
    if set_rhs_lhs != rhs_local {
        return None;
    }

    let (stable_condition, stable_return_call) = if_two_arg_parts(stable_if)?;
    let stable_return = return_one_arg_parts(stable_return_call)?;

    Some(ModuleRepeatCParts {
        res_local: *res_local,
        rhs_local: *rhs_local,
        res_rhs,
        failed_const,
        failed_return,
        rhs_rhs,
        stable_condition,
        stable_return,
        final_expr,
    })
}

fn module_if_sameq_parts(expr: &Expr) -> Option<ModuleIfSameQParts<'_>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, "Module") || args.len() != 2 {
        return None;
    }

    let locals = parse_module_binding_list(&args[0])?;
    let [(local, init)] = locals.as_slice() else {
        return None;
    };
    if init.is_some() {
        return None;
    }

    let (set_expr, if_expr) = compound_expression_pair(&args[1])?;
    let (set_lhs, rhs) = set_assignment_parts(set_expr)?;
    if set_lhs != local {
        return None;
    }

    let (condition, alt, else_branch) = if_three_arg_parts(if_expr)?;
    if expr_as_symbol(else_branch) != Some(local) {
        return None;
    }

    let (same_lhs, const_expr) = sameq_two_arg_parts(condition)?;
    if same_lhs != local {
        return None;
    }

    if expr_contains_symbol(rhs, local)
        || expr_contains_symbol(const_expr, local)
        || expr_contains_symbol(alt, local)
    {
        return None;
    }

    Some(ModuleIfSameQParts {
        rhs,
        const_expr,
        alt,
    })
}

fn compound_expression_pair(expr: &Expr) -> Option<(&Expr, &Expr)> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if is_system_symbol_name(head, "CompoundExpression") && args.len() == 2 {
        Some((&args[0], &args[1]))
    } else {
        None
    }
}

fn compound_expression_items(expr: &Expr) -> Option<&[Expr]> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if is_system_symbol_name(head, "CompoundExpression") {
        Some(args.as_slice())
    } else {
        None
    }
}

fn set_assignment_parts(expr: &Expr) -> Option<(&Symbol, &Expr)> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if is_system_symbol_name(head, "Set") && args.len() == 2 {
        Some((expr_as_symbol(&args[0])?, &args[1]))
    } else {
        None
    }
}

fn if_two_arg_parts(expr: &Expr) -> Option<(&Expr, &Expr)> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if is_system_symbol_name(head, "If") && args.len() == 2 {
        Some((&args[0], &args[1]))
    } else {
        None
    }
}

fn if_three_arg_parts(expr: &Expr) -> Option<(&Expr, &Expr, &Expr)> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if is_system_symbol_name(head, "If") && args.len() == 3 {
        Some((&args[0], &args[1], &args[2]))
    } else {
        None
    }
}

fn return_one_arg_parts(expr: &Expr) -> Option<&Expr> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if is_system_symbol_name(head, "Return") && args.len() == 1 {
        Some(&args[0])
    } else {
        None
    }
}

fn sameq_two_arg_parts(expr: &Expr) -> Option<(&Symbol, &Expr)> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if is_system_symbol_name(head, "SameQ") && args.len() == 2 {
        Some((expr_as_symbol(&args[0])?, &args[1]))
    } else {
        None
    }
}

fn rhs_contains_uncompiled_scope(expr: &Expr) -> bool {
    match expr {
        Expr::Call { head, args } => {
            (is_system_symbol_name(head, "Module") && args.len() == 2)
                || (is_system_symbol_name(head, "With") && args.len() == 2)
                || is_system_symbol_name(head, "Function")
                || rhs_contains_uncompiled_scope(head)
                || args.iter().any(rhs_contains_uncompiled_scope)
        }
        Expr::Atom(_) => false,
    }
}

fn expr_contains_bound_symbol(expr: &Expr, bindings: &SymbolExprMap) -> bool {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => bindings.contains_key(symbol),
        Expr::Atom(_) => false,
        Expr::Call { head, args } => {
            expr_contains_bound_symbol(head, bindings)
                || args
                    .iter()
                    .any(|arg| expr_contains_bound_symbol(arg, bindings))
        }
    }
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

fn expr_contains_system_head(expr: &Expr, name: &str) -> bool {
    match expr {
        Expr::Call { head, args } => {
            is_system_symbol_name(head, name)
                || expr_contains_system_head(head, name)
                || args.iter().any(|arg| expr_contains_system_head(arg, name))
        }
        Expr::Atom(_) => false,
    }
}

fn repeatc_part_contains_nested_return(parts: &ModuleRepeatCParts<'_>) -> bool {
    [
        parts.res_rhs,
        parts.failed_const,
        parts.failed_return,
        parts.rhs_rhs,
        parts.stable_condition,
        parts.stable_return,
        parts.final_expr,
    ]
    .into_iter()
    .any(|expr| expr_contains_system_head(expr, "Return"))
}

fn local_reads_are_register_safe(expr: &Expr, locals: &[Symbol]) -> bool {
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
            local_reads_are_register_safe(head, locals)
                && args
                    .iter()
                    .all(|arg| local_reads_are_register_safe(arg, locals))
        }
    }
}

fn head_blocks_registerized_locals(head: &Expr) -> bool {
    [
        "CompoundExpression",
        "Condition",
        "Defer",
        "Function",
        "Hold",
        "HoldComplete",
        "HoldForm",
        "If",
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

fn code_compatible_with_bindings(code: &CodeObject, bindings: &SymbolExprMap) -> bool {
    code.slots
        .iter()
        .all(|symbol| bindings.contains_key(symbol))
        && code
            .const_symbols
            .iter()
            .all(|symbol| !bindings.contains_key(symbol))
}

fn execute_rhs_template(code: &CodeObject, bindings: &SymbolExprMap) -> Option<Expr> {
    let CodeProgram::Construct { ops } = &code.program else {
        return None;
    };
    let slot_values = slot_values_for_code(code, bindings)?;
    execute_construct_ops(ops, &slot_values)
}

fn prepare_compiled_module_template(
    code: CodeObject,
    bindings: &SymbolExprMap,
) -> Option<PreparedCompiledRhsTemplate> {
    if !matches!(
        code.program,
        CodeProgram::ModuleIfSameQ { .. } | CodeProgram::ModuleRepeatC { .. }
    ) {
        return None;
    }
    let slot_values = slot_values_for_code(&code, bindings)?;
    Some(PreparedCompiledRhsTemplate { code, slot_values })
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

fn execute_construct_ops(ops: &[Op], slot_values: &[Expr]) -> Option<Expr> {
    let mut stack = Vec::new();
    for op in ops {
        match op {
            Op::PushConst(expr) => stack.push(expr.as_ref().clone()),
            Op::PushBinding(slot) => stack.push(slot_values.get(*slot)?.clone()),
            Op::Construct { argc } => {
                apply_construct_op(&mut stack, *argc)?;
            }
            Op::PushLocal(_) | Op::Eval => return None,
        }
    }

    if stack.len() == 1 { stack.pop() } else { None }
}

fn execute_compiled_module_template(
    session: &mut Session,
    prepared: PreparedCompiledRhsTemplate,
    option_scope: Option<OptionValueScope>,
) -> Result<Expr, EvalError> {
    match &prepared.code.program {
        CodeProgram::ModuleIfSameQ { .. } => {
            execute_compiled_module_if_sameq_template(session, prepared, option_scope)
        }
        CodeProgram::ModuleRepeatC { .. } => {
            execute_compiled_module_repeatc_template(session, prepared, option_scope)
        }
        CodeProgram::Construct { .. } => Ok(prepared.code.source),
    }
}

fn execute_compiled_module_if_sameq_template(
    session: &mut Session,
    prepared: PreparedCompiledRhsTemplate,
    option_scope: Option<OptionValueScope>,
) -> Result<Expr, EvalError> {
    let CodeProgram::ModuleIfSameQ { rhs_ops, .. } = &prepared.code.program else {
        return Ok(prepared.code.source);
    };
    let rhs_result = execute_vm_ops(
        session,
        rhs_ops.clone(),
        prepared.slot_values.clone(),
        Vec::new(),
        option_scope.clone(),
        Vec::new(),
        0,
    );
    let value = session.suspend_with_frame_lazy(rhs_result, || {
        let prepared = prepared.clone();
        let option_scope = option_scope.clone();
        move |session, value| {
            resume_compiled_module_after_rhs(session, prepared, option_scope, value)
        }
    })?;

    resume_compiled_module_after_rhs(session, prepared, option_scope, value)
}

fn resume_compiled_module_after_rhs(
    session: &mut Session,
    prepared: PreparedCompiledRhsTemplate,
    option_scope: Option<OptionValueScope>,
    value: Expr,
) -> Result<Expr, EvalError> {
    let CodeProgram::ModuleIfSameQ {
        const_ops, alt_ops, ..
    } = &prepared.code.program
    else {
        return Ok(value);
    };

    let const_expr = execute_construct_ops(const_ops, &prepared.slot_values)
        .expect("compiled SameQ constant ops should be stack-valid");
    if value != const_expr {
        return Ok(value);
    }

    execute_vm_ops(
        session,
        alt_ops.clone(),
        prepared.slot_values,
        Vec::new(),
        option_scope,
        Vec::new(),
        0,
    )
}

fn execute_compiled_module_repeatc_template(
    session: &mut Session,
    prepared: PreparedCompiledRhsTemplate,
    option_scope: Option<OptionValueScope>,
) -> Result<Expr, EvalError> {
    let CodeProgram::ModuleRepeatC { res_rhs_ops, .. } = &prepared.code.program else {
        return Ok(prepared.code.source);
    };
    let res_result = execute_vm_ops(
        session,
        res_rhs_ops.clone(),
        prepared.slot_values.clone(),
        repeatc_local_values(None, None),
        option_scope.clone(),
        Vec::new(),
        0,
    );
    let res = session.suspend_with_frame_lazy(res_result, || {
        let prepared = prepared.clone();
        let option_scope = option_scope.clone();
        move |session, res| resume_repeatc_after_res(session, prepared, option_scope, res)
    })?;

    resume_repeatc_after_res(session, prepared, option_scope, res)
}

fn resume_repeatc_after_res(
    session: &mut Session,
    prepared: PreparedCompiledRhsTemplate,
    option_scope: Option<OptionValueScope>,
    res: Expr,
) -> Result<Expr, EvalError> {
    let CodeProgram::ModuleRepeatC {
        failed_const_ops,
        failed_return_ops,
        ..
    } = &prepared.code.program
    else {
        return Ok(res);
    };
    let local_values = repeatc_local_values(Some(res.clone()), None);
    let failed_const = execute_construct_ops(failed_const_ops, &prepared.slot_values)
        .expect("compiled REPEATC failed constant ops should be stack-valid");
    if res == failed_const {
        return execute_vm_ops(
            session,
            failed_return_ops.clone(),
            prepared.slot_values,
            local_values,
            option_scope,
            Vec::new(),
            0,
        );
    }

    execute_repeatc_rhs_statement(session, prepared, option_scope, local_values)
}

fn execute_repeatc_rhs_statement(
    session: &mut Session,
    prepared: PreparedCompiledRhsTemplate,
    option_scope: Option<OptionValueScope>,
    local_values: Vec<Option<Expr>>,
) -> Result<Expr, EvalError> {
    let CodeProgram::ModuleRepeatC { rhs_rhs_ops, .. } = &prepared.code.program else {
        return Ok(system_expr("Null"));
    };
    let rhs_result = execute_vm_ops(
        session,
        rhs_rhs_ops.clone(),
        prepared.slot_values.clone(),
        local_values.clone(),
        option_scope.clone(),
        Vec::new(),
        0,
    );
    let rhs = session.suspend_with_frame_lazy(rhs_result, || {
        let prepared = prepared.clone();
        let option_scope = option_scope.clone();
        let local_values = local_values.clone();
        move |session, rhs| {
            resume_repeatc_after_rhs(session, prepared, option_scope, local_values, rhs)
        }
    })?;

    resume_repeatc_after_rhs(session, prepared, option_scope, local_values, rhs)
}

fn resume_repeatc_after_rhs(
    session: &mut Session,
    prepared: PreparedCompiledRhsTemplate,
    option_scope: Option<OptionValueScope>,
    mut local_values: Vec<Option<Expr>>,
    rhs: Expr,
) -> Result<Expr, EvalError> {
    local_values[1] = Some(rhs);
    execute_repeatc_stable_condition(session, prepared, option_scope, local_values)
}

fn execute_repeatc_stable_condition(
    session: &mut Session,
    prepared: PreparedCompiledRhsTemplate,
    option_scope: Option<OptionValueScope>,
    local_values: Vec<Option<Expr>>,
) -> Result<Expr, EvalError> {
    let CodeProgram::ModuleRepeatC {
        stable_condition_ops,
        ..
    } = &prepared.code.program
    else {
        return Ok(system_expr("Null"));
    };
    let condition_result = execute_vm_ops(
        session,
        stable_condition_ops.clone(),
        prepared.slot_values.clone(),
        local_values.clone(),
        option_scope.clone(),
        Vec::new(),
        0,
    );
    let condition = session.suspend_with_frame_lazy(condition_result, || {
        let prepared = prepared.clone();
        let option_scope = option_scope.clone();
        let local_values = local_values.clone();
        move |session, condition| {
            resume_repeatc_after_stable_condition(
                session,
                prepared,
                option_scope,
                local_values,
                condition,
            )
        }
    })?;

    resume_repeatc_after_stable_condition(session, prepared, option_scope, local_values, condition)
}

fn resume_repeatc_after_stable_condition(
    session: &mut Session,
    prepared: PreparedCompiledRhsTemplate,
    option_scope: Option<OptionValueScope>,
    local_values: Vec<Option<Expr>>,
    condition: Expr,
) -> Result<Expr, EvalError> {
    let CodeProgram::ModuleRepeatC {
        stable_return_ops,
        final_ops,
        ..
    } = &prepared.code.program
    else {
        return Ok(condition);
    };
    let ops = if truthiness_symbol(&condition) == Some(true) {
        stable_return_ops.clone()
    } else {
        final_ops.clone()
    };

    execute_vm_ops(
        session,
        ops,
        prepared.slot_values,
        local_values,
        option_scope,
        Vec::new(),
        0,
    )
}

fn repeatc_local_values(res: Option<Expr>, rhs: Option<Expr>) -> Vec<Option<Expr>> {
    vec![res, rhs]
}

fn execute_vm_ops(
    session: &mut Session,
    ops: Vec<Op>,
    slot_values: Vec<Expr>,
    local_values: Vec<Option<Expr>>,
    option_scope: Option<OptionValueScope>,
    mut stack: Vec<Expr>,
    mut ip: usize,
) -> Result<Expr, EvalError> {
    while let Some(op) = ops.get(ip).cloned() {
        match op {
            Op::PushConst(expr) => stack.push(expr.as_ref().clone()),
            Op::PushBinding(slot) => {
                stack.push(
                    slot_values
                        .get(slot)
                        .cloned()
                        .expect("compiled RHS binding slot should exist"),
                );
            }
            Op::PushLocal(slot) => {
                stack.push(
                    local_values
                        .get(slot)
                        .and_then(|value| value.clone())
                        .expect("compiled RHS local slot should be initialized"),
                );
            }
            Op::Construct { argc } => {
                apply_construct_op(&mut stack, argc)
                    .expect("compiled RHS construct ops should be stack-valid");
            }
            Op::Eval => {
                let expr = stack
                    .pop()
                    .expect("compiled RHS Eval should have one operand");
                let result = session.evaluate_with_option_value_scope(option_scope.clone(), expr);
                let value = session.suspend_with_frame_lazy(result, || {
                    let ops = ops.clone();
                    let slot_values = slot_values.clone();
                    let local_values = local_values.clone();
                    let option_scope = option_scope.clone();
                    let stack = stack.clone();
                    move |session, value| {
                        let mut stack = stack;
                        stack.push(value);
                        execute_vm_ops(
                            session,
                            ops,
                            slot_values,
                            local_values,
                            option_scope,
                            stack,
                            ip + 1,
                        )
                    }
                })?;
                stack.push(value);
            }
        }
        ip += 1;
    }

    debug_assert_eq!(stack.len(), 1, "compiled RHS ops should leave one value");
    Ok(stack
        .pop()
        .expect("compiled RHS ops should leave a result value"))
}

fn apply_construct_op(stack: &mut Vec<Expr>, argc: usize) -> Option<()> {
    if stack.len() < argc.saturating_add(1) {
        return None;
    }
    let args_start = stack.len() - argc;
    let args = stack.split_off(args_start);
    let head = stack.pop()?;
    stack.push(Expr::call(head, splice_sequence_args(args)));
    Some(())
}

fn splice_sequence_args(args: Vec<Expr>) -> Vec<Expr> {
    let mut output = Vec::with_capacity(args.len());
    for arg in args {
        if let Some(items) = sequence_items(&arg) {
            output.extend(items.iter().cloned());
        } else {
            output.push(arg);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rhs_template_matches_plain_binding_substitution() {
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

        assert_template_matches_tree_walk(&rhs, &bindings);
    }

    #[test]
    fn rhs_template_splices_bound_and_existing_sequences() {
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

        assert_template_matches_tree_walk(&rhs, &bindings);
    }

    #[test]
    fn rhs_template_keeps_scoped_forms_on_fallback_path() {
        let bindings = SymbolExprMap::default();
        let module_rhs = Expr::call(
            system_expr("Module"),
            vec![
                Expr::call(system_expr("List"), vec![Expr::symbol("x")]),
                Expr::symbol("x"),
            ],
        );
        let with_rhs = Expr::call(
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
        );
        let function_rhs = Expr::call(
            system_expr("Function"),
            vec![
                Expr::call(system_expr("List"), vec![Expr::symbol("x")]),
                Expr::symbol("x"),
            ],
        );

        assert!(compile_rhs_template(&module_rhs, &bindings).is_none());
        assert!(compile_rhs_template(&with_rhs, &bindings).is_none());
        assert!(compile_rhs_template(&function_rhs, &bindings).is_none());
    }

    #[test]
    fn module_if_sameq_template_matches_tree_walk_when_branch_not_taken() {
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
    fn module_if_sameq_template_matches_tree_walk_when_branch_taken() {
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
    fn module_if_sameq_template_rejects_escaping_local() {
        let x = Symbol::from_name("x");
        let res = Symbol::from_name("res");
        let rhs = module_if_sameq_rhs(
            Expr::call(system_expr("Hold"), vec![symbol_expr(&res)]),
            Expr::symbol("failed"),
            Expr::call(system_expr("fallback"), vec![symbol_expr(&x)]),
        );
        let mut bindings = SymbolExprMap::default();
        bindings.insert(x, Expr::symbol("value"));

        assert!(compile_rhs_template(&rhs, &bindings).is_none());
    }

    #[test]
    fn module_if_sameq_template_rejects_repeatc_shape() {
        let res = Symbol::from_name("res");
        let rhs = Expr::call(
            system_expr("Module"),
            vec![
                Expr::call(
                    system_expr("List"),
                    vec![symbol_expr(&res), Expr::symbol("rhs")],
                ),
                Expr::call(
                    system_expr("CompoundExpression"),
                    vec![
                        Expr::call(
                            system_expr("Set"),
                            vec![symbol_expr(&res), Expr::symbol("x")],
                        ),
                        Expr::call(
                            system_expr("If"),
                            vec![
                                Expr::call(
                                    system_expr("SameQ"),
                                    vec![symbol_expr(&res), Expr::symbol("failed")],
                                ),
                                Expr::call(system_expr("Return"), vec![Expr::symbol("fallback")]),
                            ],
                        ),
                    ],
                ),
            ],
        );

        assert!(compile_rhs_template(&rhs, &SymbolExprMap::default()).is_none());
    }

    #[test]
    fn module_repeatc_template_matches_tree_walk_when_failed_return_taken() {
        let (rhs, mut bindings) = repeatc_rhs_and_bindings("failConv", "start");
        assert_compiled_repeatc_matches_tree_walk(&rhs, &bindings, "refl[start]");

        bindings.insert(Symbol::from_name("c"), Expr::symbol("stableConv"));
        assert_compiled_repeatc_matches_tree_walk(&rhs, &bindings, "thm[start, start]");
    }

    #[test]
    fn module_repeatc_template_matches_tree_walk_when_stable_return_taken() {
        let (rhs, bindings) = repeatc_rhs_and_bindings("stableConv", "start");
        assert_compiled_repeatc_matches_tree_walk(&rhs, &bindings, "thm[start, start]");
    }

    #[test]
    fn module_repeatc_template_matches_tree_walk_when_recursive_fallthrough_runs() {
        let (rhs, bindings) = repeatc_rhs_and_bindings("stepConv", "start");
        assert_compiled_repeatc_matches_tree_walk(
            &rhs,
            &bindings,
            "transResult[thm[start, mid], refl[mid]]",
        );
    }

    #[test]
    fn module_repeatc_template_rejects_held_local_read() {
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

        assert!(compile_rhs_template(&rhs, &bindings).is_none());
    }

    fn assert_template_matches_tree_walk(rhs: &Expr, bindings: &SymbolExprMap) {
        let code = compile_rhs_template(rhs, bindings).expect("RHS should compile");
        let actual = execute_rhs_template(&code, bindings).expect("template should execute");
        let expected = substitute_rule_bindings_for_rhs(
            rhs,
            bindings,
            rule_rhs_may_splice_without_bindings(rhs),
        );
        assert_eq!(actual, expected);
    }

    fn assert_compiled_module_matches_tree_walk(
        rhs: &Expr,
        bindings: &SymbolExprMap,
        expected_text: &str,
    ) {
        let code = compile_rhs_template(rhs, bindings).expect("Module RHS should compile");
        assert!(matches!(code.program, CodeProgram::ModuleIfSameQ { .. }));
        let prepared = prepare_compiled_module_template(code, bindings)
            .expect("compiled module should prepare");
        let mut actual_session = Session::new();
        let actual = evaluate_prepared_rhs_template(
            &mut actual_session,
            PreparedRhsTemplate::CompiledModule(prepared),
            None,
        )
        .expect("compiled module should evaluate");

        let substituted = substitute_rule_bindings_for_rhs(
            rhs,
            bindings,
            rule_rhs_may_splice_without_bindings(rhs),
        );
        let mut expected_session = Session::new();
        let expected = expected_session
            .evaluate_definition_rhs(None, substituted)
            .expect("tree-walk Module should evaluate");

        assert_eq!(actual, expected);
        assert_eq!(actual.to_string(), expected_text);
    }

    fn assert_compiled_repeatc_matches_tree_walk(
        rhs: &Expr,
        bindings: &SymbolExprMap,
        expected_text: &str,
    ) {
        let code = compile_rhs_template(rhs, bindings).expect("REPEATC Module RHS should compile");
        assert!(matches!(code.program, CodeProgram::ModuleRepeatC { .. }));
        let prepared = prepare_compiled_module_template(code, bindings)
            .expect("compiled REPEATC module should prepare");
        let mut actual_session = repeatc_test_session();
        let actual = evaluate_prepared_rhs_template(
            &mut actual_session,
            PreparedRhsTemplate::CompiledModule(prepared),
            None,
        )
        .expect("compiled REPEATC module should evaluate");

        let substituted = substitute_rule_bindings_for_rhs(
            rhs,
            bindings,
            rule_rhs_may_splice_without_bindings(rhs),
        );
        let mut expected_session = repeatc_test_session();
        let expected = expected_session
            .evaluate_definition_rhs(None, substituted)
            .expect("tree-walk REPEATC Module should evaluate");

        assert_eq!(actual, expected);
        assert_eq!(actual.to_string(), expected_text);
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
        let mut session = Session::new();
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

    fn symbol_expr(symbol: &Symbol) -> Expr {
        Expr::Atom(Atom::Symbol(*symbol))
    }
}
