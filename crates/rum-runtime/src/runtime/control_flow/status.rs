use super::*;

impl Session {
    pub(in crate::runtime) fn eval_failure(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [tag, details] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Failure", args, 2));
        };

        let Some(details) = normalize_status_details_expr(details) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::call(head.clone(), vec![tag.clone(), details]))
    }

    pub(in crate::runtime) fn eval_failure_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("FailureQ", args, 1));
        };

        Ok(system_expr(if is_failure_q_expr(expr) {
            "True"
        } else {
            "False"
        }))
    }

    pub(in crate::runtime) fn eval_success(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [tag, details] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Success", args, 2));
        };

        let Some(details) = normalize_status_details_expr(details) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::call(head.clone(), vec![tag.clone(), details]))
    }

    pub(in crate::runtime) fn eval_terminated_evaluation(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [reason] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("TerminatedEvaluation", args, 1));
        };

        Ok(Expr::call(head.clone(), vec![reason.clone()]))
    }
}

pub(in crate::runtime) fn is_failure_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::Call { head, args } if is_system_symbol_name(head, "Failure") && args.len() == 2)
}

pub(in crate::runtime) fn is_success_expr(expr: &Expr) -> bool {
    matches!(expr, Expr::Call { head, args } if is_system_symbol_name(head, "Success") && args.len() == 2)
}

pub(in crate::runtime) fn is_failure_q_expr(expr: &Expr) -> bool {
    is_failure_expr(expr)
        || is_system_symbol_name(expr, "$Failed")
        || is_system_symbol_name(expr, "$Aborted")
}

pub(in crate::runtime) fn normalize_status_details_expr(expr: &Expr) -> Option<Expr> {
    if association_entries_from_expr(expr).is_some() {
        return Some(expr.clone());
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, heads::structure::LIST) {
        return None;
    }

    let mut entries = Vec::with_capacity(args.len());
    for item in args {
        entries.push(parse_association_entry(item)?);
    }

    Some(render_association(entries))
}

pub(in crate::runtime) fn failure_property_value(failure: &Expr, property: &str) -> Expr {
    status_object_property_value(failure, property).unwrap_or_else(|| {
        Expr::call(
            system_expr("Missing"),
            vec![Expr::string("KeyAbsent"), Expr::string(property)],
        )
    })
}

pub(in crate::runtime) fn apply_status_object_to_args(head: &Expr, args: &[Expr]) -> Option<Expr> {
    if !is_failure_expr(head) && !is_success_expr(head) {
        return None;
    }

    match args {
        [Expr::Atom(Atom::String(property))] => Some(failure_property_value(head, property)),
        [Expr::Call {
            head: list_head,
            args: properties,
        }] if is_system_symbol_name(list_head, heads::structure::LIST) => {
            let values = properties
                .iter()
                .map(|property| match property {
                    Expr::Atom(Atom::String(property)) => failure_property_value(head, property),
                    other => Expr::call(head.clone(), vec![other.clone()]),
                })
                .collect();
            Some(Expr::call(system_expr("List"), values))
        }
        _ => None,
    }
}

fn status_object_property_value(status: &Expr, property: &str) -> Option<Expr> {
    let (_, details) = status_object_parts(status)?;

    if property == "Properties" {
        let properties = association_entries_from_expr(details)?
            .into_iter()
            .map(|(key, _)| key)
            .collect::<Vec<_>>();
        return Some(Expr::call(system_expr("List"), properties));
    }

    Some(
        association_value_by_key(details, &Expr::string(property)).unwrap_or_else(|| {
            Expr::call(
                system_expr("Missing"),
                vec![Expr::string("KeyAbsent"), Expr::string(property)],
            )
        }),
    )
}

fn status_object_parts(status: &Expr) -> Option<(&Expr, &Expr)> {
    let Expr::Call { head, args } = status else {
        return None;
    };

    if !(is_system_symbol_name(head, "Failure") || is_system_symbol_name(head, "Success")) {
        return None;
    }

    let [tag, details] = args.as_slice() else {
        return None;
    };

    Some((tag, details))
}
