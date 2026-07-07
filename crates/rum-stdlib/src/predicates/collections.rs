use super::*;

#[derive(Clone, Copy)]
enum SameTestRelationKind {
    Intersecting,
    Disjoint,
    Subset,
    ContainsAll,
    ContainsAny,
    ContainsExactly,
    ContainsNone,
    ContainsOnly,
}

#[derive(Clone)]
struct DuplicateFreeSameTestState {
    same_test: Expr,
    items: Vec<Expr>,
    lhs_index: usize,
    rhs_index: usize,
}

#[derive(Clone)]
struct SameTestMatrixState {
    same_test: Expr,
    lhs: Vec<Expr>,
    rhs: Vec<Expr>,
    lhs_index: usize,
    rhs_index: usize,
    matches: Vec<Vec<bool>>,
    kind: SameTestRelationKind,
}

pub(crate) fn eval_duplicate_free_q(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let (data_args, explicit_options) = split_same_test_options(args, 1);
    let [expr] = data_args else {
        return Ok(session.invalid_call_with_exact_arg_count("DuplicateFreeQ", args, 1));
    };
    let same_test = effective_same_test(session, "DuplicateFreeQ", &explicit_options)?;

    let Some(items) = list_items(expr) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    if let Some(same_test) = same_test {
        return eval_duplicate_free_same_test(session, same_test, items);
    }

    let mut seen = std::collections::HashSet::new();
    for item in items {
        if !seen.insert(item.clone()) {
            return Ok(predicate_bool_expr(false));
        }
    }

    Ok(predicate_bool_expr(true))
}

pub(crate) fn eval_intersecting_q(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_list_containment_relation(
        session,
        head,
        "IntersectingQ",
        args,
        SameTestRelationKind::Intersecting,
        |lhs, rhs| {
            let rhs_set = expr_set(rhs);
            lhs.iter().any(|item| rhs_set.contains(item))
        },
    )
}

pub(crate) fn eval_disjoint_q(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_list_containment_relation(
        session,
        head,
        "DisjointQ",
        args,
        SameTestRelationKind::Disjoint,
        |lhs, rhs| {
            let rhs_set = expr_set(rhs);
            !lhs.iter().any(|item| rhs_set.contains(item))
        },
    )
}

pub(crate) fn eval_subset_q(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_list_containment_relation(
        session,
        head,
        "SubsetQ",
        args,
        SameTestRelationKind::Subset,
        |lhs, rhs| {
            let lhs_set = expr_set(lhs);
            expr_set(rhs)
                .into_iter()
                .all(|item| lhs_set.contains(&item))
        },
    )
}

pub(crate) fn eval_contains_all(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_list_containment_relation(
        session,
        head,
        "ContainsAll",
        args,
        SameTestRelationKind::ContainsAll,
        |lhs, rhs| {
            let lhs_set = expr_set(lhs);
            expr_set(rhs)
                .into_iter()
                .all(|item| lhs_set.contains(&item))
        },
    )
}

pub(crate) fn eval_contains_any(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_list_containment_relation(
        session,
        head,
        "ContainsAny",
        args,
        SameTestRelationKind::ContainsAny,
        |lhs, rhs| {
            let lhs_set = expr_set(lhs);
            expr_set(rhs)
                .into_iter()
                .any(|item| lhs_set.contains(&item))
        },
    )
}

pub(crate) fn eval_contains_exactly(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_list_containment_relation(
        session,
        head,
        "ContainsExactly",
        args,
        SameTestRelationKind::ContainsExactly,
        |lhs, rhs| expr_counts(lhs) == expr_counts(rhs),
    )
}

pub(crate) fn eval_contains_none(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_list_containment_relation(
        session,
        head,
        "ContainsNone",
        args,
        SameTestRelationKind::ContainsNone,
        |lhs, rhs| {
            let lhs_set = expr_set(lhs);
            expr_set(rhs)
                .into_iter()
                .all(|item| !lhs_set.contains(&item))
        },
    )
}

pub(crate) fn eval_contains_only(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    eval_list_containment_relation(
        session,
        head,
        "ContainsOnly",
        args,
        SameTestRelationKind::ContainsOnly,
        |lhs, rhs| {
            let rhs_set = expr_set(rhs);
            lhs.iter().all(|item| rhs_set.contains(item))
        },
    )
}

fn eval_list_containment_relation<F>(
    session: &mut Session,
    head: &Expr,
    name: &str,
    args: &[Expr],
    same_test_kind: SameTestRelationKind,
    predicate: F,
) -> Result<Expr, EvalError>
where
    F: FnOnce(&[Expr], &[Expr]) -> bool,
{
    let (data_args, explicit_options) = split_same_test_options(args, 2);
    let same_test = effective_same_test(session, name, &explicit_options)?;
    let Some((lhs, rhs)) = two_list_args(data_args) else {
        return Ok(if data_args.len() == 2 {
            Expr::call(head.clone(), args.to_vec())
        } else {
            session.invalid_call_with_exact_arg_count(name, args, 2)
        });
    };

    if let Some(same_test) = same_test {
        return eval_same_test_relation(session, same_test, lhs, rhs, same_test_kind);
    }

    Ok(predicate_bool_expr(predicate(lhs, rhs)))
}

fn split_same_test_options(args: &[Expr], min_data_args: usize) -> (&[Expr], Vec<Expr>) {
    let mut option_start = args.len();
    let mut suffix_groups = Vec::<Vec<Expr>>::new();

    while option_start > min_data_args {
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

fn eval_duplicate_free_same_test(
    session: &mut Session,
    same_test: Expr,
    items: &[Expr],
) -> Result<Expr, EvalError> {
    resume_duplicate_free_same_test(
        session,
        DuplicateFreeSameTestState {
            same_test,
            items: items.to_vec(),
            lhs_index: 0,
            rhs_index: 1,
        },
    )
}

fn resume_duplicate_free_same_test(
    session: &mut Session,
    mut state: DuplicateFreeSameTestState,
) -> Result<Expr, EvalError> {
    loop {
        if state.lhs_index >= state.items.len().saturating_sub(1) {
            return Ok(predicate_bool_expr(true));
        }
        if state.rhs_index < state.items.len() {
            break;
        }
        state.lhs_index += 1;
        state.rhs_index = state.lhs_index + 1;
    }

    let result = eval_same_test_pair(
        session,
        &state.same_test,
        &state.items[state.lhs_index],
        &state.items[state.rhs_index],
    );
    let suspended_state = state.clone();
    let value = session.suspend_with_frame(result, move |session, value| {
        resume_duplicate_free_same_test_with_value(session, suspended_state, value)
    })?;

    resume_duplicate_free_same_test_with_value(session, state, value)
}

fn resume_duplicate_free_same_test_with_value(
    session: &mut Session,
    mut state: DuplicateFreeSameTestState,
    value: Expr,
) -> Result<Expr, EvalError> {
    if is_true_expr(&value) {
        return Ok(predicate_bool_expr(false));
    }

    state.rhs_index += 1;
    resume_duplicate_free_same_test(session, state)
}

fn eval_same_test_relation(
    session: &mut Session,
    same_test: Expr,
    lhs: &[Expr],
    rhs: &[Expr],
    kind: SameTestRelationKind,
) -> Result<Expr, EvalError> {
    resume_same_test_matrix(
        session,
        SameTestMatrixState {
            same_test,
            lhs: lhs.to_vec(),
            rhs: rhs.to_vec(),
            lhs_index: 0,
            rhs_index: 0,
            matches: vec![vec![false; rhs.len()]; lhs.len()],
            kind,
        },
    )
}

fn resume_same_test_matrix(
    session: &mut Session,
    mut state: SameTestMatrixState,
) -> Result<Expr, EvalError> {
    loop {
        if state.lhs_index >= state.lhs.len() {
            return Ok(predicate_bool_expr(same_test_relation_result(
                state.kind,
                &state.matches,
                state.lhs.len(),
                state.rhs.len(),
            )));
        }
        if state.rhs_index < state.rhs.len() {
            break;
        }
        state.lhs_index += 1;
        state.rhs_index = 0;
    }

    let result = eval_same_test_pair(
        session,
        &state.same_test,
        &state.lhs[state.lhs_index],
        &state.rhs[state.rhs_index],
    );
    let suspended_state = state.clone();
    let value = session.suspend_with_frame(result, move |session, value| {
        resume_same_test_matrix_with_value(session, suspended_state, value)
    })?;

    resume_same_test_matrix_with_value(session, state, value)
}

fn resume_same_test_matrix_with_value(
    session: &mut Session,
    mut state: SameTestMatrixState,
    value: Expr,
) -> Result<Expr, EvalError> {
    if is_true_expr(&value) {
        state.matches[state.lhs_index][state.rhs_index] = true;
    }
    state.rhs_index += 1;
    resume_same_test_matrix(session, state)
}

fn eval_same_test_pair(
    session: &mut Session,
    same_test: &Expr,
    lhs: &Expr,
    rhs: &Expr,
) -> Result<Expr, EvalError> {
    session.evaluate(Expr::call(
        same_test.clone(),
        vec![lhs.clone(), rhs.clone()],
    ))
}

fn is_true_expr(expr: &Expr) -> bool {
    is_system_symbol_name(expr, heads::structure::TRUE)
}

fn same_test_relation_result(
    kind: SameTestRelationKind,
    matches: &[Vec<bool>],
    lhs_len: usize,
    rhs_len: usize,
) -> bool {
    match kind {
        SameTestRelationKind::Intersecting | SameTestRelationKind::ContainsAny => {
            any_same_test_match(matches)
        }
        SameTestRelationKind::Disjoint | SameTestRelationKind::ContainsNone => {
            !any_same_test_match(matches)
        }
        SameTestRelationKind::Subset | SameTestRelationKind::ContainsAll => {
            all_rhs_items_matched(matches, rhs_len)
        }
        SameTestRelationKind::ContainsOnly => all_lhs_items_matched(matches),
        SameTestRelationKind::ContainsExactly => {
            lhs_len == rhs_len && has_perfect_same_test_matching(matches, rhs_len)
        }
    }
}

fn any_same_test_match(matches: &[Vec<bool>]) -> bool {
    matches.iter().any(|row| row.iter().any(|value| *value))
}

fn all_rhs_items_matched(matches: &[Vec<bool>], rhs_len: usize) -> bool {
    (0..rhs_len).all(|rhs_index| matches.iter().any(|row| row[rhs_index]))
}

fn all_lhs_items_matched(matches: &[Vec<bool>]) -> bool {
    matches.iter().all(|row| row.iter().any(|value| *value))
}

fn has_perfect_same_test_matching(matches: &[Vec<bool>], rhs_len: usize) -> bool {
    let mut used_rhs = vec![false; rhs_len];
    perfect_same_test_matching_from(0, matches, &mut used_rhs)
}

fn perfect_same_test_matching_from(
    lhs_index: usize,
    matches: &[Vec<bool>],
    used_rhs: &mut [bool],
) -> bool {
    if lhs_index >= matches.len() {
        return true;
    }

    for rhs_index in 0..used_rhs.len() {
        if matches[lhs_index][rhs_index] && !used_rhs[rhs_index] {
            used_rhs[rhs_index] = true;
            if perfect_same_test_matching_from(lhs_index + 1, matches, used_rhs) {
                return true;
            }
            used_rhs[rhs_index] = false;
        }
    }

    false
}
