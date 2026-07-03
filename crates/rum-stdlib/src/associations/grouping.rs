use super::*;

mod association;
mod list;

use self::{association::*, list::*};

#[derive(Clone, Debug, PartialEq, Eq)]
struct GroupBySpec {
    key_function: Expr,
    value_function: Option<Expr>,
}

fn group_by_spec(function: &Expr) -> GroupBySpec {
    match function {
        Expr::Call { head, args }
            if (is_system_symbol_name(head, "Rule")
                || is_system_symbol_name(head, "RuleDelayed"))
                && args.len() == 2 =>
        {
            GroupBySpec {
                key_function: args[0].clone(),
                value_function: Some(args[1].clone()),
            }
        }
        _ => GroupBySpec {
            key_function: function.clone(),
            value_function: None,
        },
    }
}

fn group_by_specs(function: &Expr) -> Vec<GroupBySpec> {
    if let Some(items) = list_items(function) {
        return items.iter().map(group_by_spec).collect();
    }

    vec![group_by_spec(function)]
}

pub(crate) fn eval_group_by(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    match args {
        [function] => Ok(keyed_operator_form_slot_first("GroupBy", function.clone())),
        [expr, function] => {
            let specs = group_by_specs(function);
            if let Some(entries) = association_entry_forms_from_expr(expr) {
                if specs.is_empty() {
                    return finish_association_group_by_leaf(
                        session,
                        head.clone(),
                        args.to_vec(),
                        None,
                        entries,
                    );
                }

                return resume_association_group_by_from_index(
                    session,
                    head.clone(),
                    args.to_vec(),
                    specs,
                    0,
                    None,
                    entries,
                    0,
                    Vec::new(),
                );
            }

            let Some(items) = list_items(expr) else {
                return Ok(session.invalid_call_with_message(
                    "GroupBy",
                    args,
                    "list1",
                    invalid_keyed_collection_list_message(expr),
                ));
            };

            if specs.is_empty() {
                return finish_group_by_leaf(
                    session,
                    head.clone(),
                    args.to_vec(),
                    None,
                    items.to_vec(),
                );
            }

            resume_group_by_from_index(
                session,
                head.clone(),
                args.to_vec(),
                specs,
                0,
                None,
                items.to_vec(),
                0,
                Vec::new(),
            )
        }
        [expr, function, reducer] => {
            let specs = group_by_specs(function);
            if let Some(entries) = association_entry_forms_from_expr(expr) {
                if specs.is_empty() {
                    return finish_association_group_by_leaf(
                        session,
                        head.clone(),
                        args.to_vec(),
                        Some(reducer.clone()),
                        entries,
                    );
                }

                return resume_association_group_by_from_index(
                    session,
                    head.clone(),
                    args.to_vec(),
                    specs,
                    0,
                    Some(reducer.clone()),
                    entries,
                    0,
                    Vec::new(),
                );
            }

            let Some(items) = list_items(expr) else {
                return Ok(session.invalid_call_with_message(
                    "GroupBy",
                    args,
                    "list1",
                    invalid_keyed_collection_list_message(expr),
                ));
            };

            if specs.is_empty() {
                return finish_group_by_leaf(
                    session,
                    head.clone(),
                    args.to_vec(),
                    Some(reducer.clone()),
                    items.to_vec(),
                );
            }

            resume_group_by_from_index(
                session,
                head.clone(),
                args.to_vec(),
                specs,
                0,
                Some(reducer.clone()),
                items.to_vec(),
                0,
                Vec::new(),
            )
        }
        _ => Ok(session.invalid_call_with_message(
            "GroupBy",
            args,
            "argb",
            between_one_and_three_argument_count_message("GroupBy", args.len()),
        )),
    }
}

fn between_one_and_three_argument_count_message(symbol: &str, count: usize) -> String {
    format!("{symbol} called with {count} arguments; between 1 and 3 arguments are expected.")
}
