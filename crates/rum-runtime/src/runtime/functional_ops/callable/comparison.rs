use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ComparisonOperatorKind {
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    Equal,
    Unequal,
    Same,
}

#[derive(Clone, Debug)]
struct ComparisonOperatorSpec<'a> {
    kind: ComparisonOperatorKind,
    bound: &'a Expr,
}

impl ComparisonOperatorKind {
    fn wrapper_symbol(self) -> &'static str {
        match self {
            Self::Less => "LessThan",
            Self::Greater => "GreaterThan",
            Self::LessEqual => "LessEqualThan",
            Self::GreaterEqual => "GreaterEqualThan",
            Self::Equal => "EqualTo",
            Self::Unequal => "UnequalTo",
            Self::Same => "SameAs",
        }
    }
}

pub(super) fn is_comparison_operator_wrapper(expr: &Expr) -> bool {
    parse_comparison_operator_wrapper(expr).is_some()
}

pub(super) fn invalid_comparison_operator_wrapper_argx_symbol(
    head: &Expr,
    args: &[Expr],
) -> Option<&'static str> {
    let spec = parse_comparison_operator_wrapper(head)?;
    if args.len() == 1 {
        return None;
    }

    match spec.kind {
        ComparisonOperatorKind::Less
        | ComparisonOperatorKind::Greater
        | ComparisonOperatorKind::LessEqual
        | ComparisonOperatorKind::GreaterEqual => Some(spec.kind.wrapper_symbol()),
        ComparisonOperatorKind::Equal
        | ComparisonOperatorKind::Unequal
        | ComparisonOperatorKind::Same => None,
    }
}

pub(super) fn apply_comparison_operator_wrapper_to_args(
    head: &Expr,
    args: &[Expr],
) -> Option<Expr> {
    let spec = parse_comparison_operator_wrapper(head)?;
    let head = match spec.kind {
        ComparisonOperatorKind::Less => "Less",
        ComparisonOperatorKind::Greater => "Greater",
        ComparisonOperatorKind::LessEqual => "LessEqual",
        ComparisonOperatorKind::GreaterEqual => "GreaterEqual",
        ComparisonOperatorKind::Equal => "Equal",
        ComparisonOperatorKind::Unequal => "Unequal",
        ComparisonOperatorKind::Same => "SameQ",
    };

    match spec.kind {
        ComparisonOperatorKind::Equal
        | ComparisonOperatorKind::Unequal
        | ComparisonOperatorKind::Same => {
            let mut comparison_args = args.to_vec();
            comparison_args.push(spec.bound.clone());
            Some(Expr::call(system_expr(head), comparison_args))
        }
        ComparisonOperatorKind::Less
        | ComparisonOperatorKind::Greater
        | ComparisonOperatorKind::LessEqual
        | ComparisonOperatorKind::GreaterEqual => {
            let [arg] = args else {
                return None;
            };
            Some(Expr::call(
                system_expr(head),
                vec![arg.clone(), spec.bound.clone()],
            ))
        }
    }
}

fn parse_comparison_operator_wrapper(expr: &Expr) -> Option<ComparisonOperatorSpec<'_>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    let [bound] = args.as_slice() else {
        return None;
    };

    let kind = if is_system_symbol_name(head, "LessThan") {
        ComparisonOperatorKind::Less
    } else if is_system_symbol_name(head, "GreaterThan") {
        ComparisonOperatorKind::Greater
    } else if is_system_symbol_name(head, "LessEqualThan") {
        ComparisonOperatorKind::LessEqual
    } else if is_system_symbol_name(head, "GreaterEqualThan") {
        ComparisonOperatorKind::GreaterEqual
    } else if is_system_symbol_name(head, "EqualTo") {
        ComparisonOperatorKind::Equal
    } else if is_system_symbol_name(head, "UnequalTo") {
        ComparisonOperatorKind::Unequal
    } else if is_system_symbol_name(head, "SameAs") {
        ComparisonOperatorKind::Same
    } else {
        return None;
    };

    Some(ComparisonOperatorSpec { kind, bound })
}
