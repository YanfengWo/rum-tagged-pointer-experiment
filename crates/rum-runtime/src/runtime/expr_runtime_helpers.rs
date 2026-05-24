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
        _ => expr_order_rank(lhs).cmp(&expr_order_rank(rhs)),
    }
}

fn compare_atom_canonical(lhs: &Atom, rhs: &Atom) -> std::cmp::Ordering {
    let rank_ordering = atom_order_rank(lhs).cmp(&atom_order_rank(rhs));
    if rank_ordering != std::cmp::Ordering::Equal {
        return rank_ordering;
    }

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
        (Atom::Symbol(lhs), Atom::Symbol(rhs)) => compare_symbol_canonical(lhs, rhs),
        _ => std::cmp::Ordering::Equal,
    }
}

fn compare_call_canonical(
    lhs_head: &Expr,
    lhs_args: &[Expr],
    rhs_head: &Expr,
    rhs_args: &[Expr],
) -> std::cmp::Ordering {
    let head_ordering = compare_call_head_canonical(lhs_head, rhs_head);
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

fn compare_call_head_canonical(lhs: &Expr, rhs: &Expr) -> std::cmp::Ordering {
    match (expr_symbol(lhs), expr_symbol(rhs)) {
        (Some(lhs), Some(rhs)) => compare_call_head_symbols(lhs, rhs),
        _ => compare_expr_canonical(lhs, rhs),
    }
}

fn compare_call_head_symbols(lhs: &Symbol, rhs: &Symbol) -> std::cmp::Ordering {
    let lhs_rank = system_call_head_rank(lhs);
    let rhs_rank = system_call_head_rank(rhs);
    match (lhs_rank, rhs_rank) {
        (Some(lhs_rank), Some(rhs_rank)) => lhs_rank.cmp(&rhs_rank),
        (Some(lhs_rank), None) => lhs_rank.cmp(&(2, 0)),
        (None, Some(rhs_rank)) => (2, 0).cmp(&rhs_rank),
        (None, None) => compare_symbol_canonical(lhs, rhs),
    }
}

fn system_call_head_rank(symbol: &Symbol) -> Option<(u8, u8)> {
    if symbol.context() != SYSTEM_CONTEXT {
        return None;
    }

    // Wolfram's expression order gives common call heads precedence that is
    // distinct from standalone symbol order; keep this as an explicit subset.
    match symbol.name() {
        "Association" => Some((0, 0)),
        "Power" => Some((1, 0)),
        "Times" => Some((1, 1)),
        "Plus" => Some((1, 2)),
        _ => None,
    }
}

fn compare_symbol_canonical(lhs: &Symbol, rhs: &Symbol) -> std::cmp::Ordering {
    let name_ordering = compare_symbol_names(lhs.name(), rhs.name());
    if name_ordering != std::cmp::Ordering::Equal {
        return name_ordering;
    }

    let context_ordering = lhs.context().cmp(rhs.context());
    if context_ordering != std::cmp::Ordering::Equal {
        return context_ordering;
    }

    lhs.identity().cmp(&rhs.identity())
}

fn compare_symbol_names(lhs: &str, rhs: &str) -> std::cmp::Ordering {
    let mut lhs_chars = lhs.chars();
    let mut rhs_chars = rhs.chars();

    loop {
        match (lhs_chars.next(), rhs_chars.next()) {
            (Some(lhs), Some(rhs)) => {
                let ordering = symbol_name_char_key(lhs).cmp(&symbol_name_char_key(rhs));
                if ordering != std::cmp::Ordering::Equal {
                    return ordering;
                }
            }
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (None, None) => return std::cmp::Ordering::Equal,
        }
    }
}

fn symbol_name_char_key(ch: char) -> (u8, u32, u8) {
    if ch.is_ascii_alphabetic() {
        return (
            0,
            ch.to_ascii_lowercase() as u32,
            u8::from(ch.is_ascii_uppercase()),
        );
    }

    if ch.is_ascii_digit() {
        return (1, ch as u32, 0);
    }

    (2, ch as u32, 0)
}

fn expr_symbol(expr: &Expr) -> Option<&Symbol> {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => Some(symbol),
        _ => None,
    }
}

fn compare_rational_values(lhs: &Rational, rhs: &Rational) -> std::cmp::Ordering {
    let lhs_cross = lhs.numerator() * rhs.denominator();
    let rhs_cross = rhs.numerator() * lhs.denominator();
    lhs_cross.cmp(&rhs_cross)
}

fn expr_order_rank(expr: &Expr) -> u8 {
    match expr {
        Expr::Atom(atom) => atom_order_rank(atom),
        Expr::Call { .. } => 3,
    }
}

fn atom_order_rank(atom: &Atom) -> u8 {
    match atom {
        Atom::Integer(_) | Atom::Rational(_) => 0,
        Atom::String(_) => 1,
        Atom::Symbol(_) => 2,
    }
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
