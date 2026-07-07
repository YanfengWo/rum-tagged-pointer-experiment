use super::*;

#[derive(Clone)]
struct DeleteDuplicatesTestState {
    expr_head: Expr,
    test: Expr,
    items: Vec<Expr>,
    index: usize,
    kept: Vec<Expr>,
}

pub(crate) fn eval_delete_duplicates(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    match args {
        [expr] => eval_delete_duplicates_structural(session, expr, args),
        [expr, test] => eval_delete_duplicates_with_test(session, expr, test, args),
        _ => Ok(Expr::call(head.clone(), args.to_vec())),
    }
}

fn eval_delete_duplicates_structural(
    session: &mut Session,
    expr: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let Expr::Call {
        head: expr_head,
        args: expr_args,
    } = expr
    else {
        return Ok(invalid_delete_duplicates_normal_call(
            session,
            "DeleteDuplicates",
            args,
        ));
    };

    Ok(Expr::call(
        expr_head.as_ref().clone(),
        stable_unique_exprs(expr_args),
    ))
}

fn eval_delete_duplicates_with_test(
    session: &mut Session,
    expr: &Expr,
    test: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let Expr::Call {
        head: expr_head,
        args: expr_args,
    } = expr
    else {
        return Ok(invalid_delete_duplicates_normal_call(
            session,
            "DeleteDuplicates",
            args,
        ));
    };

    resume_delete_duplicates_test_from_index(
        session,
        DeleteDuplicatesTestState {
            expr_head: expr_head.as_ref().clone(),
            test: test.clone(),
            items: expr_args.to_vec(),
            index: 0,
            kept: Vec::new(),
        },
    )
}

pub(crate) fn eval_delete_duplicates_by(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    let [expr, function] = args else {
        return Ok(match args.len() {
            1 => Expr::call(head.clone(), args.to_vec()),
            _ => session.invalid_call_with_arg_range("DeleteDuplicatesBy", args, 1, 2),
        });
    };

    let Expr::Call {
        head: expr_head,
        args: expr_args,
    } = expr
    else {
        return Ok(invalid_delete_duplicates_normal_call(
            session,
            "DeleteDuplicatesBy",
            args,
        ));
    };

    resume_delete_duplicates_by_from_index(
        session,
        args.to_vec(),
        expr_head.as_ref().clone(),
        function.clone(),
        expr_args.to_vec(),
        0,
        Vec::new(),
        Vec::new(),
    )
}

fn resume_delete_duplicates_by_from_index(
    session: &mut Session,
    call_args: Vec<Expr>,
    expr_head: Expr,
    function: Expr,
    items: Vec<Expr>,
    index: usize,
    keys: Vec<Expr>,
    kept: Vec<Expr>,
) -> Result<Expr, EvalError> {
    let Some(item) = items.get(index).cloned() else {
        return Ok(Expr::call(expr_head, kept));
    };

    let key_result = session.evaluate(Expr::call(function.clone(), vec![item.clone()]));
    let suspended_call_args = call_args.clone();
    let suspended_expr_head = expr_head.clone();
    let suspended_function = function.clone();
    let suspended_items = items.clone();
    let suspended_keys = keys.clone();
    let suspended_kept = kept.clone();
    let suspended_item = item.clone();
    let key = session.suspend_with_frame(key_result, move |session, key| {
        resume_delete_duplicates_by_with_key(
            session,
            suspended_call_args,
            suspended_expr_head,
            suspended_function,
            suspended_items,
            index,
            suspended_keys,
            suspended_kept,
            suspended_item,
            key,
        )
    })?;

    resume_delete_duplicates_by_with_key(
        session, call_args, expr_head, function, items, index, keys, kept, item, key,
    )
}

#[allow(clippy::too_many_arguments)]
fn resume_delete_duplicates_by_with_key(
    session: &mut Session,
    call_args: Vec<Expr>,
    expr_head: Expr,
    function: Expr,
    items: Vec<Expr>,
    index: usize,
    mut keys: Vec<Expr>,
    mut kept: Vec<Expr>,
    item: Expr,
    key: Expr,
) -> Result<Expr, EvalError> {
    if !keys.iter().any(|existing| existing == &key) {
        keys.push(key);
        kept.push(item);
    }
    resume_delete_duplicates_by_from_index(
        session,
        call_args,
        expr_head,
        function,
        items,
        index + 1,
        keys,
        kept,
    )
}

fn resume_delete_duplicates_test_from_index(
    session: &mut Session,
    mut state: DeleteDuplicatesTestState,
) -> Result<Expr, EvalError> {
    loop {
        let Some(item) = state.items.get(state.index).cloned() else {
            return Ok(Expr::call(state.expr_head, state.kept));
        };

        if state.kept.is_empty() {
            state.kept.push(item);
            state.index += 1;
            continue;
        }

        return resume_delete_duplicates_test_from_kept(session, state, item, 0);
    }
}

fn resume_delete_duplicates_test_from_kept(
    session: &mut Session,
    mut state: DeleteDuplicatesTestState,
    item: Expr,
    kept_index: usize,
) -> Result<Expr, EvalError> {
    let Some(existing) = state.kept.get(kept_index).cloned() else {
        state.kept.push(item);
        state.index += 1;
        return resume_delete_duplicates_test_from_index(session, state);
    };

    let result = eval_binary_predicate(session, &state.test, &item, &existing);
    let suspended_state = state.clone();
    let suspended_item = item.clone();
    let value = session.suspend_with_frame(result, move |session, value| {
        resume_delete_duplicates_test_with_value(
            session,
            suspended_state,
            suspended_item,
            kept_index,
            value,
        )
    })?;

    resume_delete_duplicates_test_with_value(session, state, item, kept_index, value)
}

fn resume_delete_duplicates_test_with_value(
    session: &mut Session,
    mut state: DeleteDuplicatesTestState,
    item: Expr,
    kept_index: usize,
    value: Expr,
) -> Result<Expr, EvalError> {
    if is_true_expr(&value) {
        state.index += 1;
        resume_delete_duplicates_test_from_index(session, state)
    } else {
        resume_delete_duplicates_test_from_kept(session, state, item, kept_index + 1)
    }
}

fn invalid_delete_duplicates_normal_call(
    session: &mut Session,
    symbol: &str,
    args: &[Expr],
) -> Expr {
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

fn stable_unique_exprs(items: &[Expr]) -> Vec<Expr> {
    let mut seen = Vec::new();
    let mut kept = Vec::new();

    for item in items {
        if seen.iter().any(|existing| existing == item) {
            continue;
        }
        seen.push(item.clone());
        kept.push(item.clone());
    }

    kept
}
