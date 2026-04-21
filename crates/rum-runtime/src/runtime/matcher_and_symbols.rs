use super::*;

pub(super) fn compile_matcher(expr: &Expr) -> MatcherNode {
    let expr = strip_hold_pattern(expr);

    if let Some(inner) = pattern_verbatim_inner(expr) {
        return MatcherNode::Exact(inner.clone());
    }

    if let Some(entries) = pattern_key_value_pattern_entries(expr) {
        return MatcherNode::KeyValuePattern(
            entries
                .into_iter()
                .map(|(key, value)| (compile_matcher(&key), compile_matcher(&value)))
                .collect(),
        );
    }

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

    if let Some((inner, test)) = pattern_test(expr) {
        return MatcherNode::PatternTest {
            inner: Box::new(compile_matcher(inner)),
            test: test.clone(),
        };
    }

    if let Some((excluded, inner)) = pattern_except(expr) {
        return MatcherNode::Except {
            excluded: Box::new(compile_matcher(excluded)),
            inner: Box::new(
                inner
                    .map(compile_matcher)
                    .unwrap_or_else(|| MatcherNode::Blank {
                        head_constraint: None,
                    }),
            ),
        };
    }

    if let Some((inner, _)) = pattern_optional(expr) {
        return compile_matcher(inner);
    }

    if let Some(alternatives) = pattern_alternatives(expr) {
        return MatcherNode::Alternatives(alternatives.iter().map(compile_matcher).collect());
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
            args: compile_matcher_args(head.as_ref(), args),
        },
    }
}

pub(super) fn compile_matcher_args(head: &Expr, args: &[Expr]) -> Vec<MatcherArg> {
    let mut compiled = Vec::new();
    let mut position = 1;

    for arg in args {
        compile_matcher_arg_items(head, arg, &mut position, &mut compiled);
    }

    compiled
}

fn compile_matcher_arg(expr: &Expr, implicit_default: CompiledOptionalDefault) -> MatcherArg {
    let expr = strip_hold_pattern(expr);

    if let Some((inner, default)) = pattern_optional(expr) {
        return MatcherArg::Optional(CompiledOptionalPattern {
            inner: Box::new(compile_matcher_arg(
                inner,
                CompiledOptionalDefault::UnavailableImplicit,
            )),
            default: default
                .cloned()
                .map(CompiledOptionalDefault::Explicit)
                .unwrap_or(implicit_default),
        });
    }

    if let Some(spec) = decompose_sequence_pattern(expr) {
        MatcherArg::Sequence(spec)
    } else {
        MatcherArg::Single(compile_matcher(expr))
    }
}

fn compile_matcher_arg_items(
    head: &Expr,
    expr: &Expr,
    position: &mut usize,
    compiled: &mut Vec<MatcherArg>,
) {
    let expr = strip_hold_pattern(expr);

    if let Some(items) = pattern_sequence_items(expr) {
        compiled.push(MatcherArg::PatternSequenceGroup(
            compile_matcher_group_items(head, items, position),
        ));
        return;
    }

    if let Some(items) = pattern_orderless_sequence_items(expr) {
        compiled.push(MatcherArg::OrderlessPatternSequenceGroup(
            compile_matcher_group_items(head, items, position),
        ));
        return;
    }

    compiled.push(compile_matcher_arg(
        expr,
        CompiledOptionalDefault::Implicit {
            head: head.clone(),
            position: *position,
        },
    ));
    *position += 1;
}

fn compile_matcher_group_items(
    head: &Expr,
    items: &[Expr],
    position: &mut usize,
) -> Vec<MatcherArg> {
    let mut compiled = Vec::new();

    for item in items {
        compile_matcher_arg_items(head, item, position, &mut compiled);
    }

    compiled
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

pub(super) fn pattern_test(expr: &Expr) -> Option<(&Expr, &Expr)> {
    match expr {
        Expr::Call { head, args }
            if is_system_symbol_name(head, "PatternTest") && args.len() == 2 =>
        {
            Some((&args[0], &args[1]))
        }
        _ => None,
    }
}

pub(super) fn pattern_except(expr: &Expr) -> Option<(&Expr, Option<&Expr>)> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Except") && args.len() == 1 => {
            Some((&args[0], None))
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Except") && args.len() == 2 => {
            Some((&args[0], Some(&args[1])))
        }
        _ => None,
    }
}

pub(super) fn pattern_optional(expr: &Expr) -> Option<(&Expr, Option<&Expr>)> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Optional") && args.len() == 1 => {
            Some((&args[0], None))
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Optional") && args.len() == 2 => {
            Some((&args[0], Some(&args[1])))
        }
        _ => None,
    }
}

pub(super) fn pattern_key_value_pattern_entries(expr: &Expr) -> Option<Vec<(Expr, Expr)>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "KeyValuePattern") || args.len() != 1 {
        return None;
    }

    let normalized_spec = normalize_key_value_pattern_spec(&args[0])?;
    let Expr::Call {
        head: list_head,
        args: items,
    } = normalized_spec
    else {
        return None;
    };

    if !is_system_symbol_name(&list_head, heads::structure::LIST) {
        return None;
    }

    items
        .into_iter()
        .map(|item| parse_association_entry(&item))
        .collect()
}

pub(super) fn pattern_alternatives(expr: &Expr) -> Option<&[Expr]> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Alternatives") => Some(args),
        _ => None,
    }
}

pub(super) fn pattern_sequence_items(expr: &Expr) -> Option<&[Expr]> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "PatternSequence") => Some(args),
        _ => None,
    }
}

pub(super) fn pattern_orderless_sequence_items(expr: &Expr) -> Option<&[Expr]> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "OrderlessPatternSequence") => {
            Some(args)
        }
        _ => None,
    }
}

pub(super) fn pattern_verbatim_inner(expr: &Expr) -> Option<&Expr> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Verbatim") && args.len() == 1 => {
            Some(&args[0])
        }
        _ => None,
    }
}

pub(super) fn decompose_sequence_pattern(expr: &Expr) -> Option<CompiledSequencePattern> {
    let expr = strip_hold_pattern(expr);

    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Pattern") && args.len() == 2 => {
            let mut spec = decompose_sequence_pattern(&args[1])?;
            let name = expr_as_symbol(&args[0])?;

            if spec.name.is_some() {
                return None;
            }

            spec.name = Some(name.clone());
            Some(spec)
        }
        Expr::Call { head, args }
            if is_system_symbol_name(head, "Condition") && args.len() == 2 =>
        {
            let mut spec = decompose_sequence_pattern(&args[0])?;
            spec.checks
                .push(SequencePatternCheck::Condition(args[1].clone()));
            Some(spec)
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Longest") && args.len() == 1 => {
            let mut spec = decompose_sequence_pattern(&args[0])?;
            spec.greediness = SequenceGreediness::Longest;
            Some(spec)
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Shortest") && args.len() == 1 => {
            let mut spec = decompose_sequence_pattern(&args[0])?;
            spec.greediness = SequenceGreediness::Shortest;
            Some(spec)
        }
        Expr::Call { head, args }
            if is_system_symbol_name(head, "PatternTest") && args.len() == 2 =>
        {
            let mut spec = decompose_sequence_pattern(&args[0])?;
            spec.checks
                .push(SequencePatternCheck::PatternTest(args[1].clone()));
            Some(spec)
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "BlankSequence") => {
            if args.len() > 1 {
                return None;
            }

            Some(CompiledSequencePattern {
                min_len: 1,
                max_len: None,
                matcher: MatcherNode::Blank {
                    head_constraint: args.first().cloned(),
                },
                name: None,
                checks: Vec::new(),
                greediness: SequenceGreediness::Shortest,
            })
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "BlankNullSequence") => {
            if args.len() > 1 {
                return None;
            }

            Some(CompiledSequencePattern {
                min_len: 0,
                max_len: None,
                matcher: MatcherNode::Blank {
                    head_constraint: args.first().cloned(),
                },
                name: None,
                checks: Vec::new(),
                greediness: SequenceGreediness::Shortest,
            })
        }
        expr if is_options_pattern_expr(expr) => Some(options_pattern_sequence_matcher()),
        Expr::Call { head, args } if is_system_symbol_name(head, "Repeated") => {
            compile_repeated_pattern(args, 1)
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "RepeatedNull") => {
            compile_repeated_pattern(args, 0)
        }
        _ => None,
    }
}

fn compile_repeated_pattern(
    args: &[Expr],
    default_min_len: usize,
) -> Option<CompiledSequencePattern> {
    let (inner, bounds_expr) = match args {
        [inner] => (inner, None),
        [inner, bounds] => (inner, Some(bounds)),
        _ => return None,
    };

    let (min_len, max_len) = match bounds_expr {
        Some(bounds) => parse_repeated_bounds(bounds)?,
        None => (default_min_len, None),
    };

    Some(CompiledSequencePattern {
        min_len,
        max_len,
        matcher: compile_matcher(inner),
        name: None,
        checks: Vec::new(),
        greediness: SequenceGreediness::Shortest,
    })
}

fn parse_repeated_bounds(expr: &Expr) -> Option<(usize, Option<usize>)> {
    if let Some(count) = expr_as_non_negative_usize(expr) {
        return Some((count, Some(count)));
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "List") {
        return None;
    }

    match args.as_slice() {
        [single] => {
            let count = expr_as_non_negative_usize(single)?;
            Some((count, Some(count)))
        }
        [min, max] => {
            let min = expr_as_non_negative_usize(min)?;
            let max = if let Some(max) = expr_as_non_negative_usize(max) {
                Some(max)
            } else if is_system_symbol_name(max, "Infinity") {
                None
            } else {
                return None;
            };

            if let Some(max) = max {
                if max < min {
                    return None;
                }
            }

            Some((min, max))
        }
        _ => None,
    }
}

#[allow(dead_code)]
pub(super) fn min_required_arg_count(patterns: &[Expr]) -> usize {
    patterns
        .iter()
        .map(|pattern| {
            if pattern_optional(strip_hold_pattern(pattern)).is_some() {
                0
            } else {
                decompose_sequence_pattern(pattern)
                    .map(|spec| spec.min_len)
                    .unwrap_or(1)
            }
        })
        .sum()
}

pub(super) fn min_required_matcher_arg_count(patterns: &[MatcherArg]) -> usize {
    patterns
        .iter()
        .map(|pattern| match pattern {
            MatcherArg::Single(_) => 1,
            MatcherArg::Sequence(spec) => spec.min_len,
            MatcherArg::Optional(_) => 0,
            MatcherArg::PatternSequenceGroup(items)
            | MatcherArg::OrderlessPatternSequenceGroup(items) => {
                min_required_matcher_arg_count(items)
            }
        })
        .sum()
}

pub(super) fn max_possible_matcher_arg_count(patterns: &[MatcherArg]) -> Option<usize> {
    let mut total = 0usize;

    for pattern in patterns {
        let count = match pattern {
            MatcherArg::Single(_) => Some(1),
            MatcherArg::Sequence(spec) => spec.max_len,
            MatcherArg::Optional(spec) => {
                max_possible_matcher_arg_count(std::slice::from_ref(spec.inner.as_ref()))
            }
            MatcherArg::PatternSequenceGroup(items)
            | MatcherArg::OrderlessPatternSequenceGroup(items) => {
                max_possible_matcher_arg_count(items)
            }
        }?;

        total = total.checked_add(count)?;
    }

    Some(total)
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
        Expr::Atom(Atom::Rational(_)) => expected_symbol.name() == "Rational",
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
