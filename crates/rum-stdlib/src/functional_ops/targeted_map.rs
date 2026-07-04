use super::*;

enum MapThreadDispatch {
    Threaded(Expr),
    NotApplicable,
    Mismatch,
}

#[derive(Clone)]
struct MapAtEvalState {
    operator: Expr,
    paths: Vec<Vec<isize>>,
}

#[derive(Clone)]
struct SubsetMapEvalState {
    expr: Expr,
    paths: Vec<Vec<isize>>,
    fallback: Expr,
}

pub(crate) fn eval_subset_map(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    match args {
        [operator, positions] => Ok(unary_operator_form_with_tail(
            "SubsetMap",
            operator.clone(),
            positions.clone(),
        )),
        [operator, expr, positions] => {
            let Some(paths) = subset_map_paths_from_expr(expr, positions) else {
                return Ok(session.invalid_call_with_arg_form(
                    "SubsetMap",
                    args,
                    "expected an exact part specification or list of exact part specifications as the third argument",
                ));
            };

            if !subset_map_paths_are_compatible(&paths) {
                return Ok(Expr::call(head.clone(), args.to_vec()));
            }

            apply_subset_map(session, head, operator, expr, &paths, args)
        }
        _ => Ok(session.invalid_call_with_arg_range("SubsetMap", args, 2, 3)),
    }
}

pub(crate) fn eval_map_at(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    match args {
        [operator, position_spec] => Ok(unary_operator_form_with_tail(
            "MapAt",
            operator.clone(),
            position_spec.clone(),
        )),
        [operator, expr, position_spec] => {
            let Some(paths) = exact_position_paths_from_expr(expr, position_spec) else {
                return Ok(session.invalid_call_with_arg_form(
                    "MapAt",
                    args,
                    "expected an exact part specification or list of exact part specifications as the third argument",
                ));
            };

            apply_map_at_paths(session, operator, expr.clone(), paths)
        }
        _ => Ok(session.invalid_call_with_arg_range("MapAt", args, 2, 3)),
    }
}

pub(crate) fn eval_map_thread(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    match args {
        [operator] => Ok(unary_operator_form("MapThread", operator)),
        [operator, expr] => finish_map_thread(
            session,
            head,
            args,
            operator,
            expr,
            Expr::integer(BigInt::from(1_u8)),
            1,
        ),
        [operator, expr, level_expr] => {
            let Some(level) = expr_as_non_negative_usize(level_expr) else {
                return Ok(session.invalid_call_with_arg_form(
                    "MapThread",
                    args,
                    "expected a positive integer level as the third argument",
                ));
            };

            if level == 0 {
                return Ok(session.invalid_call_with_arg_form(
                    "MapThread",
                    args,
                    "expected a positive integer level as the third argument",
                ));
            }

            finish_map_thread(
                session,
                head,
                args,
                operator,
                expr,
                level_expr.clone(),
                level,
            )
        }
        _ => Ok(session.invalid_call_with_arg_range("MapThread", args, 1, 3)),
    }
}

fn finish_map_thread(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
    operator: &Expr,
    expr: &Expr,
    level_expr: Expr,
    level: usize,
) -> Result<Expr, EvalError> {
    let Some(inputs) = call_items_with_head(expr, &system_expr("List")) else {
        return Ok(session.invalid_call_with_arg_form(
            "MapThread",
            args,
            "expected a list of expressions as the second argument",
        ));
    };

    match map_thread_inputs(session, operator, inputs, level)? {
        MapThreadDispatch::Threaded(threaded) => Ok(threaded),
        MapThreadDispatch::NotApplicable | MapThreadDispatch::Mismatch => Ok(if level == 1 {
            Expr::call(head.clone(), vec![operator.clone(), expr.clone()])
        } else {
            Expr::call(
                head.clone(),
                vec![operator.clone(), expr.clone(), level_expr],
            )
        }),
    }
}

fn apply_map_at_paths(
    session: &mut Session,
    operator: &Expr,
    expr: Expr,
    paths: Vec<Vec<isize>>,
) -> Result<Expr, EvalError> {
    resume_map_at_from_index(
        session,
        MapAtEvalState {
            operator: operator.clone(),
            paths,
        },
        0,
        expr,
    )
}

fn resume_map_at_from_index(
    session: &mut Session,
    state: MapAtEvalState,
    index: usize,
    current: Expr,
) -> Result<Expr, EvalError> {
    let Some(path) = state.paths.get(index) else {
        return Ok(current);
    };

    let Some(part) = exact_expr_at_path(&current, path) else {
        return resume_map_at_from_index(session, state, index + 1, current);
    };

    let suspended_state = state.clone();
    let suspended_current = current.clone();
    let mapped_result = session.evaluate(Expr::call(state.operator.clone(), vec![part]));
    let mapped = session.suspend_with_frame(mapped_result, move |session, mapped| {
        resume_map_at_with_value(session, suspended_state, index, suspended_current, mapped)
    })?;

    resume_map_at_with_value(session, state, index, current, mapped)
}

fn resume_map_at_with_value(
    session: &mut Session,
    state: MapAtEvalState,
    index: usize,
    current: Expr,
    mapped: Expr,
) -> Result<Expr, EvalError> {
    let Some(path) = state.paths.get(index) else {
        return Ok(current);
    };

    let current = replace_exact_path(current, path, mapped);
    resume_map_at_from_index(session, state, index + 1, current)
}

fn finish_subset_map_from_mapped(
    session: &mut Session,
    state: SubsetMapEvalState,
    mapped: Expr,
) -> Result<Expr, EvalError> {
    let Expr::Call {
        args: replacements, ..
    } = mapped
    else {
        return Ok(state.fallback);
    };

    if replacements.len() != state.paths.len() {
        return Ok(state.fallback);
    }

    let mut current = state.expr;
    for (path, replacement) in state.paths.iter().zip(replacements.into_iter()) {
        current = replace_exact_path(current, path, replacement);
    }

    let evaluated_result = session.evaluate(current);
    session.suspend_with_frame(evaluated_result, |_, value| Ok(value))
}

fn map_thread_inputs(
    session: &mut Session,
    operator: &Expr,
    inputs: &[Expr],
    level: usize,
) -> Result<MapThreadDispatch, EvalError> {
    if level == 1 {
        return Ok(
            match session.thread_call_args(
                operator,
                inputs,
                &system_expr("List"),
                &system_expr("List"),
                &ThreadArgSpec::All,
            )? {
                ThreadingDispatch::Threaded(expr) => MapThreadDispatch::Threaded(expr),
                ThreadingDispatch::NotApplicable => MapThreadDispatch::NotApplicable,
                ThreadingDispatch::Mismatch => MapThreadDispatch::Mismatch,
            },
        );
    }

    let Some(first) = inputs.first() else {
        return Ok(MapThreadDispatch::Threaded(Expr::call(
            system_expr("List"),
            Vec::new(),
        )));
    };

    let Expr::Call {
        head: common_head,
        args: first_args,
    } = first
    else {
        return Ok(MapThreadDispatch::NotApplicable);
    };

    if is_system_symbol_name(common_head, "Association") {
        return Ok(MapThreadDispatch::NotApplicable);
    }

    let mut all_args = vec![first_args.clone()];
    let mut lengths = vec![first_args.len()];

    for input in inputs.iter().skip(1) {
        let Expr::Call { head, args } = input else {
            return Ok(MapThreadDispatch::NotApplicable);
        };

        if head.as_ref() != common_head.as_ref() {
            return Ok(MapThreadDispatch::NotApplicable);
        }

        lengths.push(args.len());
        if args.len() != first_args.len() {
            session.emit_thread_length_message(&system_expr("MapThread"), common_head, &lengths);
            return Ok(MapThreadDispatch::Mismatch);
        }

        all_args.push(args.clone());
    }

    let mut mapped = Vec::with_capacity(first_args.len());
    for index in 0..first_args.len() {
        let child_inputs = all_args
            .iter()
            .map(|args| args[index].clone())
            .collect::<Vec<_>>();

        match map_thread_inputs(session, operator, &child_inputs, level - 1)? {
            MapThreadDispatch::Threaded(expr) => mapped.push(expr),
            MapThreadDispatch::NotApplicable => return Ok(MapThreadDispatch::NotApplicable),
            MapThreadDispatch::Mismatch => return Ok(MapThreadDispatch::Mismatch),
        }
    }

    Ok(MapThreadDispatch::Threaded(Expr::call(
        common_head.as_ref().clone(),
        mapped,
    )))
}

fn subset_map_paths_are_compatible(paths: &[Vec<isize>]) -> bool {
    for (index, path) in paths.iter().enumerate() {
        for other in paths.iter().skip(index + 1) {
            if path == other
                || path.starts_with(other.as_slice())
                || other.starts_with(path.as_slice())
            {
                return false;
            }
        }
    }

    true
}

fn subset_map_paths_from_expr(expr: &Expr, spec_expr: &Expr) -> Option<Vec<Vec<isize>>> {
    let Some(items) = call_items_with_head(spec_expr, &system_expr("List")) else {
        return exact_position_paths_from_expr(expr, spec_expr);
    };

    if items
        .iter()
        .all(|item| !matches!(item, Expr::Call { head, .. } if is_system_symbol_name(head, "List")))
    {
        let mut paths = Vec::new();
        for item in items {
            paths.extend(exact_position_paths_from_expr(expr, item)?);
        }
        Some(paths)
    } else {
        exact_position_paths_from_expr(expr, spec_expr)
    }
}

fn apply_subset_map(
    session: &mut Session,
    head: &Expr,
    operator: &Expr,
    expr: &Expr,
    paths: &[Vec<isize>],
    raw_args: &[Expr],
) -> Result<Expr, EvalError> {
    let fallback = Expr::call(head.clone(), raw_args.to_vec());
    let Some(parts) = paths
        .iter()
        .map(|path| exact_expr_at_path(expr, path))
        .collect::<Option<Vec<_>>>()
    else {
        return Ok(fallback);
    };

    let state = SubsetMapEvalState {
        expr: expr.clone(),
        paths: paths.to_vec(),
        fallback,
    };
    let suspended_state = state.clone();
    let mapped_result = session.evaluate(Expr::call(
        operator.clone(),
        vec![Expr::call(system_expr("List"), parts)],
    ));
    let mapped = session.suspend_with_frame(mapped_result, move |session, mapped| {
        finish_subset_map_from_mapped(session, suspended_state, mapped)
    })?;

    finish_subset_map_from_mapped(session, state, mapped)
}
