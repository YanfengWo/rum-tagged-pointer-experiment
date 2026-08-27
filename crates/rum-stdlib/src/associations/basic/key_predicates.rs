use super::*;
use rum_core::SharedArgs;

pub(crate) fn eval_key_member_q(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (expr, key) = match args {
        [key] => return Ok(keyed_operator_form_slot_first("KeyMemberQ", key.clone())),
        [expr, key] => (expr, key),
        _ => {
            return Ok(session.invalid_call_with_message(
                "KeyMemberQ",
                args,
                "argrx",
                exact_argument_count_message("KeyMemberQ", args.len(), 2),
            ));
        }
    };

    let result = match key_member_q_result(expr, key) {
        Ok(result) => result,
        Err(error) => {
            return Ok(session.invalid_call_with_message(
                "KeyMemberQ",
                args,
                error.tag,
                error.message,
            ));
        }
    };

    Ok(predicate_bool_expr(result))
}

pub(crate) fn eval_key_free_q(
    session: &mut Session,
    _head: &Expr,
    args: &SharedArgs,
) -> Result<Expr, EvalError> {
    let args = args.as_ref();
    let (expr, key) = match args {
        [key] => return Ok(keyed_operator_form_slot_first("KeyFreeQ", key.clone())),
        [expr, key] => (expr, key),
        _ => {
            return Ok(session.invalid_call_with_message(
                "KeyFreeQ",
                args,
                "argrx",
                exact_argument_count_message("KeyFreeQ", args.len(), 2),
            ));
        }
    };

    let result = match key_free_q_result(expr, key) {
        Ok(result) => result,
        Err(error) => {
            return Ok(session.invalid_call_with_message(
                "KeyFreeQ",
                args,
                error.tag,
                error.message,
            ));
        }
    };

    Ok(predicate_bool_expr(result))
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct KeyedPredicateInvalid {
    tag: &'static str,
    message: String,
}

fn key_member_q_result(expr: &Expr, key: &Expr) -> Result<bool, KeyedPredicateInvalid> {
    if let Some(entries) = association_entries_from_expr(expr) {
        return Ok(entries.iter().any(|(entry_key, _)| entry_key == key));
    }

    let Some(items) = list_items(expr) else {
        return Err(KeyedPredicateInvalid {
            tag: "invas",
            message: invalid_association_or_rule_message(expr),
        });
    };

    for item in items {
        let Some((entry_key, _)) = parse_association_entry(item) else {
            return Err(KeyedPredicateInvalid {
                tag: "invrk",
                message: invalid_rule_message(item),
            });
        };
        if &entry_key == key {
            return Ok(true);
        }
    }

    Ok(false)
}

fn key_free_q_result(expr: &Expr, key: &Expr) -> Result<bool, KeyedPredicateInvalid> {
    if let Some(entries) = association_entries_from_expr(expr) {
        return Ok(!entries.iter().any(|(entry_key, _)| entry_key == key));
    }

    let Some(items) = list_items(expr) else {
        return Err(KeyedPredicateInvalid {
            tag: "invas",
            message: invalid_association_or_rule_message(expr),
        });
    };

    for item in items {
        let Some((entry_key, _)) = parse_association_entry(item) else {
            return Err(KeyedPredicateInvalid {
                tag: "invrk",
                message: invalid_rule_message(item),
            });
        };
        if &entry_key == key {
            return Ok(false);
        }
    }

    Ok(true)
}
