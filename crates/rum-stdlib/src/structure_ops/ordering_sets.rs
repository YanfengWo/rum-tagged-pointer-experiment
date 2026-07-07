use std::cmp::Ordering;

use super::*;

#[derive(Clone, Debug)]
struct SortByEntry {
    item: Expr,
    key: Expr,
    index: usize,
}

#[derive(Clone)]
enum SetOperationKind {
    Union,
    Intersection { groups: Vec<Vec<Expr>> },
    Complement { excluded: Vec<Expr> },
}

#[derive(Clone)]
struct SetSameTestState {
    expr_head: Expr,
    same_test: Expr,
    scan: Vec<Expr>,
    index: usize,
    kept: Vec<Expr>,
    kind: SetOperationKind,
}

pub(crate) fn eval_sort(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    match args {
        [expr] => sort_normal_expression(session, "Sort", args, expr),
        [expr, ordering] => {
            let Expr::Call {
                head: expr_head,
                args: expr_args,
            } = expr
            else {
                return Ok(invalid_ordering_normal_call(session, "Sort", args, 1));
            };

            let items = expr_args.to_vec();
            let before = vec![vec![false; items.len()]; items.len()];
            resume_sort_comparator_from_pair(
                session,
                args.to_vec(),
                expr_head.as_ref().clone(),
                ordering.clone(),
                items,
                0,
                0,
                before,
            )
        }
        _ => Ok(session.invalid_call_with_arg_range("Sort", args, 1, 2)),
    }
}

pub(crate) fn eval_sort_by(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    match args {
        [function] => Ok(sort_by_operator_form(function)),
        [expr, function] => {
            let Expr::Call {
                head: expr_head,
                args: expr_args,
            } = expr
            else {
                return Ok(invalid_ordering_normal_call(session, "SortBy", args, 1));
            };

            resume_sort_by_from_index(
                session,
                args.to_vec(),
                expr_head.as_ref().clone(),
                function.clone(),
                expr_args.to_vec(),
                0,
                Vec::new(),
            )
        }
        [expr, function, ordering] => {
            let Expr::Call {
                head: expr_head,
                args: expr_args,
            } = expr
            else {
                return Ok(invalid_ordering_normal_call(session, "SortBy", args, 1));
            };

            resume_sort_by_ordering_from_index(
                session,
                args.to_vec(),
                expr_head.as_ref().clone(),
                function.clone(),
                ordering.clone(),
                expr_args.to_vec(),
                0,
                Vec::new(),
            )
        }
        _ => Ok(session.invalid_call_with_arg_range("SortBy", args, 1, 3)),
    }
}

pub(crate) fn eval_union(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if args.is_empty() {
        return Ok(Expr::call(system_expr("List"), Vec::new()));
    }

    let (data_args, explicit_options) = split_same_test_options(args);
    let same_test = effective_same_test(session, "Union", &explicit_options)?;
    let (expr_head, groups) = match same_head_argument_groups(session, "Union", data_args) {
        Ok(groups) => groups,
        Err(value) => return Ok(value),
    };
    let items = groups.into_iter().flatten().collect::<Vec<_>>();
    if let Some(same_test) = same_test {
        let mut scan = items;
        scan.sort_by(compare_structural_order);
        return resume_set_same_test_from_scan(
            session,
            SetSameTestState {
                expr_head,
                same_test,
                scan,
                index: 0,
                kept: Vec::new(),
                kind: SetOperationKind::Union,
            },
        );
    }

    Ok(Expr::call(expr_head, sorted_unique_exprs(items)))
}

pub(crate) fn eval_intersection(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if args.is_empty() {
        return Ok(session.invalid_call_with_message(
            "Intersection",
            args,
            "argm",
            "Intersection called with 0 arguments; 1 or more arguments are expected.",
        ));
    }

    let (data_args, explicit_options) = split_same_test_options(args);
    let same_test = effective_same_test(session, "Intersection", &explicit_options)?;
    let (expr_head, mut groups) =
        match same_head_argument_groups(session, "Intersection", data_args) {
            Ok(groups) => groups,
            Err(value) => return Ok(value),
        };
    let first = groups.remove(0);
    if let Some(same_test) = same_test {
        let mut scan = first;
        scan.sort_by(compare_structural_order);
        return resume_set_same_test_from_scan(
            session,
            SetSameTestState {
                expr_head,
                same_test,
                scan,
                index: 0,
                kept: Vec::new(),
                kind: SetOperationKind::Intersection { groups },
            },
        );
    }

    let common = first
        .into_iter()
        .filter(|item| {
            groups
                .iter()
                .all(|group| group.iter().any(|candidate| candidate == item))
        })
        .collect::<Vec<_>>();
    Ok(Expr::call(expr_head, sorted_unique_exprs(common)))
}

pub(crate) fn eval_complement(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if args.is_empty() {
        return Ok(session.invalid_call_with_message(
            "Complement",
            args,
            "argm",
            "Complement called with 0 arguments; 1 or more arguments are expected.",
        ));
    }

    let (data_args, explicit_options) = split_same_test_options(args);
    let same_test = effective_same_test(session, "Complement", &explicit_options)?;
    let (expr_head, mut groups) = match same_head_argument_groups(session, "Complement", data_args)
    {
        Ok(groups) => groups,
        Err(value) => return Ok(value),
    };
    let first = groups.remove(0);
    let excluded = groups.into_iter().flatten().collect::<Vec<_>>();
    if let Some(same_test) = same_test {
        let mut scan = first;
        scan.sort_by(compare_structural_order);
        return resume_set_same_test_from_scan(
            session,
            SetSameTestState {
                expr_head,
                same_test,
                scan,
                index: 0,
                kept: Vec::new(),
                kind: SetOperationKind::Complement { excluded },
            },
        );
    }

    let kept = first
        .into_iter()
        .filter(|item| !excluded.iter().any(|excluded| excluded == item))
        .collect::<Vec<_>>();
    Ok(Expr::call(expr_head, sorted_unique_exprs(kept)))
}

fn sort_normal_expression(
    session: &mut Session,
    symbol: &str,
    args: &[Expr],
    expr: &Expr,
) -> Result<Expr, EvalError> {
    let Expr::Call {
        head: expr_head,
        args: expr_args,
    } = expr
    else {
        return Ok(invalid_ordering_normal_call(session, symbol, args, 1));
    };

    let mut sorted = expr_args.to_vec();
    sorted.sort_by(compare_structural_order);
    Ok(Expr::call(expr_head.as_ref().clone(), sorted))
}

fn same_head_argument_groups(
    session: &mut Session,
    symbol: &str,
    args: &[Expr],
) -> Result<(Expr, Vec<Vec<Expr>>), Expr> {
    let Some((first_head, first_args)) = call_head_and_args(&args[0]) else {
        return Err(invalid_ordering_normal_call(session, symbol, args, 1));
    };

    let mut groups = vec![first_args.to_vec()];
    for (index, arg) in args.iter().enumerate().skip(1) {
        let Some((head, items)) = call_head_and_args(arg) else {
            return Err(invalid_ordering_normal_call(
                session,
                symbol,
                args,
                index + 1,
            ));
        };
        if head != first_head {
            return Err(invalid_ordering_heads_call(
                session,
                symbol,
                args,
                index + 1,
                head,
                first_head,
            ));
        }
        groups.push(items.to_vec());
    }

    Ok((first_head.clone(), groups))
}

fn split_same_test_options(args: &[Expr]) -> (&[Expr], Vec<Expr>) {
    let mut option_start = args.len();
    let mut suffix_groups = Vec::<Vec<Expr>>::new();

    while option_start > 1 {
        let Some(entries) = same_test_option_entries(&args[option_start - 1]) else {
            break;
        };
        suffix_groups.push(entries);
        option_start -= 1;
    }

    suffix_groups.reverse();
    let explicit_options = suffix_groups.into_iter().flatten().collect();
    (&args[..option_start], explicit_options)
}

fn same_test_option_entries(expr: &Expr) -> Option<Vec<Expr>> {
    if same_test_option_rule_value(expr).is_some() {
        return Some(vec![expr.clone()]);
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !is_system_symbol_name(head, heads::structure::LIST) || args.is_empty() {
        return None;
    }

    let mut entries = Vec::new();
    for arg in args {
        same_test_option_rule_value(arg)?;
        entries.push(arg.clone());
    }
    Some(entries)
}

fn same_test_option_rule_value(expr: &Expr) -> Option<&Expr> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return None;
    }
    same_test_option_name(&args[0]).filter(|name| *name == "SameTest")?;
    Some(&args[1])
}

fn same_test_option_name(expr: &Expr) -> Option<&str> {
    if let Some(name) = expr_as_string(expr) {
        return Some(name);
    }
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => Some(symbol.name()),
        _ => None,
    }
}

fn effective_same_test(
    session: &mut Session,
    symbol: &str,
    explicit_options: &[Expr],
) -> Result<Option<Expr>, EvalError> {
    let option_exprs = session.merged_option_exprs_for_system_symbol(symbol, explicit_options)?;
    let mut same_test = None;

    for expr in option_exprs {
        let Some(value) = same_test_option_rule_value(&expr) else {
            return Ok(None);
        };
        same_test = Some(value.clone());
    }

    Ok(same_test.filter(|value| !is_system_symbol_name(value, "Automatic")))
}

fn resume_set_same_test_from_scan(
    session: &mut Session,
    state: SetSameTestState,
) -> Result<Expr, EvalError> {
    let Some(candidate) = state.scan.get(state.index).cloned() else {
        return Ok(Expr::call(state.expr_head, state.kept));
    };

    if state.kept.is_empty() {
        return resume_set_same_test_membership(session, state, candidate);
    }

    resume_set_same_test_dedup(session, state, candidate, 0)
}

fn resume_set_same_test_dedup(
    session: &mut Session,
    state: SetSameTestState,
    candidate: Expr,
    kept_index: usize,
) -> Result<Expr, EvalError> {
    let Some(existing) = state.kept.get(kept_index).cloned() else {
        return resume_set_same_test_membership(session, state, candidate);
    };

    let result = eval_binary_predicate(session, &state.same_test, &candidate, &existing);
    let suspended_state = state.clone();
    let suspended_candidate = candidate.clone();
    let value = session.suspend_with_frame(result, move |session, value| {
        resume_set_same_test_dedup_with_value(
            session,
            suspended_state,
            suspended_candidate,
            kept_index,
            value,
        )
    })?;

    resume_set_same_test_dedup_with_value(session, state, candidate, kept_index, value)
}

fn resume_set_same_test_dedup_with_value(
    session: &mut Session,
    mut state: SetSameTestState,
    candidate: Expr,
    kept_index: usize,
    value: Expr,
) -> Result<Expr, EvalError> {
    if is_true_expr(&value) {
        state.index += 1;
        resume_set_same_test_from_scan(session, state)
    } else {
        resume_set_same_test_dedup(session, state, candidate, kept_index + 1)
    }
}

fn resume_set_same_test_membership(
    session: &mut Session,
    mut state: SetSameTestState,
    candidate: Expr,
) -> Result<Expr, EvalError> {
    match &state.kind {
        SetOperationKind::Union => {
            state.kept.push(candidate);
            state.index += 1;
            resume_set_same_test_from_scan(session, state)
        }
        SetOperationKind::Intersection { .. } => {
            resume_set_same_test_intersection_group(session, state, candidate, 0, 0)
        }
        SetOperationKind::Complement { .. } => {
            resume_set_same_test_complement_excluded(session, state, candidate, 0)
        }
    }
}

fn resume_set_same_test_intersection_group(
    session: &mut Session,
    mut state: SetSameTestState,
    candidate: Expr,
    group_index: usize,
    item_index: usize,
) -> Result<Expr, EvalError> {
    let SetOperationKind::Intersection { groups } = &state.kind else {
        unreachable!("intersection membership is used only for Intersection")
    };
    let Some(group) = groups.get(group_index) else {
        state.kept.push(candidate);
        state.index += 1;
        return resume_set_same_test_from_scan(session, state);
    };
    let Some(item) = group.get(item_index).cloned() else {
        state.index += 1;
        return resume_set_same_test_from_scan(session, state);
    };

    let result = eval_binary_predicate(session, &state.same_test, &candidate, &item);
    let suspended_state = state.clone();
    let suspended_candidate = candidate.clone();
    let value = session.suspend_with_frame(result, move |session, value| {
        resume_set_same_test_intersection_with_value(
            session,
            suspended_state,
            suspended_candidate,
            group_index,
            item_index,
            value,
        )
    })?;

    resume_set_same_test_intersection_with_value(
        session,
        state,
        candidate,
        group_index,
        item_index,
        value,
    )
}

fn resume_set_same_test_intersection_with_value(
    session: &mut Session,
    state: SetSameTestState,
    candidate: Expr,
    group_index: usize,
    item_index: usize,
    value: Expr,
) -> Result<Expr, EvalError> {
    if is_true_expr(&value) {
        resume_set_same_test_intersection_group(session, state, candidate, group_index + 1, 0)
    } else {
        resume_set_same_test_intersection_group(
            session,
            state,
            candidate,
            group_index,
            item_index + 1,
        )
    }
}

fn resume_set_same_test_complement_excluded(
    session: &mut Session,
    mut state: SetSameTestState,
    candidate: Expr,
    excluded_index: usize,
) -> Result<Expr, EvalError> {
    let SetOperationKind::Complement { excluded } = &state.kind else {
        unreachable!("complement membership is used only for Complement")
    };
    let Some(item) = excluded.get(excluded_index).cloned() else {
        state.kept.push(candidate);
        state.index += 1;
        return resume_set_same_test_from_scan(session, state);
    };

    let result = eval_binary_predicate(session, &state.same_test, &candidate, &item);
    let suspended_state = state.clone();
    let suspended_candidate = candidate.clone();
    let value = session.suspend_with_frame(result, move |session, value| {
        resume_set_same_test_complement_with_value(
            session,
            suspended_state,
            suspended_candidate,
            excluded_index,
            value,
        )
    })?;

    resume_set_same_test_complement_with_value(session, state, candidate, excluded_index, value)
}

fn resume_set_same_test_complement_with_value(
    session: &mut Session,
    mut state: SetSameTestState,
    candidate: Expr,
    excluded_index: usize,
    value: Expr,
) -> Result<Expr, EvalError> {
    if is_true_expr(&value) {
        state.index += 1;
        resume_set_same_test_from_scan(session, state)
    } else {
        resume_set_same_test_complement_excluded(session, state, candidate, excluded_index + 1)
    }
}

fn resume_sort_by_from_index(
    session: &mut Session,
    call_args: Vec<Expr>,
    expr_head: Expr,
    function: Expr,
    items: Vec<Expr>,
    index: usize,
    decorated: Vec<SortByEntry>,
) -> Result<Expr, EvalError> {
    let Some(item) = items.get(index).cloned() else {
        let mut sorted = decorated;
        sorted.sort_by(|lhs, rhs| {
            compare_structural_order(&lhs.key, &rhs.key).then(lhs.index.cmp(&rhs.index))
        });
        return Ok(Expr::call(
            expr_head,
            sorted.into_iter().map(|entry| entry.item).collect(),
        ));
    };

    let key_result = session.evaluate(Expr::call(function.clone(), vec![item.clone()]));
    let suspended_call_args = call_args.clone();
    let suspended_expr_head = expr_head.clone();
    let suspended_function = function.clone();
    let suspended_items = items.clone();
    let suspended_decorated = decorated.clone();
    let suspended_item = item.clone();
    let key = session.suspend_with_frame(key_result, move |session, key| {
        resume_sort_by_with_key(
            session,
            suspended_call_args,
            suspended_expr_head,
            suspended_function,
            suspended_items,
            index,
            suspended_decorated,
            suspended_item,
            key,
        )
    })?;

    resume_sort_by_with_key(
        session, call_args, expr_head, function, items, index, decorated, item, key,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_sort_by_with_key(
    session: &mut Session,
    call_args: Vec<Expr>,
    expr_head: Expr,
    function: Expr,
    items: Vec<Expr>,
    index: usize,
    mut decorated: Vec<SortByEntry>,
    item: Expr,
    key: Expr,
) -> Result<Expr, EvalError> {
    decorated.push(SortByEntry { item, key, index });
    resume_sort_by_from_index(
        session,
        call_args,
        expr_head,
        function,
        items,
        index + 1,
        decorated,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_sort_by_ordering_from_index(
    session: &mut Session,
    call_args: Vec<Expr>,
    expr_head: Expr,
    function: Expr,
    ordering: Expr,
    items: Vec<Expr>,
    index: usize,
    decorated: Vec<SortByEntry>,
) -> Result<Expr, EvalError> {
    let Some(item) = items.get(index).cloned() else {
        let before = vec![vec![false; decorated.len()]; decorated.len()];
        return resume_sort_by_key_comparator_from_pair(
            session, expr_head, ordering, decorated, 0, 0, before,
        );
    };

    let key_result = session.evaluate(Expr::call(function.clone(), vec![item.clone()]));
    let suspended_call_args = call_args.clone();
    let suspended_expr_head = expr_head.clone();
    let suspended_function = function.clone();
    let suspended_ordering = ordering.clone();
    let suspended_items = items.clone();
    let suspended_decorated = decorated.clone();
    let suspended_item = item.clone();
    let key = session.suspend_with_frame(key_result, move |session, key| {
        resume_sort_by_ordering_with_key(
            session,
            suspended_call_args,
            suspended_expr_head,
            suspended_function,
            suspended_ordering,
            suspended_items,
            index,
            suspended_decorated,
            suspended_item,
            key,
        )
    })?;

    resume_sort_by_ordering_with_key(
        session, call_args, expr_head, function, ordering, items, index, decorated, item, key,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_sort_by_ordering_with_key(
    session: &mut Session,
    call_args: Vec<Expr>,
    expr_head: Expr,
    function: Expr,
    ordering: Expr,
    items: Vec<Expr>,
    index: usize,
    mut decorated: Vec<SortByEntry>,
    item: Expr,
    key: Expr,
) -> Result<Expr, EvalError> {
    decorated.push(SortByEntry { item, key, index });
    resume_sort_by_ordering_from_index(
        session,
        call_args,
        expr_head,
        function,
        ordering,
        items,
        index + 1,
        decorated,
    )
}

fn resume_sort_by_key_comparator_from_pair(
    session: &mut Session,
    expr_head: Expr,
    ordering: Expr,
    entries: Vec<SortByEntry>,
    mut lhs_index: usize,
    mut rhs_index: usize,
    before: Vec<Vec<bool>>,
) -> Result<Expr, EvalError> {
    loop {
        if lhs_index >= entries.len() {
            return Ok(sort_by_key_comparator_table(expr_head, entries, &before));
        }
        if rhs_index >= entries.len() {
            lhs_index += 1;
            rhs_index = 0;
            continue;
        }
        if lhs_index != rhs_index {
            break;
        }
        rhs_index += 1;
    }

    let comparison_result = eval_binary_predicate(
        session,
        &ordering,
        &entries[lhs_index].key,
        &entries[rhs_index].key,
    );
    let suspended_expr_head = expr_head.clone();
    let suspended_ordering = ordering.clone();
    let suspended_entries = entries.clone();
    let suspended_before = before.clone();
    let value = session.suspend_with_frame(comparison_result, move |session, value| {
        resume_sort_by_key_comparator_with_value(
            session,
            suspended_expr_head,
            suspended_ordering,
            suspended_entries,
            lhs_index,
            rhs_index,
            suspended_before,
            value,
        )
    })?;

    resume_sort_by_key_comparator_with_value(
        session, expr_head, ordering, entries, lhs_index, rhs_index, before, value,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_sort_by_key_comparator_with_value(
    session: &mut Session,
    expr_head: Expr,
    ordering: Expr,
    entries: Vec<SortByEntry>,
    lhs_index: usize,
    rhs_index: usize,
    mut before: Vec<Vec<bool>>,
    value: Expr,
) -> Result<Expr, EvalError> {
    if is_true_expr(&value) {
        before[lhs_index][rhs_index] = true;
    }

    resume_sort_by_key_comparator_from_pair(
        session,
        expr_head,
        ordering,
        entries,
        lhs_index,
        rhs_index + 1,
        before,
    )
}

fn resume_sort_comparator_from_pair(
    session: &mut Session,
    call_args: Vec<Expr>,
    expr_head: Expr,
    ordering: Expr,
    items: Vec<Expr>,
    mut lhs_index: usize,
    mut rhs_index: usize,
    before: Vec<Vec<bool>>,
) -> Result<Expr, EvalError> {
    loop {
        if lhs_index >= items.len() {
            return Ok(sort_by_comparator_table(expr_head, items, &before));
        }
        if rhs_index >= items.len() {
            lhs_index += 1;
            rhs_index = 0;
            continue;
        }
        if lhs_index != rhs_index {
            break;
        }
        rhs_index += 1;
    }

    let comparison_result = session.evaluate(Expr::call(
        ordering.clone(),
        vec![items[lhs_index].clone(), items[rhs_index].clone()],
    ));
    let suspended_call_args = call_args.clone();
    let suspended_expr_head = expr_head.clone();
    let suspended_ordering = ordering.clone();
    let suspended_items = items.clone();
    let suspended_before = before.clone();
    let value = session.suspend_with_frame(comparison_result, move |session, value| {
        resume_sort_comparator_with_value(
            session,
            suspended_call_args,
            suspended_expr_head,
            suspended_ordering,
            suspended_items,
            lhs_index,
            rhs_index,
            suspended_before,
            value,
        )
    })?;

    resume_sort_comparator_with_value(
        session, call_args, expr_head, ordering, items, lhs_index, rhs_index, before, value,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_sort_comparator_with_value(
    session: &mut Session,
    call_args: Vec<Expr>,
    expr_head: Expr,
    ordering: Expr,
    items: Vec<Expr>,
    lhs_index: usize,
    rhs_index: usize,
    mut before: Vec<Vec<bool>>,
    value: Expr,
) -> Result<Expr, EvalError> {
    if is_system_symbol_name(&value, "True") {
        before[lhs_index][rhs_index] = true;
    }

    resume_sort_comparator_from_pair(
        session,
        call_args,
        expr_head,
        ordering,
        items,
        lhs_index,
        rhs_index + 1,
        before,
    )
}

fn invalid_ordering_normal_call(
    session: &mut Session,
    symbol: &str,
    args: &[Expr],
    position: usize,
) -> Expr {
    session.invalid_call_with_message(
        symbol,
        args,
        "normal",
        format!(
            "Nonatomic expression expected at position {position} in {}.",
            Expr::call(system_expr(symbol), args.to_vec())
        ),
    )
}

fn invalid_ordering_heads_call(
    session: &mut Session,
    symbol: &str,
    args: &[Expr],
    position: usize,
    actual_head: &Expr,
    expected_head: &Expr,
) -> Expr {
    session.invalid_call_with_message(
            symbol,
            args,
            "heads",
            format!(
                "Heads {actual_head} and {expected_head} at positions {position} and 1 are expected to be the same."
            ),
        )
}

fn sort_by_operator_form(function: &Expr) -> Expr {
    let arg = Expr::symbol("rum$sortArg");
    Expr::call(
        system_expr("Function"),
        vec![
            arg.clone(),
            Expr::call(system_expr("SortBy"), vec![arg, function.clone()]),
        ],
    )
}

fn call_head_and_args(expr: &Expr) -> Option<(&Expr, &[Expr])> {
    match expr {
        Expr::Call { head, args } => Some((head.as_ref(), args)),
        Expr::Atom(_) => None,
    }
}

fn sorted_unique_exprs(mut items: Vec<Expr>) -> Vec<Expr> {
    items.sort_by(compare_structural_order);
    let mut unique = Vec::new();
    for item in items {
        if unique.iter().any(|existing| existing == &item) {
            continue;
        }
        unique.push(item);
    }
    unique
}

fn sort_by_comparator_table(expr_head: Expr, items: Vec<Expr>, before: &[Vec<bool>]) -> Expr {
    let mut indices = (0..items.len()).collect::<Vec<_>>();
    indices.sort_by(|lhs, rhs| match (before[*lhs][*rhs], before[*rhs][*lhs]) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => lhs.cmp(rhs),
    });

    Expr::call(
        expr_head,
        indices
            .into_iter()
            .map(|index| items[index].clone())
            .collect(),
    )
}

fn sort_by_key_comparator_table(
    expr_head: Expr,
    entries: Vec<SortByEntry>,
    before: &[Vec<bool>],
) -> Expr {
    let mut indices = (0..entries.len()).collect::<Vec<_>>();
    indices.sort_by(|lhs, rhs| match (before[*lhs][*rhs], before[*rhs][*lhs]) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => lhs.cmp(rhs),
    });

    Expr::call(
        expr_head,
        indices
            .into_iter()
            .map(|index| entries[index].item.clone())
            .collect(),
    )
}

pub(super) fn compare_structural_order(lhs: &Expr, rhs: &Expr) -> Ordering {
    match (lhs, rhs) {
        (Expr::Atom(lhs), Expr::Atom(rhs)) => compare_atom_order(lhs, rhs),
        (Expr::Atom(_), Expr::Call { .. }) => Ordering::Less,
        (Expr::Call { .. }, Expr::Atom(_)) => Ordering::Greater,
        (
            Expr::Call {
                head: lhs_head,
                args: lhs_args,
            },
            Expr::Call {
                head: rhs_head,
                args: rhs_args,
            },
        ) => compare_structural_order(lhs_head, rhs_head)
            .then_with(|| compare_expr_slices(lhs_args, rhs_args)),
    }
}

fn compare_atom_order(lhs: &Atom, rhs: &Atom) -> Ordering {
    atom_order_rank(lhs)
        .cmp(&atom_order_rank(rhs))
        .then_with(|| match (lhs, rhs) {
            (Atom::Integer(lhs), Atom::Integer(rhs)) => lhs.cmp(rhs),
            (Atom::String(lhs), Atom::String(rhs)) => lhs.cmp(rhs),
            (Atom::Symbol(lhs), Atom::Symbol(rhs)) => lhs.to_string().cmp(&rhs.to_string()),
            (Atom::Rational(lhs), Atom::Rational(rhs)) => lhs.cmp(rhs),
            _ => lhs.to_string().cmp(&rhs.to_string()),
        })
}

fn atom_order_rank(atom: &Atom) -> u8 {
    match atom {
        Atom::Integer(_) | Atom::Rational(_) => 0,
        Atom::String(_) => 1,
        Atom::Symbol(_) => 2,
    }
}

fn compare_expr_slices(lhs: &[Expr], rhs: &[Expr]) -> Ordering {
    for (lhs, rhs) in lhs.iter().zip(rhs) {
        let ordering = compare_structural_order(lhs, rhs);
        if ordering != Ordering::Equal {
            return ordering;
        }
    }
    lhs.len().cmp(&rhs.len())
}
