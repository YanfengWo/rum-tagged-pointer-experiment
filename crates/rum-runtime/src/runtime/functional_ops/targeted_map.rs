use super::*;

enum MapThreadDispatch {
    Threaded(Expr),
    NotApplicable,
    Mismatch,
}

impl Session {
    pub(in crate::runtime) fn eval_subset_map(
        &mut self,
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
                    return Ok(self.invalid_call_with_arg_form(
                        "SubsetMap",
                        args,
                        "expected an exact part specification or list of exact part specifications as the third argument",
                    ));
                };

                if !subset_map_paths_are_compatible(&paths) {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                }

                apply_subset_map(self, head, operator, expr, &paths, args)
            }
            _ => Ok(self.invalid_call_with_arg_range("SubsetMap", args, 2, 3)),
        }
    }

    pub(in crate::runtime) fn eval_map_at(
        &mut self,
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
                    return Ok(self.invalid_call_with_arg_form(
                        "MapAt",
                        args,
                        "expected an exact part specification or list of exact part specifications as the third argument",
                    ));
                };

                self.apply_map_at_paths(operator, expr.clone(), paths)
            }
            _ => Ok(self.invalid_call_with_arg_range("MapAt", args, 2, 3)),
        }
    }

    pub(in crate::runtime) fn eval_map_thread(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [operator] => Ok(unary_operator_form("MapThread", operator)),
            [operator, expr] => self.finish_map_thread(
                head,
                args,
                operator,
                expr,
                Expr::integer(BigInt::from(1_u8)),
                1,
            ),
            [operator, expr, level_expr] => {
                let Some(level) = expr_as_non_negative_usize(level_expr) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "MapThread",
                        args,
                        "expected a positive integer level as the third argument",
                    ));
                };

                if level == 0 {
                    return Ok(self.invalid_call_with_arg_form(
                        "MapThread",
                        args,
                        "expected a positive integer level as the third argument",
                    ));
                }

                self.finish_map_thread(head, args, operator, expr, level_expr.clone(), level)
            }
            _ => Ok(self.invalid_call_with_arg_range("MapThread", args, 1, 3)),
        }
    }

    fn finish_map_thread(
        &mut self,
        head: &Expr,
        args: &[Expr],
        operator: &Expr,
        expr: &Expr,
        level_expr: Expr,
        level: usize,
    ) -> Result<Expr, EvalError> {
        let Some(inputs) = call_items_with_head(expr, &system_expr("List")) else {
            return Ok(self.invalid_call_with_arg_form(
                "MapThread",
                args,
                "expected a list of expressions as the second argument",
            ));
        };

        match map_thread_inputs(self, operator, inputs, level)? {
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
        &mut self,
        operator: &Expr,
        expr: Expr,
        paths: Vec<Vec<isize>>,
    ) -> Result<Expr, EvalError> {
        let mut current = expr;

        for path in paths {
            let Some(part) = exact_part_at_path(&current, &path) else {
                continue;
            };

            let mapped = self.evaluate(Expr::call(operator.clone(), vec![part]))?;
            current = replace_exact_path(current, &path, mapped);
        }

        Ok(current)
    }
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

fn exact_part_at_path(expr: &Expr, path: &[isize]) -> Option<Expr> {
    let Some((first, rest)) = path.split_first() else {
        return Some(expr.clone());
    };

    if *first == 0 {
        return exact_part_at_path(&functional_structural_head_expr(expr), rest);
    }

    match expr {
        Expr::Atom(_) => None,
        Expr::Call { args, .. } => {
            let index = normalize_exact_path_component(*first, args.len())?;
            exact_part_at_path(&args[index], rest)
        }
    }
}

fn normalize_exact_path_component(component: isize, len: usize) -> Option<usize> {
    if component == 0 {
        return None;
    }

    let len = isize::try_from(len).ok()?;
    let normalized = if component > 0 {
        component - 1
    } else {
        len + component
    };

    (0..len)
        .contains(&normalized)
        .then_some(normalized as usize)
}

fn functional_structural_head_expr(expr: &Expr) -> Expr {
    match expr {
        Expr::Atom(Atom::Integer(_)) => system_expr("Integer"),
        Expr::Atom(Atom::Rational(_)) => system_expr("Rational"),
        Expr::Atom(Atom::String(_)) => system_expr("String"),
        Expr::Atom(Atom::Symbol(_)) => system_expr("Symbol"),
        Expr::Call { head, .. } => head.as_ref().clone(),
    }
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
    let Some(parts) = paths
        .iter()
        .map(|path| exact_part_at_path(expr, path))
        .collect::<Option<Vec<_>>>()
    else {
        return Ok(Expr::call(head.clone(), raw_args.to_vec()));
    };

    let mapped = session.evaluate(Expr::call(
        operator.clone(),
        vec![Expr::call(system_expr("List"), parts)],
    ))?;
    let Expr::Call {
        args: replacements, ..
    } = mapped
    else {
        return Ok(Expr::call(head.clone(), raw_args.to_vec()));
    };

    if replacements.len() != paths.len() {
        return Ok(Expr::call(head.clone(), raw_args.to_vec()));
    }

    let mut current = expr.clone();
    for (path, replacement) in paths.iter().zip(replacements.into_iter()) {
        current = replace_exact_path(current, path, replacement);
    }

    session.evaluate(current)
}
