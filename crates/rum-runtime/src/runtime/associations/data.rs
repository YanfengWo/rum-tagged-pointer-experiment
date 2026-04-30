use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum KeyedCollectionKind {
    Association,
    RuleList,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct KeyedEntries {
    pub(in crate::runtime) kind: KeyedCollectionKind,
    pub(in crate::runtime) entries: Vec<(Expr, Expr)>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) struct AssociationEntryForm {
    pub(in crate::runtime) key: Expr,
    pub(in crate::runtime) value: Expr,
    pub(in crate::runtime) delayed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(in crate::runtime) enum KeyedEntriesListError {
    NotList,
    InvalidElement(Expr),
}

pub(in crate::runtime) fn association_entries_from_expr(expr: &Expr) -> Option<Vec<(Expr, Expr)>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "Association") {
        return None;
    }

    Some(canonicalize_association_entries(
        parse_association_entries_deep(args)?,
    ))
}

pub(in crate::runtime) fn rule_list_entries_from_expr(expr: &Expr) -> Option<Vec<(Expr, Expr)>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    parse_association_entries(args)
}

pub(in crate::runtime) fn keyed_entries_from_expr(expr: &Expr) -> Option<KeyedEntries> {
    if let Some(entries) = association_entries_from_expr(expr) {
        return Some(KeyedEntries {
            kind: KeyedCollectionKind::Association,
            entries,
        });
    }

    rule_list_entries_from_expr(expr).map(|entries| KeyedEntries {
        kind: KeyedCollectionKind::RuleList,
        entries,
    })
}

pub(in crate::runtime) fn single_rule_or_keyed_entries_from_expr(
    expr: &Expr,
) -> Option<Vec<(Expr, Expr)>> {
    if let Some(entry) = parse_association_entry(expr) {
        return Some(vec![entry]);
    }

    keyed_entries_from_expr(expr).map(|keyed| keyed.entries)
}

pub(in crate::runtime) fn single_rule_or_keyed_entry_forms_from_expr(
    expr: &Expr,
) -> Option<Vec<AssociationEntryForm>> {
    if let Some(entry) = parse_association_entry_form(expr) {
        return Some(vec![entry]);
    }

    if let Some(entries) = association_entry_forms_from_expr(expr) {
        return Some(entries);
    }

    rule_list_entry_forms_from_expr(expr)
}

pub(in crate::runtime) fn association_entry_forms_from_expr(
    expr: &Expr,
) -> Option<Vec<AssociationEntryForm>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "Association") {
        return None;
    }

    Some(canonicalize_association_entry_forms(
        parse_association_entry_forms_deep(args)?,
    ))
}

pub(in crate::runtime) fn keyed_entry_forms_from_expr(
    expr: &Expr,
) -> Option<Vec<AssociationEntryForm>> {
    if let Some(entries) = association_entry_forms_from_expr(expr) {
        return Some(entries);
    }

    rule_list_entry_forms_from_expr(expr)
}

pub(in crate::runtime) fn parse_association_entries(items: &[Expr]) -> Option<Vec<(Expr, Expr)>> {
    items.iter().map(parse_association_entry).collect()
}

pub(in crate::runtime) fn parse_association_entries_deep(
    items: &[Expr],
) -> Option<Vec<(Expr, Expr)>> {
    let mut entries = Vec::new();
    for item in items {
        entries.extend(parse_association_item_deep(item)?);
    }
    Some(entries)
}

fn parse_association_item_deep(expr: &Expr) -> Option<Vec<(Expr, Expr)>> {
    if let Some(entry) = parse_association_entry(expr) {
        return Some(vec![entry]);
    }

    if let Some(entries) = association_entries_from_expr(expr) {
        return Some(entries);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    parse_association_entries_deep(args)
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

pub(in crate::runtime) fn render_association_preserving_rule_heads(args: &[Expr]) -> Option<Expr> {
    Some(render_association_entry_forms(
        parse_association_entry_forms_deep(args)?,
    ))
}

pub(in crate::runtime) fn render_association_entry_forms_preserving_heads(
    entries: Vec<AssociationEntryForm>,
) -> Expr {
    render_association_entry_forms(entries)
}

pub(in crate::runtime) fn association_rule_exprs_preserving_heads(
    expr: &Expr,
) -> Option<Vec<Expr>> {
    Some(
        association_entry_forms_from_expr(expr)?
            .into_iter()
            .map(association_entry_form_expr)
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
        return Some(vec![key_spec_key_expr(expr)]);
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return Some(vec![key_spec_key_expr(expr)]);
    }

    Some(args.iter().map(key_spec_key_expr).collect())
}

fn key_spec_key_expr(expr: &Expr) -> Expr {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Key") && args.len() == 1 => {
            args[0].clone()
        }
        _ => expr.clone(),
    }
}

pub(in crate::runtime) fn parse_association_update_spec_forms(
    spec: &Expr,
) -> Option<Vec<AssociationEntryForm>> {
    if let Some(entry) = parse_association_entry_form(spec) {
        return Some(vec![entry]);
    }

    if let Some(entries) = association_entry_forms_from_expr(spec) {
        return Some(entries);
    }

    let Expr::Call { head, args } = spec else {
        return None;
    };
    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    parse_association_entry_forms_deep(args)
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

pub(in crate::runtime) fn parse_keyed_entries_list_expr_with_diagnostics(
    expr: &Expr,
) -> Result<Vec<Vec<(Expr, Expr)>>, KeyedEntriesListError> {
    let items = list_items(expr).ok_or(KeyedEntriesListError::NotList)?;
    items
        .iter()
        .map(|item| {
            single_rule_or_keyed_entries_from_expr(item)
                .ok_or_else(|| KeyedEntriesListError::InvalidElement(item.clone()))
        })
        .collect()
}

pub(in crate::runtime) fn parse_keyed_entry_forms_list_expr_with_diagnostics(
    expr: &Expr,
) -> Result<Vec<Vec<AssociationEntryForm>>, KeyedEntriesListError> {
    let items = list_items(expr).ok_or(KeyedEntriesListError::NotList)?;
    items
        .iter()
        .map(|item| {
            single_rule_or_keyed_entry_forms_from_expr(item)
                .ok_or_else(|| KeyedEntriesListError::InvalidElement(item.clone()))
        })
        .collect()
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

pub(in crate::runtime) fn parse_association_entry_form(
    expr: &Expr,
) -> Option<AssociationEntryForm> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return None;
    }

    Some(AssociationEntryForm {
        key: args[0].clone(),
        value: args[1].clone(),
        delayed: is_system_symbol_name(head, "RuleDelayed"),
    })
}

fn rule_list_entry_forms_from_expr(expr: &Expr) -> Option<Vec<AssociationEntryForm>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    parse_association_entry_forms(args)
}

fn parse_association_entry_forms(items: &[Expr]) -> Option<Vec<AssociationEntryForm>> {
    items.iter().map(parse_association_entry_form).collect()
}

fn parse_association_entry_forms_deep(items: &[Expr]) -> Option<Vec<AssociationEntryForm>> {
    let mut entries = Vec::new();
    for item in items {
        entries.extend(parse_association_item_forms_deep(item)?);
    }
    Some(entries)
}

fn parse_association_item_forms_deep(expr: &Expr) -> Option<Vec<AssociationEntryForm>> {
    if let Some(entry) = parse_association_entry_form(expr) {
        return Some(vec![entry]);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if is_system_symbol_name(head, "Association")
        || is_system_symbol_name(head, heads::structure::LIST)
    {
        return parse_association_entry_forms_deep(args);
    }

    None
}

fn render_association_entry_forms(entries: Vec<AssociationEntryForm>) -> Expr {
    let entries = canonicalize_association_entry_forms(entries);
    Expr::call(
        system_expr("Association"),
        entries
            .into_iter()
            .map(association_entry_form_expr)
            .collect(),
    )
}

fn association_entry_form_expr(entry: AssociationEntryForm) -> Expr {
    Expr::call(
        system_expr(if entry.delayed { "RuleDelayed" } else { "Rule" }),
        vec![entry.key, entry.value],
    )
}

fn canonicalize_association_entry_forms(
    entries: Vec<AssociationEntryForm>,
) -> Vec<AssociationEntryForm> {
    let mut canonical: Vec<AssociationEntryForm> = Vec::new();
    for entry in entries {
        if let Some(existing) = canonical
            .iter_mut()
            .find(|existing| existing.key == entry.key)
        {
            *existing = entry;
        } else {
            canonical.push(entry);
        }
    }

    canonical
}

pub(in crate::runtime) fn predicate_bool_expr(value: bool) -> Expr {
    system_expr(if value {
        heads::structure::TRUE
    } else {
        heads::structure::FALSE
    })
}

pub(in crate::runtime) fn invalid_association_message(expr: &Expr) -> String {
    format!("The argument {expr} is not a valid Association.")
}

pub(in crate::runtime) fn invalid_association_or_rule_list_message(expr: &Expr) -> String {
    format!("The argument {expr} is not a valid Association or a list of rules.")
}

pub(in crate::runtime) fn invalid_association_or_rule_message(expr: &Expr) -> String {
    format!("The argument {expr} is not a valid Association or rule.")
}

pub(in crate::runtime) fn invalid_rule_message(expr: &Expr) -> String {
    format!("The argument {expr} is not a valid rule.")
}

pub(in crate::runtime) fn invalid_association_or_list_message(expr: &Expr) -> String {
    format!("The argument {expr} is not a valid Association or a list.")
}

pub(in crate::runtime) fn invalid_keyed_collection_list_message(expr: &Expr) -> String {
    format!("The argument {expr} is not a valid list of Associations or rules or lists of rules.")
}

pub(in crate::runtime) fn invalid_keyed_collection_outer_list_message(expr: &Expr) -> String {
    format!("The argument {expr} is not a valid list of Associations or rules.")
}

pub(in crate::runtime) fn invalid_association_update_spec_message(expr: &Expr) -> String {
    format!("The argument {expr} is not a list, Rule or Association.")
}

pub(in crate::runtime) fn missing_immediate_value_message(
    symbol: &Symbol,
    position: usize,
) -> String {
    format!("The symbol {symbol} at position {position} should have an immediate value defined.")
}

pub(in crate::runtime) fn list_expected_message(position: usize, expr: &Expr) -> String {
    format!("List expected at position {position} in {expr}.")
}

pub(in crate::runtime) fn nonempty_list_message(expr: &Expr) -> String {
    format!("Argument {expr} should be a nonempty list.")
}

pub(in crate::runtime) fn same_length_message(lhs: &Expr, rhs: &Expr) -> String {
    format!("{lhs} and {rhs} must have the same length.")
}

pub(in crate::runtime) fn protected_tag_write_message(expr: &Expr) -> Option<String> {
    let Expr::Call { head, .. } = expr else {
        return None;
    };

    Some(format!("Tag {head} in {expr} is Protected."))
}

pub(in crate::runtime) fn keyed_operator_form_slot_first(symbol: &str, tail: Expr) -> Expr {
    let parameter = Expr::symbol("rum$keyedArg");
    Expr::call(
        system_expr("Function"),
        vec![
            parameter.clone(),
            Expr::call(system_expr(symbol), vec![parameter, tail]),
        ],
    )
}

pub(in crate::runtime) fn keyed_operator_form_slot_second(symbol: &str, head_arg: Expr) -> Expr {
    let parameter = Expr::symbol("rum$keyedArg");
    Expr::call(
        system_expr("Function"),
        vec![
            parameter.clone(),
            Expr::call(system_expr(symbol), vec![head_arg, parameter]),
        ],
    )
}
