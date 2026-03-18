use super::*;

pub(super) fn is_slot_sequence_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Call { head, args } => is_system_symbol_name(head, "SlotSequence") && args.len() == 1,
        _ => false,
    }
}

pub(super) fn sequence_items(expr: &Expr) -> Option<&[Expr]> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Sequence") => Some(args),
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
    matches!(
        name,
        "$Context"
            | "$ContextPath"
            | "$Failed"
            | "$MessageList"
            | "$Packages"
            | "$Path"
            | "All"
            | "Attributes"
            | "Begin"
            | "BeginPackage"
            | "Blank"
            | "BlankNullSequence"
            | "BlankSequence"
            | "Block"
            | "Cases"
            | "Check"
            | "Clear"
            | "ClearAll"
            | "ClearAttributes"
            | "Condition"
            | "Context"
            | "End"
            | "EndPackage"
            | "Evaluate"
            | "False"
            | "File"
            | "FindFile"
            | "Flat"
            | "Function"
            | "Get"
            | "General"
            | "Heads"
            | "Hold"
            | "HoldAll"
            | "HoldAllComplete"
            | "HoldFirst"
            | "HoldPattern"
            | "HoldRest"
            | "Identity"
            | "If"
            | "Infinity"
            | "Integer"
            | "List"
            | "Listable"
            | "MatchQ"
            | "Message"
            | "MessageDefinitions"
            | "MessageName"
            | "Messages"
            | "MessageTags"
            | "Module"
            | "Needs"
            | "NormalizeRules"
            | "None"
            | "Null"
            | "OneIdentity"
            | "OwnValues"
            | "Orderless"
            | "Pattern"
            | "Plus"
            | "Power"
            | "ReplaceAll"
            | "ReplaceRepeated"
            | "Rule"
            | "RuleDelayed"
            | "Quiet"
            | "ReleaseHold"
            | "Sequence"
            | "SequenceHold"
            | "Set"
            | "SetAttributes"
            | "SetDelayed"
            | "Slot"
            | "SlotSequence"
            | "String"
            | "SubValues"
            | "Symbol"
            | "TagSet"
            | "TagSetDelayed"
            | "TagUnset"
            | "Thread"
            | "Times"
            | "True"
            | "Unevaluated"
            | "DownValues"
            | "Unset"
            | "UpValues"
            | "Which"
            | "With"
    )
}

pub(super) fn is_system_symbol_name(expr: &Expr, name: &str) -> bool {
    expr_as_symbol(expr)
        .map(|symbol| symbol.context() == SYSTEM_CONTEXT && symbol.name() == name)
        .unwrap_or(false)
}
