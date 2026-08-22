use std::collections::HashSet;

use super::*;

struct OrderedQComparatorState {
    ordering: Expr,
    items: Vec<Expr>,
    index: usize,
}

struct LexicographicListComparatorState {
    ordering: Expr,
    lhs: Vec<Expr>,
    rhs: Vec<Expr>,
    index: usize,
}

struct LexicographicPairComparatorState {
    ordering: Expr,
    lhs: Expr,
    rhs: Expr,
}

pub(crate) fn eval_order(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [lhs, rhs] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("Order", args, 2));
    };

    Ok(order_result_expr(compare_exact_order_subset(lhs, rhs)))
}

pub(crate) fn eval_ordered_q(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    match args {
        [expr] => eval_ordered_q_default(session, head, args, expr),
        [expr, ordering] => eval_ordered_q_with_comparator(session, head, args, expr, ordering),
        _ => Ok(session.invalid_call_with_arg_range("OrderedQ", args, 1, 2)),
    }
}

fn eval_ordered_q_default(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    expr: &Expr,
) -> Result<Expr, EvalError> {
    let items = match expr {
        Expr::Call { args, .. } => args.as_slice(),
        Expr::Atom(_) => {
            return Ok(session.invalid_call_with_message(
                "OrderedQ",
                args,
                "normal",
                format!(
                    "Nonatomic expression expected at position 1 in {}.",
                    Expr::call(head.clone(), args.to_vec())
                ),
            ));
        }
    };

    Ok(predicate_bool_expr(items.windows(2).all(|pair| {
        compare_exact_order_subset(&pair[0], &pair[1]) != std::cmp::Ordering::Greater
    })))
}

fn eval_ordered_q_with_comparator(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    expr: &Expr,
    ordering: &Expr,
) -> Result<Expr, EvalError> {
    let items = match expr {
        Expr::Call { args, .. } => args.to_vec(),
        Expr::Atom(_) => {
            return Ok(session.invalid_call_with_message(
                "OrderedQ",
                args,
                "normal",
                format!(
                    "Nonatomic expression expected at position 1 in {}.",
                    Expr::call(head.clone(), args.to_vec())
                ),
            ));
        }
    };

    resume_ordered_q_comparator(
        session,
        OrderedQComparatorState {
            ordering: ordering.clone(),
            items,
            index: 0,
        },
    )
}

fn resume_ordered_q_comparator(
    session: &mut Session,
    mut state: OrderedQComparatorState,
) -> Result<Expr, EvalError> {
    while state.index + 1 < state.items.len() {
        let value = match eval_binary_predicate(
            session,
            &state.ordering,
            &state.items[state.index],
            &state.items[state.index + 1],
        ) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame(
                    Err(EvalError::InputRequired { request }),
                    move |session, value| {
                        resume_ordered_q_comparator_with_value(session, state, value)
                    },
                );
            }
            Err(error) => return Err(error),
        };

        if !comparator_says_before(&value) {
            return Ok(predicate_bool_expr(false));
        }
        state.index += 1;
    }

    Ok(predicate_bool_expr(true))
}

fn resume_ordered_q_comparator_with_value(
    session: &mut Session,
    mut state: OrderedQComparatorState,
    value: Expr,
) -> Result<Expr, EvalError> {
    if !comparator_says_before(&value) {
        return Ok(predicate_bool_expr(false));
    }

    state.index += 1;
    resume_ordered_q_comparator(session, state)
}

pub(crate) fn eval_lexicographic_order(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    match args {
        [lhs, rhs] => Ok(order_result_expr(compare_lexicographic_order_subset(
            lhs, rhs,
        ))),
        [lhs, rhs, ordering] => eval_lexicographic_order_with_comparator(
            session,
            ordering.clone(),
            lhs.clone(),
            rhs.clone(),
        ),
        _ => Ok(if (1..=3).contains(&args.len()) {
            Expr::call(head.clone(), args.to_vec())
        } else {
            session.invalid_call_with_arg_range("LexicographicOrder", args, 1, 3)
        }),
    }
}

fn eval_lexicographic_order_with_comparator(
    session: &mut Session,
    ordering: Expr,
    lhs: Expr,
    rhs: Expr,
) -> Result<Expr, EvalError> {
    match (list_items(&lhs), list_items(&rhs)) {
        (Some(lhs_items), Some(rhs_items)) => resume_lexicographic_list_comparator(
            session,
            LexicographicListComparatorState {
                ordering,
                lhs: lhs_items.to_vec(),
                rhs: rhs_items.to_vec(),
                index: 0,
            },
        ),
        _ => resume_lexicographic_pair_comparator(
            session,
            LexicographicPairComparatorState { ordering, lhs, rhs },
        ),
    }
}

fn resume_lexicographic_list_comparator(
    session: &mut Session,
    mut state: LexicographicListComparatorState,
) -> Result<Expr, EvalError> {
    while state.index < state.lhs.len() && state.index < state.rhs.len() {
        let forward_value = match eval_binary_predicate(
            session,
            &state.ordering,
            &state.lhs[state.index],
            &state.rhs[state.index],
        ) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame(
                    Err(EvalError::InputRequired { request }),
                    move |session, value| {
                        resume_lexicographic_list_comparator_with_forward_value(
                            session, state, value,
                        )
                    },
                );
            }
            Err(error) => return Err(error),
        };

        if comparator_says_before(&forward_value) {
            return Ok(order_result_expr(std::cmp::Ordering::Less));
        }

        let reverse_value = match eval_binary_predicate(
            session,
            &state.ordering,
            &state.rhs[state.index],
            &state.lhs[state.index],
        ) {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame(
                    Err(EvalError::InputRequired { request }),
                    move |session, value| {
                        resume_lexicographic_list_comparator_with_reverse_value(
                            session, state, value,
                        )
                    },
                );
            }
            Err(error) => return Err(error),
        };

        if comparator_says_before(&reverse_value) {
            return Ok(order_result_expr(std::cmp::Ordering::Greater));
        }
        state.index += 1;
    }

    Ok(order_result_expr(state.lhs.len().cmp(&state.rhs.len())))
}

fn resume_lexicographic_list_comparator_with_forward_value(
    session: &mut Session,
    state: LexicographicListComparatorState,
    value: Expr,
) -> Result<Expr, EvalError> {
    if comparator_says_before(&value) {
        return Ok(order_result_expr(std::cmp::Ordering::Less));
    }

    let value = match eval_binary_predicate(
        session,
        &state.ordering,
        &state.rhs[state.index],
        &state.lhs[state.index],
    ) {
        Ok(value) => value,
        Err(EvalError::InputRequired { request }) => {
            return session.suspend_with_frame(
                Err(EvalError::InputRequired { request }),
                move |session, value| {
                    resume_lexicographic_list_comparator_with_reverse_value(session, state, value)
                },
            );
        }
        Err(error) => return Err(error),
    };

    resume_lexicographic_list_comparator_with_reverse_value(session, state, value)
}

fn resume_lexicographic_list_comparator_with_reverse_value(
    session: &mut Session,
    mut state: LexicographicListComparatorState,
    value: Expr,
) -> Result<Expr, EvalError> {
    if comparator_says_before(&value) {
        return Ok(order_result_expr(std::cmp::Ordering::Greater));
    }

    state.index += 1;
    resume_lexicographic_list_comparator(session, state)
}

fn resume_lexicographic_pair_comparator(
    session: &mut Session,
    state: LexicographicPairComparatorState,
) -> Result<Expr, EvalError> {
    let value = match eval_binary_predicate(session, &state.ordering, &state.lhs, &state.rhs) {
        Ok(value) => value,
        Err(EvalError::InputRequired { request }) => {
            return session.suspend_with_frame(
                Err(EvalError::InputRequired { request }),
                move |session, value| {
                    resume_lexicographic_pair_comparator_with_forward_value(session, state, value)
                },
            );
        }
        Err(error) => return Err(error),
    };

    resume_lexicographic_pair_comparator_with_forward_value(session, state, value)
}

fn resume_lexicographic_pair_comparator_with_forward_value(
    session: &mut Session,
    state: LexicographicPairComparatorState,
    value: Expr,
) -> Result<Expr, EvalError> {
    if comparator_says_before(&value) {
        return Ok(order_result_expr(std::cmp::Ordering::Less));
    }

    let value = match eval_binary_predicate(session, &state.ordering, &state.rhs, &state.lhs) {
        Ok(value) => value,
        Err(EvalError::InputRequired { request }) => {
            return session.suspend_with_frame(
                Err(EvalError::InputRequired { request }),
                move |session, value| {
                    resume_lexicographic_pair_comparator_with_reverse_value(session, state, value)
                },
            );
        }
        Err(error) => return Err(error),
    };

    resume_lexicographic_pair_comparator_with_reverse_value(session, state, value)
}

fn resume_lexicographic_pair_comparator_with_reverse_value(
    _session: &mut Session,
    _state: LexicographicPairComparatorState,
    value: Expr,
) -> Result<Expr, EvalError> {
    if comparator_says_before(&value) {
        Ok(order_result_expr(std::cmp::Ordering::Greater))
    } else {
        Ok(order_result_expr(std::cmp::Ordering::Equal))
    }
}

pub(crate) fn eval_numerical_order(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [lhs, rhs] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("NumericalOrder", args, 2));
    };

    Ok(order_result_expr(compare_exact_order_subset(lhs, rhs)))
}

pub(crate) fn eval_alphabetic_order(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [lhs, rhs] = args else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let Some(lhs) = string_atom_value(lhs) else {
        return Ok(session.invalid_call_with_message(
            "AlphabeticOrder",
            args,
            "nstr",
            format!("argument {} is not a string", lhs),
        ));
    };
    let Some(rhs) = string_atom_value(rhs) else {
        return Ok(session.invalid_call_with_message(
            "AlphabeticOrder",
            args,
            "nstr",
            format!("argument {} is not a string", rhs),
        ));
    };

    Ok(order_result_expr(lhs.cmp(rhs)))
}

pub(crate) fn eval_between(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if args.len() == 1 {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    }

    let [value_expr, interval_expr] = args else {
        return Ok(session.invalid_call_with_exact_arg_count("Between", args, 2));
    };

    let Some(value) = exact_numeric_value(value_expr) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let Some(interval_items) = list_items(interval_expr) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let [lower_expr, upper_expr] = interval_items else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    let (Some(lower), Some(upper)) = (
        exact_numeric_value(lower_expr),
        exact_numeric_value(upper_expr),
    ) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    Ok(predicate_bool_expr(
        compare_exact_numeric(&lower, &value) != std::cmp::Ordering::Greater
            && compare_exact_numeric(&value, &upper) != std::cmp::Ordering::Greater,
    ))
}

pub(crate) fn eval_same_q(
    _session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    Ok(predicate_bool_expr(
        args.windows(2).all(|pair| pair[0] == pair[1]),
    ))
}

pub(crate) fn eval_unsame_q(
    _session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    const INDEX_THRESHOLD: usize = 16;

    if args.len() <= INDEX_THRESHOLD {
        for (index, lhs) in args.iter().enumerate() {
            if args[index + 1..].iter().any(|rhs| lhs == rhs) {
                return Ok(predicate_bool_expr(false));
            }
        }
        return Ok(predicate_bool_expr(true));
    }

    let mut seen = HashSet::with_capacity(args.len());
    Ok(predicate_bool_expr(args.iter().all(|arg| seen.insert(arg))))
}

fn eval_binary_predicate(
    session: &mut Session,
    predicate: &Expr,
    lhs: &Expr,
    rhs: &Expr,
) -> Result<Expr, EvalError> {
    session.evaluate(Expr::call(
        predicate.clone(),
        vec![lhs.clone(), rhs.clone()],
    ))
}

fn comparator_says_before(value: &Expr) -> bool {
    match value {
        Expr::Atom(Atom::Integer(integer)) if integer.as_i64() == Some(1) => true,
        _ => is_system_symbol_name(value, heads::structure::TRUE),
    }
}
