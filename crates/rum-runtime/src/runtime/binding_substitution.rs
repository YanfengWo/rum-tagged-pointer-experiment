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
static RULE_RHS_ALPHA_RENAME_ID: AtomicU64 = AtomicU64::new(0);

const RULE_RHS_ALPHA_ID_MARKER: u64 = 1_u64 << 63;

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
    substitute_rule_bindings_changed(expr, bindings, &SymbolExprMap::default())
        .unwrap_or_else(|| expr.clone())
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

fn substitute_rule_bindings_changed(
    expr: &Expr,
    bindings: &SymbolExprMap,
    alpha_renames: &SymbolExprMap,
) -> Option<Expr> {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => {
            let value = bindings
                .get(symbol)
                .cloned()
                .or_else(|| alpha_renames.get(symbol).cloned());
            if let Some(value) = &value {
                record_rule_rhs_binding_hit(value);
            }
            value
        }
        Expr::Atom(_) => None,
        Expr::Call { head, args } if is_system_symbol_name(head, "Module") && args.len() == 2 => {
            substitute_rule_scoped_call_changed(head, args, bindings, alpha_renames)
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "With") && args.len() == 2 => {
            substitute_rule_with_call_changed(head, args, bindings, alpha_renames)
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Function") => {
            substitute_rule_function_call_changed(head, args, bindings, alpha_renames)
        }
        Expr::Call { head, args } => substitute_rule_call_changed_with_arg_alphas(
            head,
            args,
            bindings,
            alpha_renames,
            |_| alpha_renames,
        ),
    }
}

fn substitute_rule_scoped_call_changed(
    head: &Expr,
    args: &SharedArgs,
    bindings: &SymbolExprMap,
    alpha_renames: &SymbolExprMap,
) -> Option<Expr> {
    let Some(local_symbols) = parse_module_binding_list(&args[0]).map(scoped_binding_symbols)
    else {
        return substitute_rule_call_changed_with_arg_alphas(
            head,
            args,
            bindings,
            alpha_renames,
            |_| alpha_renames,
        );
    };
    let scoped_alpha_renames =
        alpha_renames_with_fresh_locals(alpha_renames, bindings, local_symbols);

    substitute_rule_call_changed_with_arg_alphas(head, args, bindings, alpha_renames, |_| {
        &scoped_alpha_renames
    })
}

fn substitute_rule_with_call_changed(
    head: &Expr,
    args: &SharedArgs,
    bindings: &SymbolExprMap,
    alpha_renames: &SymbolExprMap,
) -> Option<Expr> {
    let Some(local_symbols) = parse_with_binding_list(&args[0]).map(with_binding_symbols) else {
        return substitute_rule_call_changed_with_arg_alphas(
            head,
            args,
            bindings,
            alpha_renames,
            |_| alpha_renames,
        );
    };
    let scoped_alpha_renames =
        alpha_renames_with_fresh_locals(alpha_renames, bindings, local_symbols);

    let substituted_head = substitute_rule_bindings_changed(head, bindings, alpha_renames);
    let substituted_bindings = substitute_rule_with_binding_list_changed(
        &args[0],
        bindings,
        alpha_renames,
        &scoped_alpha_renames,
    );
    let substituted_body =
        substitute_rule_bindings_changed(&args[1], bindings, &scoped_alpha_renames);

    if substituted_head.is_none() && substituted_bindings.is_none() && substituted_body.is_none() {
        return None;
    }

    Some(Expr::call(
        substituted_head.unwrap_or_else(|| head.clone()),
        vec![
            substituted_bindings.unwrap_or_else(|| args[0].clone()),
            substituted_body.unwrap_or_else(|| args[1].clone()),
        ],
    ))
}

fn substitute_rule_function_call_changed(
    head: &Expr,
    args: &SharedArgs,
    bindings: &SymbolExprMap,
    alpha_renames: &SymbolExprMap,
) -> Option<Expr> {
    let Some(params) = explicit_function_parameters_from_args(args) else {
        return substitute_rule_call_changed_with_arg_alphas(
            head,
            args,
            bindings,
            alpha_renames,
            |_| alpha_renames,
        );
    };
    let scoped_alpha_renames = alpha_renames_with_fresh_locals(alpha_renames, bindings, params);

    substitute_rule_call_changed_with_arg_alphas(head, args, bindings, alpha_renames, |index| {
        if index <= 1 {
            &scoped_alpha_renames
        } else {
            alpha_renames
        }
    })
}

fn substitute_rule_call_changed_with_arg_alphas<'a, F>(
    head: &Expr,
    args: &SharedArgs,
    bindings: &SymbolExprMap,
    head_alpha_renames: &SymbolExprMap,
    mut arg_alpha_renames: F,
) -> Option<Expr>
where
    F: FnMut(usize) -> &'a SymbolExprMap,
{
    let substituted_head = substitute_rule_bindings_changed(head, bindings, head_alpha_renames);
    let mut substituted_args = None;

    for (index, arg) in args.iter().enumerate() {
        let arg_alpha_renames = arg_alpha_renames(index);
        if let Some(substituted_arg) =
            substitute_rule_bindings_changed(arg, bindings, arg_alpha_renames)
        {
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

    rebuild_rule_call(head, args, substituted_head, substituted_args)
}

fn substitute_rule_with_binding_list_changed(
    expr: &Expr,
    bindings: &SymbolExprMap,
    alpha_renames: &SymbolExprMap,
    scoped_alpha_renames: &SymbolExprMap,
) -> Option<Expr> {
    let Expr::Call { head, args } = expr else {
        return substitute_rule_bindings_changed(expr, bindings, alpha_renames);
    };

    if !is_system_symbol_name(head, "List") {
        return substitute_rule_bindings_changed(expr, bindings, alpha_renames);
    }

    let substituted_head = substitute_rule_bindings_changed(head, bindings, alpha_renames);
    let mut substituted_items = None;
    let mut item_rhs_alpha_renames = alpha_renames.clone();

    for (index, item) in args.iter().enumerate() {
        let substituted_item = substitute_rule_with_binding_item_changed(
            item,
            bindings,
            &item_rhs_alpha_renames,
            scoped_alpha_renames,
        );

        if let Some(substituted_item) = substituted_item {
            substituted_items
                .get_or_insert_with(|| args[..index].to_vec())
                .push(substituted_item);
        } else if let Some(substituted_items) = &mut substituted_items {
            substituted_items.push(item.clone());
        }

        if let Some((symbol, _)) = parse_with_binding_item(item) {
            if let Some(alpha) = scoped_alpha_renames.get(&symbol) {
                item_rhs_alpha_renames.insert(symbol, alpha.clone());
            }
        }
    }

    match (substituted_head, substituted_items) {
        (None, None) => None,
        (Some(head), None) => Some(Expr::call_with_shared_args(head, args.clone())),
        (None, Some(args)) => Some(Expr::call_shared(
            SharedExpr::from(head.as_ref().clone()),
            SharedArgs::new(args),
        )),
        (Some(head), Some(args)) => Some(Expr::call(head, args)),
    }
}

fn substitute_rule_with_binding_item_changed(
    expr: &Expr,
    bindings: &SymbolExprMap,
    rhs_alpha_renames: &SymbolExprMap,
    scoped_alpha_renames: &SymbolExprMap,
) -> Option<Expr> {
    match expr {
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Set") || is_system_symbol_name(head, "Rule"))
                && args.len() == 2
                && expr_as_symbol(&args[0]).is_some() =>
        {
            let substituted_head =
                substitute_rule_bindings_changed(head, bindings, rhs_alpha_renames);
            let substituted_lhs =
                substitute_rule_bindings_changed(&args[0], bindings, scoped_alpha_renames);
            let substituted_rhs =
                substitute_rule_bindings_changed(&args[1], bindings, rhs_alpha_renames);

            if substituted_head.is_none() && substituted_lhs.is_none() && substituted_rhs.is_none()
            {
                return None;
            }

            Some(Expr::call(
                substituted_head.unwrap_or_else(|| head.as_ref().clone()),
                vec![
                    substituted_lhs.unwrap_or_else(|| args[0].clone()),
                    substituted_rhs.unwrap_or_else(|| args[1].clone()),
                ],
            ))
        }
        _ => substitute_rule_bindings_changed(expr, bindings, rhs_alpha_renames),
    }
}

fn rebuild_rule_call(
    head: &Expr,
    original_args: &SharedArgs,
    substituted_head: Option<Expr>,
    substituted_args: Option<Vec<Expr>>,
) -> Option<Expr> {
    match (substituted_head, substituted_args) {
        (None, None) => None,
        (Some(head), None) => {
            record_substitution_sharing_counter(
                &RULE_RHS_REBUILT_REUSE_SHARED_ARGS_COUNT,
                "rule_rhs_rebuilt_reuse_shared_args",
            );
            Some(Expr::call_with_shared_args(head, original_args.clone()))
        }
        (None, Some(args)) => {
            record_substitution_sharing_counter(
                &RULE_RHS_REBUILT_REUSE_SHARED_HEAD_COUNT,
                "rule_rhs_rebuilt_reuse_shared_head",
            );
            Some(Expr::call_shared(
                SharedExpr::from(head.clone()),
                SharedArgs::new(args),
            ))
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

fn scoped_binding_symbols(items: Vec<(Symbol, Option<Expr>)>) -> Vec<Symbol> {
    items.into_iter().map(|(symbol, _)| symbol).collect()
}

fn with_binding_symbols(items: Vec<(Symbol, Expr)>) -> Vec<Symbol> {
    items.into_iter().map(|(symbol, _)| symbol).collect()
}

fn explicit_function_parameters_from_args(args: &[Expr]) -> Option<Vec<Symbol>> {
    match args {
        [params, _body] => parse_function_parameters(params),
        [params, _body, _attributes] => parse_function_parameters(params),
        _ => None,
    }
}

fn alpha_renames_with_fresh_locals(
    alpha_renames: &SymbolExprMap,
    bindings: &SymbolExprMap,
    locals: Vec<Symbol>,
) -> SymbolExprMap {
    let mut scoped = alpha_renames.clone();
    for local in locals {
        if bindings.contains_key(&local) {
            continue;
        }
        let alpha = fresh_rule_rhs_alpha_symbol(&local);
        scoped.insert(local, Expr::Atom(Atom::Symbol(alpha)));
    }
    scoped
}

fn fresh_rule_rhs_alpha_symbol(symbol: &Symbol) -> Symbol {
    let id = RULE_RHS_ALPHA_RENAME_ID.fetch_add(1, Ordering::Relaxed) + 1;
    Symbol::with_identity(
        symbol.context(),
        symbol.name(),
        RULE_RHS_ALPHA_ID_MARKER | id,
    )
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
