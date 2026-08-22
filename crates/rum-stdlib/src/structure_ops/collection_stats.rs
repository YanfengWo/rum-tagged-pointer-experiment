use std::cmp::Ordering;

use super::ordering_sets::compare_structural_order;
use super::stable_groups::{StableCounts, StableGroups};
use super::*;

#[derive(Clone, Debug)]
struct KeyedEntry {
    item: Expr,
    key: Expr,
    index: usize,
}

#[derive(Clone, Debug)]
struct GatherPathEntry {
    item: Expr,
    keys: Vec<Expr>,
}

#[derive(Clone)]
struct DecorateByState {
    expr_head: Expr,
    function: Expr,
    items: Vec<Expr>,
    operation: DecoratedOperation,
}

#[derive(Clone)]
struct TallyTestState {
    test: Expr,
    items: Vec<Expr>,
    index: usize,
    counts: Vec<(Expr, usize)>,
}

#[derive(Clone)]
struct GatherByFunctionListState {
    items: Vec<Expr>,
    functions: Vec<Expr>,
    item_index: usize,
    function_index: usize,
    keys: Vec<Vec<Expr>>,
}

#[derive(Clone, Copy)]
enum DecoratedOperation {
    MaxBy { select_max: bool },
    ExtremalElementsBy { select_max: bool },
    ExtremeByCount { largest: bool, count: usize },
    TakeBy { largest: bool, count: usize },
    ReverseSortBy,
    GatherBy,
}

pub(crate) fn eval_tally(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let (expr, test) = match args {
        [expr] => (expr, None),
        [expr, test] => (expr, Some(test)),
        _ => return Ok(session.invalid_call_with_exact_arg_count("Tally", args, 1)),
    };

    let Some((_expr_head, items)) = normal_call_parts(expr) else {
        return Ok(invalid_collection_normal_call(session, "Tally", args));
    };

    if let Some(test) = test {
        return resume_tally_test_from_index(
            session,
            TallyTestState {
                test: test.clone(),
                items,
                index: 0,
                counts: Vec::new(),
            },
        );
    }

    let counts = StableCounts::collect(items).into_entries();

    Ok(list_expr(
        counts
            .into_iter()
            .map(|(item, count)| list_expr(vec![item, Expr::integer(BigInt::from(count))]))
            .collect(),
    ))
}

fn resume_tally_test_from_index(
    session: &mut Session,
    mut state: TallyTestState,
) -> Result<Expr, EvalError> {
    loop {
        let Some(item) = state.items.get(state.index).cloned() else {
            return Ok(render_tally_counts(state.counts));
        };

        if state.counts.is_empty() {
            state.counts.push((item, 1));
            state.index += 1;
            continue;
        }

        return resume_tally_test_from_count(session, state, item, 0);
    }
}

fn resume_tally_test_from_count(
    session: &mut Session,
    mut state: TallyTestState,
    item: Expr,
    count_index: usize,
) -> Result<Expr, EvalError> {
    let Some((representative, _)) = state.counts.get(count_index).cloned() else {
        state.counts.push((item, 1));
        state.index += 1;
        return resume_tally_test_from_index(session, state);
    };

    let result = eval_binary_predicate(session, &state.test, &item, &representative);
    let suspended_state = state.clone();
    let suspended_item = item.clone();
    let value = session.suspend_with_frame(result, move |session, value| {
        resume_tally_test_with_value(session, suspended_state, suspended_item, count_index, value)
    })?;

    resume_tally_test_with_value(session, state, item, count_index, value)
}

fn resume_tally_test_with_value(
    session: &mut Session,
    mut state: TallyTestState,
    item: Expr,
    count_index: usize,
    value: Expr,
) -> Result<Expr, EvalError> {
    if is_true_expr(&value) {
        state.counts[count_index].1 += 1;
        state.index += 1;
        resume_tally_test_from_index(session, state)
    } else {
        resume_tally_test_from_count(session, state, item, count_index + 1)
    }
}

fn render_tally_counts(counts: Vec<(Expr, usize)>) -> Expr {
    list_expr(
        counts
            .into_iter()
            .map(|(item, count)| list_expr(vec![item, Expr::integer(BigInt::from(count))]))
            .collect(),
    )
}

pub(crate) fn eval_max_by(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_extreme_by(session, head, "MaxBy", args, true)
}

pub(crate) fn eval_min_by(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_extreme_by(session, head, "MinBy", args, false)
}

pub(crate) fn eval_maximal_by(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_extremal_elements_by(session, "MaximalBy", args, true)
}

pub(crate) fn eval_minimal_by(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_extremal_elements_by(session, "MinimalBy", args, false)
}

pub(crate) fn eval_take_largest_by(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_take_by(session, head, args, "TakeLargestBy", true)
}

pub(crate) fn eval_take_smallest_by(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_take_by(session, head, args, "TakeSmallestBy", false)
}

pub(crate) fn eval_reverse_sort(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let (expr, ordering) = match args {
        [expr] => (expr, None),
        [expr, ordering] => (expr, Some(ordering)),
        _ => return Ok(session.invalid_call_with_exact_arg_count("ReverseSort", args, 1)),
    };

    let Some((expr_head, mut items)) = normal_call_parts(expr) else {
        return Ok(invalid_collection_normal_call(session, "ReverseSort", args));
    };

    if let Some(ordering) = ordering {
        let before = vec![vec![false; items.len()]; items.len()];
        return resume_reverse_sort_comparator_from_pair(
            session,
            expr_head,
            ordering.clone(),
            items,
            0,
            0,
            before,
        );
    }

    items.sort_by(compare_structural_order);
    items.reverse();
    Ok(Expr::call(expr_head, items))
}

pub(crate) fn eval_reverse_sort_by(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [expr, function] = args else {
        if args.len() == 1 {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }
        return Ok(session.invalid_call_with_exact_arg_count("ReverseSortBy", args, 2));
    };

    let Some((expr_head, items)) = normal_call_parts(expr) else {
        return Ok(invalid_collection_normal_call(
            session,
            "ReverseSortBy",
            args,
        ));
    };

    resume_decorate_by_from_index(
        session,
        DecorateByState {
            expr_head,
            function: function.clone(),
            items,
            operation: DecoratedOperation::ReverseSortBy,
        },
        0,
        Vec::new(),
    )
}

pub(crate) fn eval_gather_by(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [expr, function] = args else {
        if args.len() == 1 {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }
        return Ok(session.invalid_call_with_exact_arg_count("GatherBy", args, 2));
    };

    let Some((_expr_head, items)) = normal_call_parts(expr) else {
        return Ok(invalid_collection_normal_call(session, "GatherBy", args));
    };

    if let Some(functions) = list_items(function) {
        return resume_gather_by_function_list_from_position(
            session,
            GatherByFunctionListState {
                items,
                functions: functions.to_vec(),
                item_index: 0,
                function_index: 0,
                keys: Vec::new(),
            },
        );
    }

    resume_decorate_by_from_index(
        session,
        DecorateByState {
            expr_head: system_expr("List"),
            function: function.clone(),
            items,
            operation: DecoratedOperation::GatherBy,
        },
        0,
        Vec::new(),
    )
}

pub(crate) fn eval_riffle(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let (expr, separator, positions) = match args {
        [expr, separator] => (expr, separator, RifflePositions::Every(1)),
        [expr, separator, positions] => {
            let Some(positions) = parse_riffle_positions(positions) else {
                return Ok(invalid_riffle_position_call(session, args));
            };
            (expr, separator, positions)
        }
        _ => {
            if args.len() == 1 {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            }
            return Ok(session.invalid_call_with_arg_range("Riffle", args, 2, 3));
        }
    };

    let Some((expr_head, items)) = normal_call_parts(expr) else {
        return Ok(invalid_collection_normal_call(session, "Riffle", args));
    };

    if items.len() <= 1 {
        return Ok(Expr::call(expr_head, items));
    }

    let insertion_positions = riffle_insertion_positions(&positions, items.len());
    let separator_count = insertion_positions.iter().filter(|insert| **insert).count();
    let output_len = items.len().saturating_add(separator_count);
    session.enforce_generated_element_limit_for_count(output_len)?;

    let mut riffled = Vec::with_capacity(output_len);
    for (index, item) in items.into_iter().enumerate() {
        riffled.push(item);
        if insertion_positions[index + 1] {
            riffled.push(separator.clone());
        }
    }

    Ok(Expr::call(expr_head, riffled))
}

fn resume_reverse_sort_comparator_from_pair(
    session: &mut Session,
    expr_head: Expr,
    ordering: Expr,
    items: Vec<Expr>,
    mut lhs_index: usize,
    mut rhs_index: usize,
    mut before: Vec<Vec<bool>>,
) -> Result<Expr, EvalError> {
    loop {
        if lhs_index >= items.len() {
            return Ok(reverse_sort_by_comparator_table(expr_head, items, &before));
        }
        if rhs_index >= items.len() {
            lhs_index += 1;
            rhs_index = 0;
            continue;
        }
        if lhs_index != rhs_index {
            let comparison_result =
                eval_binary_predicate(session, &ordering, &items[lhs_index], &items[rhs_index]);
            let value = match comparison_result {
                Ok(value) => value,
                Err(EvalError::InputRequired { request }) => {
                    return session.suspend_with_frame(
                        Err(EvalError::InputRequired { request }),
                        move |session, value| {
                            resume_reverse_sort_comparator_with_value(
                                session, expr_head, ordering, items, lhs_index, rhs_index, before,
                                value,
                            )
                        },
                    );
                }
                Err(error) => return Err(error),
            };

            if is_true_expr(&value) {
                before[lhs_index][rhs_index] = true;
            }
        }
        rhs_index += 1;
    }
}

#[allow(clippy::too_many_arguments)]
fn resume_reverse_sort_comparator_with_value(
    session: &mut Session,
    expr_head: Expr,
    ordering: Expr,
    items: Vec<Expr>,
    lhs_index: usize,
    rhs_index: usize,
    mut before: Vec<Vec<bool>>,
    value: Expr,
) -> Result<Expr, EvalError> {
    if is_true_expr(&value) {
        before[lhs_index][rhs_index] = true;
    }

    resume_reverse_sort_comparator_from_pair(
        session,
        expr_head,
        ordering,
        items,
        lhs_index,
        rhs_index + 1,
        before,
    )
}

fn reverse_sort_by_comparator_table(
    expr_head: Expr,
    items: Vec<Expr>,
    before: &[Vec<bool>],
) -> Expr {
    let mut indices = (0..items.len()).collect::<Vec<_>>();
    indices.sort_by(|lhs, rhs| match (before[*lhs][*rhs], before[*rhs][*lhs]) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => lhs.cmp(rhs),
    });
    indices.reverse();

    Expr::call(
        expr_head,
        indices
            .into_iter()
            .map(|index| items[index].clone())
            .collect(),
    )
}

fn resume_gather_by_function_list_from_position(
    session: &mut Session,
    mut state: GatherByFunctionListState,
) -> Result<Expr, EvalError> {
    if state.functions.is_empty() {
        return Ok(list_expr(state.items));
    }

    loop {
        if state.item_index >= state.items.len() {
            return Ok(render_hierarchical_gather_by(state.items, state.keys));
        }
        if state.function_index >= state.functions.len() {
            state.item_index += 1;
            state.function_index = 0;
            continue;
        }

        while state.keys.len() <= state.item_index {
            state.keys.push(Vec::with_capacity(state.functions.len()));
        }

        let item = state.items[state.item_index].clone();
        let function = state.functions[state.function_index].clone();
        let key = match session.evaluate(Expr::call(function, vec![item])) {
            Ok(key) => key,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame(
                    Err(EvalError::InputRequired { request }),
                    move |session, key| {
                        resume_gather_by_function_list_with_key(session, state, key)
                    },
                );
            }
            Err(error) => return Err(error),
        };

        state.keys[state.item_index].push(key);
        state.function_index += 1;
    }
}

fn resume_gather_by_function_list_with_key(
    session: &mut Session,
    mut state: GatherByFunctionListState,
    key: Expr,
) -> Result<Expr, EvalError> {
    state.keys[state.item_index].push(key);
    state.function_index += 1;
    resume_gather_by_function_list_from_position(session, state)
}

fn render_hierarchical_gather_by(items: Vec<Expr>, keys: Vec<Vec<Expr>>) -> Expr {
    let key_count = keys.first().map(|item_keys| item_keys.len()).unwrap_or(0);
    let entries = items
        .into_iter()
        .zip(keys)
        .map(|(item, keys)| GatherPathEntry { item, keys })
        .collect();
    list_expr(hierarchical_gather_entries(entries, 0, key_count))
}

fn hierarchical_gather_entries(
    entries: Vec<GatherPathEntry>,
    depth: usize,
    key_count: usize,
) -> Vec<Expr> {
    if depth >= key_count {
        return entries.into_iter().map(|entry| entry.item).collect();
    }

    StableGroups::collect(
        entries
            .into_iter()
            .map(|entry| (entry.keys[depth].clone(), entry)),
    )
    .into_entries()
    .into_iter()
    .map(|(_, group)| list_expr(hierarchical_gather_entries(group, depth + 1, key_count)))
    .collect()
}

fn eval_extreme_by(
    session: &mut Session,
    head: &Expr,
    symbol: &str,
    args: &[Expr],
    select_max: bool,
) -> Result<Expr, EvalError> {
    let (expr, function, count) = match args {
        [expr, function] => (expr, function, None),
        [expr, function, count] => (expr, function, Some(count)),
        _ => {
            if args.len() == 1 {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            }
            return Ok(session.invalid_call_with_exact_arg_count(symbol, args, 2));
        }
    };

    let Some((expr_head, items)) = normal_call_parts(expr) else {
        return Ok(invalid_collection_normal_call(session, symbol, args));
    };
    if count.is_none() && items.is_empty() {
        return Ok(invalid_empty_collection_call(session, symbol, args));
    }
    let count = match count {
        Some(count) => Some(match expr_as_non_negative_usize(count) {
            Some(count) => count,
            None => return Ok(Expr::call(head.clone(), args.to_vec())),
        }),
        None => None,
    };

    resume_decorate_by_from_index(
        session,
        DecorateByState {
            expr_head,
            function: function.clone(),
            items,
            operation: match count {
                Some(count) => DecoratedOperation::ExtremeByCount {
                    largest: select_max,
                    count,
                },
                None => DecoratedOperation::MaxBy { select_max },
            },
        },
        0,
        Vec::new(),
    )
}

fn eval_extremal_elements_by(
    session: &mut Session,
    symbol: &str,
    args: &[Expr],
    select_max: bool,
) -> Result<Expr, EvalError> {
    let (expr, function) = match args {
        [function] => return Ok(extremal_elements_by_operator_form(symbol, function)),
        [expr, function] => (expr, function),
        _ => return Ok(session.invalid_call_with_arg_range(symbol, args, 1, 2)),
    };

    let Some((expr_head, items)) = normal_call_parts(expr) else {
        return Ok(invalid_collection_normal_call(session, symbol, args));
    };

    resume_decorate_by_from_index(
        session,
        DecorateByState {
            expr_head,
            function: function.clone(),
            items,
            operation: DecoratedOperation::ExtremalElementsBy { select_max },
        },
        0,
        Vec::new(),
    )
}

fn eval_take_by(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    symbol: &str,
    largest: bool,
) -> Result<Expr, EvalError> {
    let [expr, function, count] = args else {
        if args.len() == 2 {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }
        return Ok(session.invalid_call_with_exact_arg_count(symbol, args, 3));
    };

    let Some((expr_head, items)) = normal_call_parts(expr) else {
        return Ok(invalid_collection_normal_call(session, symbol, args));
    };
    let Some(count) = expr_as_non_negative_usize(count) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    resume_decorate_by_from_index(
        session,
        DecorateByState {
            expr_head,
            function: function.clone(),
            items,
            operation: DecoratedOperation::TakeBy { largest, count },
        },
        0,
        Vec::new(),
    )
}

fn resume_decorate_by_from_index(
    session: &mut Session,
    state: DecorateByState,
    mut index: usize,
    mut decorated: Vec<KeyedEntry>,
) -> Result<Expr, EvalError> {
    loop {
        let Some(item) = state.items.get(index).cloned() else {
            return finish_decorated(state, decorated);
        };

        let key = match session.evaluate(Expr::call(state.function.clone(), vec![item.clone()])) {
            Ok(key) => key,
            Err(EvalError::InputRequired { request }) => {
                return session.suspend_with_frame(
                    Err(EvalError::InputRequired { request }),
                    move |session, key| {
                        resume_decorate_by_with_key(session, state, index, decorated, item, key)
                    },
                );
            }
            Err(error) => return Err(error),
        };

        decorated.push(KeyedEntry { item, key, index });
        index += 1;
    }
}

fn resume_decorate_by_with_key(
    session: &mut Session,
    state: DecorateByState,
    index: usize,
    mut decorated: Vec<KeyedEntry>,
    item: Expr,
    key: Expr,
) -> Result<Expr, EvalError> {
    decorated.push(KeyedEntry { item, key, index });
    resume_decorate_by_from_index(session, state, index + 1, decorated)
}

fn finish_decorated(state: DecorateByState, decorated: Vec<KeyedEntry>) -> Result<Expr, EvalError> {
    match state.operation {
        DecoratedOperation::MaxBy { select_max } => Ok(select_extreme_by(decorated, select_max)
            .unwrap_or_else(|| Expr::call(state.expr_head, Vec::new()))),
        DecoratedOperation::ExtremalElementsBy { select_max } => Ok(Expr::call(
            state.expr_head,
            select_extremal_elements_by(decorated, select_max),
        )),
        DecoratedOperation::ExtremeByCount { largest, count } => Ok(Expr::call(
            state.expr_head,
            take_by(decorated, largest, count),
        )),
        DecoratedOperation::TakeBy { largest, count } => Ok(Expr::call(
            state.expr_head,
            take_by(decorated, largest, count),
        )),
        DecoratedOperation::ReverseSortBy => {
            let mut sorted = decorated;
            sorted.sort_by(|lhs, rhs| {
                compare_structural_order(&lhs.key, &rhs.key).then(lhs.index.cmp(&rhs.index))
            });
            sorted.reverse();
            Ok(Expr::call(
                state.expr_head,
                sorted.into_iter().map(|entry| entry.item).collect(),
            ))
        }
        DecoratedOperation::GatherBy => Ok(list_expr(gather_by_key(decorated))),
    }
}

fn select_extreme_by(decorated: Vec<KeyedEntry>, select_max: bool) -> Option<Expr> {
    let mut entries = decorated.into_iter();
    let mut best = entries.next()?;

    for entry in entries {
        let ordering = compare_structural_order(&entry.key, &best.key);
        if (select_max && ordering == Ordering::Greater)
            || (!select_max && ordering == Ordering::Less)
        {
            best = entry;
        }
    }

    Some(best.item)
}

fn select_extremal_elements_by(decorated: Vec<KeyedEntry>, select_max: bool) -> Vec<Expr> {
    let mut entries = decorated.into_iter();
    let Some(first) = entries.next() else {
        return Vec::new();
    };
    let mut best_key = first.key;
    let mut selected = vec![first.item];

    for entry in entries {
        let ordering = compare_structural_order(&entry.key, &best_key);
        if (select_max && ordering == Ordering::Greater)
            || (!select_max && ordering == Ordering::Less)
        {
            best_key = entry.key;
            selected.clear();
            selected.push(entry.item);
        } else if ordering == Ordering::Equal {
            selected.push(entry.item);
        }
    }

    selected
}

fn extremal_elements_by_operator_form(symbol: &str, function: &Expr) -> Expr {
    let arg = Expr::symbol("rum$extremalByArg");
    Expr::call(
        system_expr("Function"),
        vec![
            arg.clone(),
            Expr::call(system_expr(symbol), vec![arg, function.clone()]),
        ],
    )
}

fn take_by(mut decorated: Vec<KeyedEntry>, largest: bool, count: usize) -> Vec<Expr> {
    decorated.sort_by(|lhs, rhs| {
        let key_order = if largest {
            compare_structural_order(&rhs.key, &lhs.key)
        } else {
            compare_structural_order(&lhs.key, &rhs.key)
        };
        key_order.then(lhs.index.cmp(&rhs.index))
    });
    decorated.truncate(count);
    decorated.into_iter().map(|entry| entry.item).collect()
}

fn gather_by_key(decorated: Vec<KeyedEntry>) -> Vec<Expr> {
    StableGroups::collect(decorated.into_iter().map(|entry| (entry.key, entry.item)))
        .into_entries()
        .into_iter()
        .map(|(_, group)| list_expr(group))
        .collect()
}

fn normal_call_parts(expr: &Expr) -> Option<(Expr, Vec<Expr>)> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    Some((head.as_ref().clone(), args.to_vec()))
}

fn invalid_collection_normal_call(session: &mut Session, symbol: &str, args: &[Expr]) -> Expr {
    session.invalid_call_with_message(
        symbol,
        args,
        "normal",
        format!(
            "Nonatomic expression expected at position 1 in {}.",
            Expr::call(system_expr(symbol), args.to_vec())
        ),
    )
}

fn invalid_empty_collection_call(session: &mut Session, symbol: &str, args: &[Expr]) -> Expr {
    session.invalid_call_with_message(
        symbol,
        args,
        "empty",
        format!("{symbol} cannot select from an empty list."),
    )
}

fn invalid_riffle_position_call(session: &mut Session, args: &[Expr]) -> Expr {
    session.invalid_call_with_message(
        "Riffle",
        args,
        "seqs",
        format!(
            "Position 3 of {} must be a positive machine-sized integer or {{imin, imax, di}} with nonzero integer step.",
            Expr::call(system_expr("Riffle"), args.to_vec())
        ),
    )
}

fn list_expr(items: Vec<Expr>) -> Expr {
    Expr::call(system_expr("List"), items)
}

#[derive(Clone, Copy)]
enum RifflePositions {
    Every(usize),
    Range {
        start: isize,
        end: isize,
        step: isize,
    },
}

fn parse_riffle_positions(expr: &Expr) -> Option<RifflePositions> {
    if let Some(interval) = expr_as_positive_usize(expr) {
        return Some(RifflePositions::Every(interval));
    }

    let [start, end, step] = list_items(expr)? else {
        return None;
    };
    let start = expr_as_signed_isize(start)?;
    let end = expr_as_signed_isize(end)?;
    let step = expr_as_signed_isize(step)?;
    (step != 0).then_some(RifflePositions::Range { start, end, step })
}

fn expr_as_positive_usize(expr: &Expr) -> Option<usize> {
    expr_as_non_negative_usize(expr).filter(|value| *value > 0)
}

fn riffle_insertion_positions(positions: &RifflePositions, item_count: usize) -> Vec<bool> {
    let mut insert = vec![false; item_count + 1];
    match *positions {
        RifflePositions::Every(interval) => {
            let mut position = interval;
            while position < item_count {
                insert[position] = true;
                position = position.saturating_add(interval);
            }
        }
        RifflePositions::Range { start, end, step } => {
            for position in 1..item_count {
                let positive = position as isize;
                let negative = positive - item_count as isize - 1;
                if signed_position_in_range(positive, start, end, step)
                    || signed_position_in_range(negative, start, end, step)
                {
                    insert[position] = true;
                }
            }
        }
    }
    insert
}

fn signed_position_in_range(position: isize, start: isize, end: isize, step: isize) -> bool {
    let position = position as i128;
    let start = start as i128;
    let end = end as i128;
    let step = step as i128;
    if step > 0 {
        position >= start && position <= end && (position - start) % step == 0
    } else {
        let step = -step;
        position <= start && position >= end && (start - position) % step == 0
    }
}
