use super::*;

pub(in crate::runtime) fn association_entries_from_expr(expr: &Expr) -> Option<Vec<(Expr, Expr)>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "Association") {
        return None;
    }

    Some(canonicalize_association_entries(parse_association_entries(
        args,
    )?))
}

pub(in crate::runtime) fn parse_association_entries(items: &[Expr]) -> Option<Vec<(Expr, Expr)>> {
    items.iter().map(parse_association_entry).collect()
}

pub(in crate::runtime) fn render_association(entries: Vec<(Expr, Expr)>) -> Expr {
    let entries = canonicalize_association_entries(entries);
    Expr::call(
        system_expr("Association"),
        entries
            .into_iter()
            .map(|(key, value)| Expr::call(system_expr("Rule"), vec![key, value]))
            .collect(),
    )
}

pub(in crate::runtime) fn association_value_by_key(expr: &Expr, key: &Expr) -> Option<Expr> {
    association_entries_from_expr(expr)?
        .into_iter()
        .find(|(entry_key, _)| entry_key == key)
        .map(|(_, value)| value)
}

pub(in crate::runtime) fn association_position_and_value_by_key(
    expr: &Expr,
    key: &Expr,
) -> Option<(isize, Expr)> {
    association_entries_from_expr(expr)?
        .into_iter()
        .enumerate()
        .find(|(_, (entry_key, _))| entry_key == key)
        .map(|(index, (_, value))| (index as isize + 1, value))
}

pub(in crate::runtime) fn parse_key_spec(expr: &Expr) -> Option<Vec<Expr>> {
    let Expr::Call { head, args } = expr else {
        return Some(vec![expr.clone()]);
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return Some(vec![expr.clone()]);
    }

    Some(args.clone())
}

pub(in crate::runtime) fn parse_association_update_spec(spec: &Expr) -> Option<Vec<(Expr, Expr)>> {
    if let Some(entry) = parse_association_entry(spec) {
        return Some(vec![entry]);
    }

    let Expr::Call { head, args } = spec else {
        return None;
    };
    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    parse_association_entries(args)
}

pub(in crate::runtime) fn normalize_key_value_pattern_spec(spec: &Expr) -> Option<Expr> {
    if let Some((key, value)) = parse_association_entry(spec) {
        return Some(Expr::call(
            system_expr("List"),
            vec![Expr::call(system_expr("Rule"), vec![key, value])],
        ));
    }

    let Expr::Call { head, args } = spec else {
        return None;
    };
    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    let entries = parse_association_entries(args)?;
    Some(Expr::call(
        system_expr("List"),
        entries
            .into_iter()
            .map(|(key, value)| Expr::call(system_expr("Rule"), vec![key, value]))
            .collect(),
    ))
}

pub(in crate::runtime) fn list_items(expr: &Expr) -> Option<&[Expr]> {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, heads::structure::LIST) => {
            Some(args)
        }
        _ => None,
    }
}

pub(in crate::runtime) fn parse_association_args(args: &[Expr]) -> Option<Vec<Vec<(Expr, Expr)>>> {
    args.iter().map(association_entries_from_expr).collect()
}

pub(in crate::runtime) fn parse_association_list_expr(
    expr: &Expr,
) -> Option<Vec<Vec<(Expr, Expr)>>> {
    let items = list_items(expr)?;
    items.iter().map(association_entries_from_expr).collect()
}

pub(in crate::runtime) fn increment_count(counts: &mut Vec<(Expr, usize)>, key: Expr) {
    if let Some((_, count)) = counts.iter_mut().find(|(existing, _)| existing == &key) {
        *count += 1;
    } else {
        counts.push((key, 1));
    }
}

fn canonicalize_association_entries(entries: Vec<(Expr, Expr)>) -> Vec<(Expr, Expr)> {
    let mut canonical = Vec::new();

    for (key, value) in entries {
        if let Some((_, existing_value)) = canonical
            .iter_mut()
            .find(|(existing_key, _)| existing_key == &key)
        {
            *existing_value = value;
        } else {
            canonical.push((key, value));
        }
    }

    canonical
}

pub(in crate::runtime) fn parse_association_entry(expr: &Expr) -> Option<(Expr, Expr)> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return None;
    }

    Some((args[0].clone(), args[1].clone()))
}

pub(in crate::runtime) fn predicate_bool_expr(value: bool) -> Expr {
    system_expr(if value {
        heads::structure::TRUE
    } else {
        heads::structure::FALSE
    })
}
