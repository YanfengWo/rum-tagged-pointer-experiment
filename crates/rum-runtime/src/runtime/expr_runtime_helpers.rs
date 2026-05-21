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
    match (lhs, rhs) {
        (Expr::Atom(lhs), Expr::Atom(rhs)) => compare_atom_canonical(lhs, rhs),
        (
            Expr::Call {
                head: lhs_head,
                args: lhs_args,
            },
            Expr::Call {
                head: rhs_head,
                args: rhs_args,
            },
        ) => compare_call_canonical(lhs_head, lhs_args, rhs_head, rhs_args),
        _ => lhs.to_string().cmp(&rhs.to_string()),
    }
}

fn compare_atom_canonical(lhs: &Atom, rhs: &Atom) -> std::cmp::Ordering {
    match (lhs, rhs) {
        (Atom::Integer(lhs), Atom::Integer(rhs)) => lhs.cmp(rhs),
        (Atom::Integer(lhs), Atom::Rational(rhs)) => {
            compare_rational_values(&Rational::from_integer(lhs.clone()), rhs)
        }
        (Atom::Rational(lhs), Atom::Integer(rhs)) => {
            compare_rational_values(lhs, &Rational::from_integer(rhs.clone()))
        }
        (Atom::Rational(lhs), Atom::Rational(rhs)) => compare_rational_values(lhs, rhs),
        (Atom::String(lhs), Atom::String(rhs)) => lhs.cmp(rhs),
        (Atom::Symbol(lhs), Atom::Symbol(rhs)) => lhs.qualified_name().cmp(&rhs.qualified_name()),
        _ => lhs.to_string().cmp(&rhs.to_string()),
    }
}

fn compare_call_canonical(
    lhs_head: &Expr,
    lhs_args: &[Expr],
    rhs_head: &Expr,
    rhs_args: &[Expr],
) -> std::cmp::Ordering {
    let head_ordering = compare_expr_canonical(lhs_head, rhs_head);
    if head_ordering != std::cmp::Ordering::Equal {
        return head_ordering;
    }

    for (lhs_arg, rhs_arg) in lhs_args.iter().zip(rhs_args) {
        let arg_ordering = compare_expr_canonical(lhs_arg, rhs_arg);
        if arg_ordering != std::cmp::Ordering::Equal {
            return arg_ordering;
        }
    }

    lhs_args.len().cmp(&rhs_args.len())
}

fn compare_rational_values(lhs: &Rational, rhs: &Rational) -> std::cmp::Ordering {
    let lhs_cross = lhs.numerator() * rhs.denominator();
    let rhs_cross = rhs.numerator() * lhs.denominator();
    lhs_cross.cmp(&rhs_cross)
}

pub(super) fn is_known_system_symbol(symbol: &Symbol) -> bool {
    symbol.context() == SYSTEM_CONTEXT && is_known_system_symbol_name(symbol.name())
}

pub(super) fn is_known_system_symbol_name(name: &str) -> bool {
    if rum_core::is_canonical_core_head_name(name) {
        return true;
    }

    if default_builtin_dispatch_spec(name).is_some() {
        return true;
    }

    matches!(
        name,
        "Alternatives"
            | "DateObject"
            | "DistanceFunction"
            | "Flat"
            | "Heads"
            | "HoldAll"
            | "HoldAllComplete"
            | "HoldFirst"
            | "HoldRest"
            | "Listable"
            | "NearestTo"
            | "OptionsPattern"
            | "EvaluationMonitor"
            | "DigitCharacter"
            | "LetterCharacter"
            | "OneIdentity"
            | "Orderless"
            | "OrderlessPatternSequence"
            | "PatternTest"
            | "PatternSequence"
            | "Protected"
            | "RegularExpression"
            | "StepMonitor"
            | "StringExpression"
            | "Temporary"
            | "TimeObject"
            | "UpTo"
            | "Verbatim"
            | "WhitespaceCharacter"
            | "WordCharacter"
            | "SequenceHold"
    )
}

pub(super) fn is_system_symbol_name(expr: &Expr, name: &str) -> bool {
    expr_as_symbol(expr)
        .map(|symbol| symbol.context() == SYSTEM_CONTEXT && symbol.name() == name)
        .unwrap_or(false)
}
