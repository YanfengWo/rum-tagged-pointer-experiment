use super::*;
use std::collections::HashSet;

pub(super) fn duplicate_scoped_symbols(symbols: impl Iterator<Item = Symbol>) -> bool {
    let mut seen = HashSet::new();
    symbols.into_iter().any(|symbol| !seen.insert(symbol))
}

pub(super) fn leaf_contains_uncompiled_scope(expr: &Expr) -> bool {
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

pub(super) fn locals_are_safe_in_compiled_expr(expr: &Expr, locals: &[Symbol]) -> bool {
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
