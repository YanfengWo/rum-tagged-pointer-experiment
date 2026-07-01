use super::*;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};

const RHS_TEMPLATE_ENV: &str = "RUM_VM_RHS_TEMPLATE";

static RHS_TEMPLATE_ENABLED: AtomicU8 = AtomicU8::new(0);
static RHS_TEMPLATE_COMPILED_DISTINCT: AtomicU64 = AtomicU64::new(0);
static RHS_TEMPLATE_FALLBACK_DISTINCT: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug)]
enum Op {
    PushConst(SharedExpr),
    PushBinding(usize),
    Construct { argc: usize },
}

#[derive(Clone, Debug)]
struct CodeObject {
    source: Expr,
    ops: Vec<Op>,
    slots: Vec<Symbol>,
    const_symbols: Vec<Symbol>,
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

pub(super) fn substitute_rule_bindings_for_rhs_with_template(
    expr: &Expr,
    bindings: &SymbolExprMap,
    may_splice_without_bindings: bool,
    evaluation_epoch: u64,
) -> Expr {
    if !rhs_template_enabled() {
        return substitute_rule_bindings_for_rhs(expr, bindings, may_splice_without_bindings);
    }

    substitute_rule_bindings_for_rhs_with_template_slow(
        expr,
        bindings,
        may_splice_without_bindings,
        evaluation_epoch,
    )
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
fn substitute_rule_bindings_for_rhs_with_template_slow(
    expr: &Expr,
    bindings: &SymbolExprMap,
    may_splice_without_bindings: bool,
    evaluation_epoch: u64,
) -> Expr {
    if let Some(code) = cached_rhs_template(expr, bindings, evaluation_epoch) {
        if let Some(result) = execute_rhs_template(&code, bindings) {
            #[cfg(debug_assertions)]
            {
                let expected =
                    substitute_rule_bindings_for_rhs(expr, bindings, may_splice_without_bindings);
                debug_assert_eq!(
                    result, expected,
                    "RHS template diverged from tree substitution"
                );
            }
            return result;
        }
    }

    substitute_rule_bindings_for_rhs(expr, bindings, may_splice_without_bindings)
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
            cache
                .entries
                .insert(key.clone(), CacheEntry::Compiled(code.clone()));
            record_distinct_template_compile(&mut cache, key);
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

fn record_distinct_template_compile(cache: &mut TemplateCache, key: RhsTemplateKey) {
    if cache.reported_compiled.insert(key) {
        let compiled = RHS_TEMPLATE_COMPILED_DISTINCT.fetch_add(1, Ordering::Relaxed) + 1;
        let fallback = RHS_TEMPLATE_FALLBACK_DISTINCT.load(Ordering::Relaxed);
        eprintln!(
            "rum-vm-rhs-template coverage compiled_distinct={compiled} fallback_distinct={fallback}"
        );
    }
}

fn record_distinct_template_fallback(cache: &mut TemplateCache, key: RhsTemplateKey) {
    if cache.reported_fallback.insert(key) {
        let fallback = RHS_TEMPLATE_FALLBACK_DISTINCT.fetch_add(1, Ordering::Relaxed) + 1;
        let compiled = RHS_TEMPLATE_COMPILED_DISTINCT.load(Ordering::Relaxed);
        eprintln!(
            "rum-vm-rhs-template coverage compiled_distinct={compiled} fallback_distinct={fallback}"
        );
    }
}

fn compile_rhs_template(expr: &Expr, bindings: &SymbolExprMap) -> Option<CodeObject> {
    let mut compiler = RhsTemplateCompiler::new(bindings);
    compiler.compile_expr(expr)?;
    Some(CodeObject {
        source: expr.clone(),
        ops: compiler.ops,
        slots: compiler.slots,
        const_symbols: compiler.const_symbols,
    })
}

struct RhsTemplateCompiler<'a> {
    bindings: &'a SymbolExprMap,
    ops: Vec<Op>,
    slots: Vec<Symbol>,
    const_symbols: Vec<Symbol>,
}

impl<'a> RhsTemplateCompiler<'a> {
    fn new(bindings: &'a SymbolExprMap) -> Self {
        Self {
            bindings,
            ops: Vec::new(),
            slots: Vec::new(),
            const_symbols: Vec::new(),
        }
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

    fn can_push_const_subtree(&self, expr: &Expr) -> bool {
        !expr_contains_bound_symbol(expr, self.bindings)
            && !rule_rhs_may_splice_without_bindings(expr)
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
    let mut slot_values = Vec::with_capacity(code.slots.len());
    for symbol in &code.slots {
        slot_values.push(bindings.get(symbol)?.clone());
    }

    if code
        .const_symbols
        .iter()
        .any(|symbol| bindings.contains_key(symbol))
    {
        return None;
    }

    let mut stack = Vec::new();
    for op in &code.ops {
        match op {
            Op::PushConst(expr) => stack.push(expr.as_ref().clone()),
            Op::PushBinding(slot) => stack.push(slot_values.get(*slot)?.clone()),
            Op::Construct { argc } => {
                if stack.len() < argc.saturating_add(1) {
                    return None;
                }
                let args_start = stack.len() - argc;
                let args = stack.split_off(args_start);
                let head = stack.pop()?;
                stack.push(Expr::call(head, splice_sequence_args(args)));
            }
        }
    }

    if stack.len() == 1 { stack.pop() } else { None }
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
}
