use super::*;
use std::sync::atomic::{AtomicU8, AtomicU64, Ordering};

const SUBSTITUTION_SHARING_PROFILE_ENV: &str = "RUM_PROFILE_SUBSTITUTION_SHARING";

static SUBSTITUTION_SHARING_PROFILE_ENABLED: AtomicU8 = AtomicU8::new(0);
static RULE_RHS_SUBSTITUTION_REQUEST_COUNT: AtomicU64 = AtomicU64::new(0);
static RULE_RHS_FAST_CLONE_EMPTY_BINDINGS_COUNT: AtomicU64 = AtomicU64::new(0);
static RULE_RHS_BINDING_HIT_COUNT: AtomicU64 = AtomicU64::new(0);
static RULE_RHS_BINDING_CALL_HIT_COUNT: AtomicU64 = AtomicU64::new(0);
static RULE_RHS_BINDING_ATOM_HIT_COUNT: AtomicU64 = AtomicU64::new(0);
static RULE_RHS_REBUILT_REUSE_SHARED_ARGS_COUNT: AtomicU64 = AtomicU64::new(0);
static RULE_RHS_REBUILT_REUSE_SHARED_HEAD_COUNT: AtomicU64 = AtomicU64::new(0);
static RULE_RHS_REBUILT_FRESH_HEAD_ARGS_COUNT: AtomicU64 = AtomicU64::new(0);
static RULE_RHS_PRESERVED_EXISTING_ARG_COUNT: AtomicU64 = AtomicU64::new(0);
static RULE_RHS_PRESERVED_EXISTING_CALL_ARG_COUNT: AtomicU64 = AtomicU64::new(0);
static RULE_RHS_SPLICE_FROM_BINDING_COUNT: AtomicU64 = AtomicU64::new(0);
static RULE_RHS_SPLICE_FROM_EXISTING_RHS_COUNT: AtomicU64 = AtomicU64::new(0);

pub(super) fn bind_pattern_name(name: &Symbol, value: Expr, bindings: &mut SymbolExprMap) -> bool {
    match bindings.get(name) {
        Some(existing) => existing == &value,
        None => {
            bindings.insert(name.clone(), value);
            true
        }
    }
}

pub(super) fn sequence_expr(values: Vec<Expr>) -> Expr {
    Expr::call(system_expr("Sequence"), values)
}

pub(super) fn substitute_bindings(expr: &Expr, bindings: &SymbolExprMap) -> Expr {
    substitute_bindings_changed(expr, bindings).unwrap_or_else(|| expr.clone())
}

fn substitute_bindings_changed(expr: &Expr, bindings: &SymbolExprMap) -> Option<Expr> {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => bindings.get(symbol).cloned(),
        Expr::Atom(_) => None,
        Expr::Call { head, args } => {
            let substituted_head = substitute_bindings_changed(head, bindings);
            let mut substituted_args = None;

            for (index, arg) in args.iter().enumerate() {
                if let Some(substituted_arg) = substitute_bindings_changed(arg, bindings) {
                    substituted_args
                        .get_or_insert_with(|| args[..index].to_vec())
                        .push(substituted_arg);
                } else if let Some(substituted_args) = &mut substituted_args {
                    substituted_args.push(arg.clone());
                }
            }

            match (substituted_head, substituted_args) {
                (None, None) => None,
                (Some(head), None) => Some(Expr::call_with_shared_args(head, args.clone())),
                (None, Some(args)) => Some(Expr::call_shared(head.clone(), SharedArgs::new(args))),
                (Some(head), Some(args)) => Some(Expr::call(head, args)),
            }
        }
    }
}

pub(super) fn substitute_rule_bindings(expr: &Expr, bindings: &SymbolExprMap) -> Expr {
    substitute_rule_bindings_changed(expr, bindings).unwrap_or_else(|| expr.clone())
}

pub(super) fn substitute_rule_bindings_for_rhs(
    expr: &Expr,
    bindings: &SymbolExprMap,
    may_splice_without_bindings: bool,
) -> Expr {
    record_substitution_sharing_counter(
        &RULE_RHS_SUBSTITUTION_REQUEST_COUNT,
        "rule_rhs_substitution_requests",
    );
    if bindings.is_empty() && !may_splice_without_bindings {
        record_substitution_sharing_counter(
            &RULE_RHS_FAST_CLONE_EMPTY_BINDINGS_COUNT,
            "rule_rhs_fast_clone_empty_bindings",
        );
        expr.clone()
    } else {
        substitute_rule_bindings(expr, bindings)
    }
}

pub(super) fn rule_rhs_may_splice_without_bindings(expr: &Expr) -> bool {
    match expr {
        Expr::Call { head, args } => {
            rule_rhs_may_splice_without_bindings(head)
                || args.iter().any(|arg| {
                    sequence_items(arg).is_some() || rule_rhs_may_splice_without_bindings(arg)
                })
        }
        Expr::Atom(_) => false,
    }
}

fn substitute_rule_bindings_changed(expr: &Expr, bindings: &SymbolExprMap) -> Option<Expr> {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => {
            let value = bindings.get(symbol).cloned();
            if let Some(value) = &value {
                record_rule_rhs_binding_hit(value);
            }
            value
        }
        Expr::Atom(_) => None,
        Expr::Call { head, args } => {
            let substituted_head = substitute_rule_bindings_changed(head, bindings);
            let mut substituted_args = None;

            for (index, arg) in args.iter().enumerate() {
                if let Some(substituted_arg) = substitute_rule_bindings_changed(arg, bindings) {
                    let output_args = substituted_args.get_or_insert_with(|| {
                        record_rule_rhs_preserved_args(&args[..index]);
                        args[..index].to_vec()
                    });
                    if let Some(sequence_items) = sequence_items(&substituted_arg) {
                        record_substitution_sharing_counter(
                            &RULE_RHS_SPLICE_FROM_BINDING_COUNT,
                            "rule_rhs_splices_from_binding",
                        );
                        output_args.extend(sequence_items.iter().cloned());
                    } else {
                        output_args.push(substituted_arg);
                    }
                } else if let Some(sequence_items) = sequence_items(arg) {
                    record_substitution_sharing_counter(
                        &RULE_RHS_SPLICE_FROM_EXISTING_RHS_COUNT,
                        "rule_rhs_splices_from_existing_rhs",
                    );
                    substituted_args
                        .get_or_insert_with(|| {
                            record_rule_rhs_preserved_args(&args[..index]);
                            args[..index].to_vec()
                        })
                        .extend(sequence_items.iter().cloned());
                } else if let Some(substituted_args) = &mut substituted_args {
                    record_rule_rhs_preserved_arg(arg);
                    substituted_args.push(arg.clone());
                }
            }

            match (substituted_head, substituted_args) {
                (None, None) => None,
                (Some(head), None) => {
                    record_substitution_sharing_counter(
                        &RULE_RHS_REBUILT_REUSE_SHARED_ARGS_COUNT,
                        "rule_rhs_rebuilt_reuse_shared_args",
                    );
                    Some(Expr::call_with_shared_args(head, args.clone()))
                }
                (None, Some(args)) => {
                    record_substitution_sharing_counter(
                        &RULE_RHS_REBUILT_REUSE_SHARED_HEAD_COUNT,
                        "rule_rhs_rebuilt_reuse_shared_head",
                    );
                    Some(Expr::call_shared(head.clone(), SharedArgs::new(args)))
                }
                (Some(head), Some(args)) => {
                    record_substitution_sharing_counter(
                        &RULE_RHS_REBUILT_FRESH_HEAD_ARGS_COUNT,
                        "rule_rhs_rebuilt_fresh_head_args",
                    );
                    Some(Expr::call(head, args))
                }
            }
        }
    }
}

fn record_rule_rhs_binding_hit(value: &Expr) {
    record_substitution_sharing_counter(&RULE_RHS_BINDING_HIT_COUNT, "rule_rhs_binding_hits");
    match value {
        Expr::Call { .. } => record_substitution_sharing_counter(
            &RULE_RHS_BINDING_CALL_HIT_COUNT,
            "rule_rhs_binding_call_hits",
        ),
        Expr::Atom(_) => record_substitution_sharing_counter(
            &RULE_RHS_BINDING_ATOM_HIT_COUNT,
            "rule_rhs_binding_atom_hits",
        ),
    }
}

fn record_rule_rhs_preserved_args(args: &[Expr]) {
    for arg in args {
        record_rule_rhs_preserved_arg(arg);
    }
}

fn record_rule_rhs_preserved_arg(arg: &Expr) {
    record_substitution_sharing_counter(
        &RULE_RHS_PRESERVED_EXISTING_ARG_COUNT,
        "rule_rhs_preserved_existing_args",
    );
    if matches!(arg, Expr::Call { .. }) {
        record_substitution_sharing_counter(
            &RULE_RHS_PRESERVED_EXISTING_CALL_ARG_COUNT,
            "rule_rhs_preserved_existing_call_args",
        );
    }
}

#[inline]
fn substitution_sharing_profile_enabled() -> bool {
    match SUBSTITUTION_SHARING_PROFILE_ENABLED.load(Ordering::Relaxed) {
        1 => true,
        2 => false,
        _ => initialize_substitution_sharing_profile_enabled(),
    }
}

#[cold]
fn initialize_substitution_sharing_profile_enabled() -> bool {
    let enabled = std::env::var_os(SUBSTITUTION_SHARING_PROFILE_ENV)
        .is_some_and(|value| !value.is_empty() && value != "0");
    SUBSTITUTION_SHARING_PROFILE_ENABLED.store(if enabled { 1 } else { 2 }, Ordering::Relaxed);
    enabled
}

#[inline]
fn record_substitution_sharing_counter(counter: &AtomicU64, name: &str) {
    if !substitution_sharing_profile_enabled() {
        return;
    }

    record_substitution_sharing_counter_slow(counter, name);
}

#[cold]
#[inline(never)]
fn record_substitution_sharing_counter_slow(counter: &AtomicU64, name: &str) {
    let value = counter.fetch_add(1, Ordering::Relaxed) + 1;
    if value.is_power_of_two() {
        eprintln!("rum-profile substitution-sharing {name}={value}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_rhs_substitution_reuses_bound_call_shared_nodes() {
        let value = Expr::call(
            system_expr("g"),
            vec![Expr::call(
                system_expr("h"),
                vec![Expr::integer(BigInt::from(1_u8))],
            )],
        );
        let mut bindings = SymbolExprMap::default();
        bindings.insert(Symbol::from_name("x"), value.clone());
        let rhs = Expr::call(
            system_expr("f"),
            vec![
                Expr::Atom(Atom::Symbol(Symbol::from_name("x"))),
                Expr::Atom(Atom::Symbol(Symbol::from_name("x"))),
            ],
        );

        let substituted = substitute_rule_bindings_for_rhs(&rhs, &bindings, false);
        let Expr::Call { args, .. } = substituted else {
            panic!("expected substituted call");
        };

        assert_eq!(args.len(), 2);
        assert_call_shared_nodes_eq(&value, &args[0]);
        assert_call_shared_nodes_eq(&value, &args[1]);
        assert_call_shared_nodes_eq(&args[0], &args[1]);
    }

    #[test]
    fn rule_rhs_substitution_preserves_unchanged_sibling_call_shared_nodes() {
        let bound = Expr::call(system_expr("g"), vec![Expr::integer(BigInt::from(1_u8))]);
        let sibling = Expr::call(system_expr("s"), vec![Expr::integer(BigInt::from(2_u8))]);
        let mut bindings = SymbolExprMap::default();
        bindings.insert(Symbol::from_name("x"), bound);
        let rhs = Expr::call(
            system_expr("f"),
            vec![
                Expr::Atom(Atom::Symbol(Symbol::from_name("x"))),
                sibling.clone(),
            ],
        );

        let substituted = substitute_rule_bindings_for_rhs(&rhs, &bindings, false);
        let Expr::Call { args, .. } = substituted else {
            panic!("expected substituted call");
        };

        assert_eq!(args.len(), 2);
        assert_call_shared_nodes_eq(&sibling, &args[1]);
    }

    #[test]
    fn rule_rhs_sequence_splice_preserves_spliced_item_shared_nodes() {
        let item = Expr::call(system_expr("s"), vec![Expr::integer(BigInt::from(1_u8))]);
        let mut bindings = SymbolExprMap::default();
        bindings.insert(Symbol::from_name("x"), sequence_expr(vec![item.clone()]));
        let rhs = Expr::call(
            system_expr("f"),
            vec![Expr::Atom(Atom::Symbol(Symbol::from_name("x")))],
        );

        let substituted = substitute_rule_bindings_for_rhs(&rhs, &bindings, false);
        let Expr::Call { args, .. } = substituted else {
            panic!("expected substituted call");
        };

        assert_eq!(args.len(), 1);
        assert_call_shared_nodes_eq(&item, &args[0]);
    }

    fn assert_call_shared_nodes_eq(expected: &Expr, actual: &Expr) {
        let (
            Expr::Call {
                head: expected_head,
                args: expected_args,
            },
            Expr::Call {
                head: actual_head,
                args: actual_args,
            },
        ) = (expected, actual)
        else {
            panic!("expected both expressions to be calls");
        };

        assert!(SharedExpr::ptr_eq(expected_head, actual_head));
        assert!(SharedArgs::ptr_eq(expected_args, actual_args));
    }
}
