use std::collections::HashMap;

use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum JoinType {
    Inner,
    Left,
    Right,
    Outer,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct JoinKeySpec {
    left: Vec<Expr>,
    right: Vec<Expr>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct JoinRow {
    expr: Expr,
    entries: Vec<AssociationEntryForm>,
}

pub(crate) fn eval_join_across(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let (left_expr, right_expr, key_spec_expr, join_type_expr) = match args {
        [left, right, key_spec] => (left, right, key_spec, None),
        [left, right, key_spec, join_type] => (left, right, key_spec, Some(join_type)),
        _ => {
            return Ok(session.invalid_call_with_message(
                "JoinAcross",
                args,
                "argb",
                join_across_argument_count_message(args.len()),
            ));
        }
    };

    let Some(left_rows) = join_rows(left_expr) else {
        return Ok(session.invalid_call_with_message(
            "JoinAcross",
            args,
            "lists",
            invalid_join_rows_message(1, args),
        ));
    };
    let Some(right_rows) = join_rows(right_expr) else {
        return Ok(session.invalid_call_with_message(
            "JoinAcross",
            args,
            "lists",
            invalid_join_rows_message(2, args),
        ));
    };
    let Some(key_spec) = parse_join_key_spec(key_spec_expr) else {
        return Ok(session.invalid_call_with_message(
            "JoinAcross",
            args,
            "keys",
            invalid_join_key_spec_message(key_spec_expr),
        ));
    };
    let join_type = match join_type_expr {
        None => JoinType::Inner,
        Some(expr) => {
            let Some(join_type) = parse_join_type(expr) else {
                return Ok(session.invalid_call_with_message(
                    "JoinAcross",
                    args,
                    "jtype",
                    invalid_join_type_message(expr),
                ));
            };
            join_type
        }
    };

    let rows = match join_type {
        JoinType::Inner => join_left_major(&left_rows, &right_rows, &key_spec, false, false),
        JoinType::Left => join_left_major(&left_rows, &right_rows, &key_spec, true, false),
        JoinType::Right => join_right_major(&left_rows, &right_rows, &key_spec),
        JoinType::Outer => join_left_major(&left_rows, &right_rows, &key_spec, true, true),
    };
    Ok(Expr::call(system_expr(heads::structure::LIST), rows))
}

fn join_rows(expr: &Expr) -> Option<Vec<JoinRow>> {
    list_items(expr)?
        .iter()
        .map(|row| {
            Some(JoinRow {
                expr: row.clone(),
                entries: association_entry_forms_from_expr(row)?,
            })
        })
        .collect()
}

fn parse_join_key_spec(expr: &Expr) -> Option<JoinKeySpec> {
    let components = if let Some(items) = list_items(expr) {
        if items.is_empty() {
            return None;
        }
        items
    } else {
        std::slice::from_ref(expr)
    };

    let mut left = Vec::with_capacity(components.len());
    let mut right = Vec::with_capacity(components.len());
    for component in components {
        let (left_key, right_key) = parse_join_key_component(component)?;
        left.push(left_key);
        right.push(right_key);
    }
    Some(JoinKeySpec { left, right })
}

fn parse_join_key_component(expr: &Expr) -> Option<(Expr, Expr)> {
    if let Expr::Call { head, args } = expr {
        if is_system_symbol_name(head, "Rule") && args.len() == 2 {
            return Some((parse_join_key(&args[0])?, parse_join_key(&args[1])?));
        }
    }

    let key = parse_join_key(expr)?;
    Some((key.clone(), key))
}

fn parse_join_key(expr: &Expr) -> Option<Expr> {
    match expr {
        Expr::Atom(Atom::String(_)) => Some(expr.clone()),
        Expr::Call { head, args } if is_system_symbol_name(head, "Key") && args.len() == 1 => {
            Some(args[0].clone())
        }
        _ => None,
    }
}

fn parse_join_type(expr: &Expr) -> Option<JoinType> {
    let Expr::Atom(Atom::String(value)) = expr else {
        return None;
    };

    match value.as_str() {
        "Inner" => Some(JoinType::Inner),
        "Left" => Some(JoinType::Left),
        "Right" => Some(JoinType::Right),
        "Outer" => Some(JoinType::Outer),
        _ => None,
    }
}

fn join_left_major(
    left_rows: &[JoinRow],
    right_rows: &[JoinRow],
    key_spec: &JoinKeySpec,
    include_unmatched_left: bool,
    append_unmatched_right: bool,
) -> Vec<Expr> {
    let right_index = index_rows(right_rows, &key_spec.right);
    let mut matched_right = vec![false; right_rows.len()];
    let mut joined = Vec::new();

    for left_row in left_rows {
        let matching_right = join_value_tuple(left_row, &key_spec.left)
            .and_then(|join_values| right_index.get(&join_values));
        if let Some(right_indices) = matching_right {
            for &right_index in right_indices {
                joined.push(join_rows_as_association(left_row, &right_rows[right_index]));
                matched_right[right_index] = true;
            }
        } else if include_unmatched_left {
            joined.push(left_row.expr.clone());
        }
    }

    if append_unmatched_right {
        joined.extend(
            right_rows
                .iter()
                .zip(matched_right)
                .filter(|(_, matched)| !matched)
                .map(|(row, _)| row.expr.clone()),
        );
    }

    joined
}

fn join_right_major(
    left_rows: &[JoinRow],
    right_rows: &[JoinRow],
    key_spec: &JoinKeySpec,
) -> Vec<Expr> {
    let left_index = index_rows(left_rows, &key_spec.left);
    let mut joined = Vec::new();

    for right_row in right_rows {
        let matching_left = join_value_tuple(right_row, &key_spec.right)
            .and_then(|join_values| left_index.get(&join_values));
        if let Some(left_indices) = matching_left {
            for &left_index in left_indices {
                joined.push(join_rows_as_association(&left_rows[left_index], right_row));
            }
        } else {
            joined.push(right_row.expr.clone());
        }
    }

    joined
}

fn index_rows(rows: &[JoinRow], keys: &[Expr]) -> HashMap<Vec<Expr>, Vec<usize>> {
    let mut index = HashMap::new();
    for (row_index, row) in rows.iter().enumerate() {
        if let Some(join_values) = join_value_tuple(row, keys) {
            index
                .entry(join_values)
                .or_insert_with(Vec::new)
                .push(row_index);
        }
    }
    index
}

fn join_value_tuple(row: &JoinRow, keys: &[Expr]) -> Option<Vec<Expr>> {
    keys.iter()
        .map(|key| {
            row.entries
                .iter()
                .find(|entry| &entry.key == key)
                .map(|entry| entry.value.clone())
        })
        .collect()
}

fn join_rows_as_association(left: &JoinRow, right: &JoinRow) -> Expr {
    let mut entries = Vec::with_capacity(left.entries.len() + right.entries.len());
    entries.extend(left.entries.iter().cloned());
    entries.extend(right.entries.iter().cloned());
    render_association_entry_forms_preserving_heads(entries)
}

fn join_across_argument_count_message(count: usize) -> String {
    format!("JoinAcross called with {count} arguments; 3 or 4 arguments are expected.")
}

fn invalid_join_rows_message(position: usize, args: &[Expr]) -> String {
    let call = Expr::call(system_expr("JoinAcross"), args.to_vec());
    format!("The argument at position {position} in {call} is not a valid list of Associations.")
}

fn invalid_join_key_spec_message(expr: &Expr) -> String {
    format!(
        "The argument {expr} is not a valid key specification: a key, a rule of keys, or a non-empty list of these."
    )
}

fn invalid_join_type_message(expr: &Expr) -> String {
    format!(
        "The argument {expr} is not a valid join type; \"Inner\", \"Left\", \"Right\", or \"Outer\" is expected."
    )
}
