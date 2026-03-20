use super::*;

pub(super) fn is_slot_sequence_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Call { head, args } => {
            is_system_symbol_name(head, heads::functions::SLOT_SEQUENCE) && args.len() == 1
        }
        _ => false,
    }
}

pub(super) fn sequence_items(expr: &Expr) -> Option<&[Expr]> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, heads::functions::SEQUENCE) => {
            Some(args)
        }
        _ => None,
    }
}

pub(super) fn call_items_with_head<'a>(expr: &'a Expr, expected_head: &Expr) -> Option<&'a [Expr]> {
    match expr {
        Expr::Call { head, args } if head.as_ref() == expected_head => Some(args),
        _ => None,
    }
}

pub(super) fn format_list_lengths(lengths: &[usize]) -> String {
    lengths
        .iter()
        .map(|length| length.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) fn flatten_flat_call_args(head: &Expr, args: Vec<Expr>) -> Vec<Expr> {
    let mut flattened = Vec::new();

    for arg in args {
        match arg {
            Expr::Call {
                head: nested_head,
                args: nested_args,
            } if nested_head.as_ref() == head => flattened.extend(nested_args),
            other => flattened.push(other),
        }
    }

    flattened
}

pub(super) fn compare_expr_canonical(lhs: &Expr, rhs: &Expr) -> std::cmp::Ordering {
    lhs.to_string().cmp(&rhs.to_string())
}

pub(super) fn is_known_system_symbol(symbol: &Symbol) -> bool {
    symbol.context() == SYSTEM_CONTEXT && is_known_system_symbol_name(symbol.name())
}

pub(super) fn is_known_system_symbol_name(name: &str) -> bool {
    if rum_core::is_canonical_core_head_name(name) {
        return true;
    }

    matches!(
        name,
        "Cases"
            | "Check"
            | "Clear"
            | "ClearAll"
            | "Flat"
            | "Heads"
            | "Hold"
            | "HoldAll"
            | "HoldAllComplete"
            | "HoldFirst"
            | "HoldRest"
            | "Identity"
            | "Listable"
            | "MatchQ"
            | "OneIdentity"
            | "Orderless"
            | "Quiet"
            | "SequenceHold"
            | "Thread"
    )
}

pub(super) fn is_system_symbol_name(expr: &Expr, name: &str) -> bool {
    expr_as_symbol(expr)
        .map(|symbol| symbol.context() == SYSTEM_CONTEXT && symbol.name() == name)
        .unwrap_or(false)
}
