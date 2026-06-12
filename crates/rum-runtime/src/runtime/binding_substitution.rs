use super::*;

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
    if bindings.is_empty() && !may_splice_without_bindings {
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
        Expr::Atom(Atom::Symbol(symbol)) => bindings.get(symbol).cloned(),
        Expr::Atom(_) => None,
        Expr::Call { head, args } => {
            let substituted_head = substitute_rule_bindings_changed(head, bindings);
            let mut substituted_args = None;

            for (index, arg) in args.iter().enumerate() {
                if let Some(substituted_arg) = substitute_rule_bindings_changed(arg, bindings) {
                    let output_args =
                        substituted_args.get_or_insert_with(|| args[..index].to_vec());
                    if let Some(sequence_items) = sequence_items(&substituted_arg) {
                        output_args.extend(sequence_items.iter().cloned());
                    } else {
                        output_args.push(substituted_arg);
                    }
                } else if let Some(sequence_items) = sequence_items(arg) {
                    substituted_args
                        .get_or_insert_with(|| args[..index].to_vec())
                        .extend(sequence_items.iter().cloned());
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
