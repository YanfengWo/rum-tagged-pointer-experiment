use super::compiler::{code_compatible_with_bindings, compile_body};
use super::model::CodeObject;
use super::*;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

const RHS_TEMPLATE_ENV: &str = "RUM_VM_RHS_TEMPLATE";

static RHS_TEMPLATE_ENABLED: AtomicU8 = AtomicU8::new(0);
static RHS_TEMPLATE_COMPILED_DISTINCT: AtomicU64 = AtomicU64::new(0);
static RHS_TEMPLATE_FALLBACK_DISTINCT: AtomicU64 = AtomicU64::new(0);

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

pub(super) fn rhs_template_enabled() -> bool {
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

pub(super) fn cached_rhs_template(
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
