use super::*;

pub(super) fn parse_replace_part_rules(
    expr: &Expr,
    heads: ReplacePartHeadsMode,
) -> Option<Vec<ReplacePartRule>> {
    if let Some(rule) = parse_replace_part_rule_with_heads(expr, heads) {
        return Some(vec![rule]);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    args.iter()
        .map(|item| parse_replace_part_rule_with_heads(item, heads))
        .collect()
}

fn parse_replace_part_rule_with_heads(
    expr: &Expr,
    heads: ReplacePartHeadsMode,
) -> Option<ReplacePartRule> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    let delayed = if is_system_symbol_name(head, heads::rules::RULE) && args.len() == 2 {
        false
    } else if is_system_symbol_name(head, heads::rules::RULE_DELAYED) && args.len() == 2 {
        true
    } else {
        return None;
    };

    Some(ReplacePartRule {
        position_specs: parse_replace_part_position_specs(&args[0])?,
        rhs: args[1].clone(),
        delayed,
        heads,
    })
}

fn parse_replace_part_position_specs(expr: &Expr) -> Option<Vec<ReplacePartPositionSpec>> {
    if is_empty_plain_list_expr(expr) {
        return Some(Vec::new());
    }

    if let Some(position_spec) = parse_replace_part_exact_position_spec(expr) {
        return Some(vec![ReplacePartPositionSpec::Exact(position_spec)]);
    }

    let Expr::Call { head, args } = expr else {
        return Some(vec![ReplacePartPositionSpec::Pattern(expr.clone())]);
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return Some(vec![ReplacePartPositionSpec::Pattern(expr.clone())]);
    }

    if !args.iter().any(is_plain_list_expr) {
        return Some(vec![ReplacePartPositionSpec::Pattern(expr.clone())]);
    }

    Some(
        args.iter()
            .map(|item| {
                parse_replace_part_exact_position_spec(item)
                    .map(ReplacePartPositionSpec::Exact)
                    .unwrap_or_else(|| ReplacePartPositionSpec::Pattern(item.clone()))
            })
            .collect(),
    )
}

fn parse_replace_part_exact_position_spec(expr: &Expr) -> Option<Vec<PartSelector>> {
    if let Some(selector) = parse_replace_part_exact_selector(expr) {
        return Some(vec![selector]);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    if args.is_empty() {
        return Some(Vec::new());
    }

    if args.iter().all(is_plain_list_expr) {
        return None;
    }

    args.iter().map(parse_replace_part_exact_selector).collect()
}

fn parse_replace_part_exact_selector(expr: &Expr) -> Option<PartSelector> {
    if let Some(index) = expr_as_signed_isize(expr) {
        return Some(PartSelector::Single(AtomicPartSpec::Index(index)));
    }

    parse_replace_part_key_selector(expr).map(PartSelector::Key)
}

fn parse_replace_part_key_selector(expr: &Expr) -> Option<Expr> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "Key") || args.len() != 1 {
        return None;
    }

    Some(args[0].clone())
}

pub(super) fn parse_replace_part_heads_option(expr: &Expr) -> Option<ReplacePartHeadsMode> {
    match parse_heads_option(expr)? {
        true => Some(ReplacePartHeadsMode::Include),
        false => Some(ReplacePartHeadsMode::Exclude),
    }
}
