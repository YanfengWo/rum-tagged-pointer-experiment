use super::*;

pub(super) fn compile_matcher(expr: &Expr) -> MatcherNode {
    let expr = strip_hold_pattern(expr);

    if let Some((name, inner)) = pattern_named_pattern(expr) {
        return MatcherNode::Named {
            name: name.clone(),
            inner: Box::new(compile_matcher(inner)),
        };
    }

    if let Some((inner, condition)) = pattern_condition(expr) {
        return MatcherNode::Condition {
            inner: Box::new(compile_matcher(inner)),
            condition: condition.clone(),
        };
    }

    if let Some(blank_head) = pattern_blank_head(expr) {
        return MatcherNode::Blank {
            head_constraint: blank_head.cloned(),
        };
    }

    match expr {
        Expr::Atom(_) => MatcherNode::Exact(expr.clone()),
        Expr::Call { head, args } => MatcherNode::Call {
            head: Box::new(compile_matcher(head)),
            args: compile_matcher_args(args),
        },
    }
}

pub(super) fn compile_matcher_args(args: &[Expr]) -> Vec<MatcherArg> {
    args.iter()
        .map(|arg| {
            if let Some(spec) = decompose_sequence_pattern(arg) {
                MatcherArg::Sequence(CompiledSequencePattern {
                    min_len: spec.min_len,
                    head_constraint: spec.head_constraint.cloned(),
                    name: spec.name.cloned(),
                    condition: spec.condition.cloned(),
                })
            } else {
                MatcherArg::Single(compile_matcher(arg))
            }
        })
        .collect()
}

pub(super) fn pattern_named_pattern(expr: &Expr) -> Option<(&Symbol, &Expr)> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Pattern") && args.len() == 2 => {
            let name = expr_as_symbol(&args[0])?;
            Some((name, &args[1]))
        }
        _ => None,
    }
}

pub(super) fn pattern_condition(expr: &Expr) -> Option<(&Expr, &Expr)> {
    match expr {
        Expr::Call { head, args }
            if is_system_symbol_name(head, "Condition") && args.len() == 2 =>
        {
            Some((&args[0], &args[1]))
        }
        _ => None,
    }
}

pub(super) fn decompose_sequence_pattern(expr: &Expr) -> Option<SequencePatternSpec<'_>> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Pattern") && args.len() == 2 => {
            let mut spec = decompose_sequence_pattern(&args[1])?;
            let name = expr_as_symbol(&args[0])?;

            if spec.name.is_some() {
                return None;
            }

            spec.name = Some(name);
            Some(spec)
        }
        Expr::Call { head, args }
            if is_system_symbol_name(head, "Condition") && args.len() == 2 =>
        {
            let mut spec = decompose_sequence_pattern(&args[0])?;

            if spec.condition.is_some() {
                return None;
            }

            spec.condition = Some(&args[1]);
            Some(spec)
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "BlankSequence") => {
            if args.len() > 1 {
                return None;
            }

            Some(SequencePatternSpec {
                min_len: 1,
                head_constraint: args.first(),
                name: None,
                condition: None,
            })
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "BlankNullSequence") => {
            if args.len() > 1 {
                return None;
            }

            Some(SequencePatternSpec {
                min_len: 0,
                head_constraint: args.first(),
                name: None,
                condition: None,
            })
        }
        _ => None,
    }
}

#[allow(dead_code)]
pub(super) fn min_required_arg_count(patterns: &[Expr]) -> usize {
    patterns
        .iter()
        .map(|pattern| {
            decompose_sequence_pattern(pattern)
                .map(|spec| spec.min_len)
                .unwrap_or(1)
        })
        .sum()
}

pub(super) fn min_required_matcher_arg_count(patterns: &[MatcherArg]) -> usize {
    patterns
        .iter()
        .map(|pattern| match pattern {
            MatcherArg::Single(_) => 1,
            MatcherArg::Sequence(spec) => spec.min_len,
        })
        .sum()
}

pub(super) fn pattern_blank_head(expr: &Expr) -> Option<Option<&Expr>> {
    match expr {
        Expr::Call { head, args }
            if is_system_symbol_name(head, "Blank")
                || is_system_symbol_name(head, "BlankSequence")
                || is_system_symbol_name(head, "BlankNullSequence") =>
        {
            if args.is_empty() {
                Some(None)
            } else if args.len() == 1 {
                Some(Some(&args[0]))
            } else {
                None
            }
        }
        _ => None,
    }
}

pub(super) fn match_blank(head_constraint: Option<&Expr>, value: &Expr) -> bool {
    match head_constraint {
        None => true,
        Some(head_constraint) => expr_has_head(value, head_constraint),
    }
}

pub(super) fn expr_has_head(value: &Expr, head_constraint: &Expr) -> bool {
    let Some(expected_symbol) = expr_as_symbol(head_constraint) else {
        return false;
    };

    match value {
        Expr::Atom(Atom::Integer(_)) => expected_symbol.name() == "Integer",
        Expr::Atom(Atom::String(_)) => expected_symbol.name() == "String",
        Expr::Atom(Atom::Symbol(_)) => expected_symbol.name() == "Symbol",
        Expr::Call { head, .. } => expr_as_symbol(head)
            .map(|actual| actual == expected_symbol)
            .unwrap_or(false),
    }
}

pub(super) fn matcher_head_has_attribute(
    head: &MatcherNode,
    session: &Session,
    attribute: Attribute,
) -> bool {
    match head {
        MatcherNode::Exact(Expr::Atom(Atom::Symbol(symbol))) => {
            session.attribute_set_for_symbol(symbol).contains(attribute)
        }
        _ => false,
    }
}
