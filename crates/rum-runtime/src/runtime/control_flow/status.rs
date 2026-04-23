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
    status_object_projection_value(failure, &Expr::string(property), None)
}

pub(in crate::runtime) fn apply_status_object_to_args(head: &Expr, args: &[Expr]) -> Option<Expr> {
    if !is_failure_expr(head) && !is_success_expr(head) {
        return None;
    }

    if is_failure_expr(head) && !failure_tag_supports_projection(head) {
        return None;
    }

    match args {
        [
            list @ Expr::Call {
                head: list_head,
                args: properties,
            },
        ] if is_system_symbol_name(list_head, heads::structure::LIST) => {
            let values = properties
                .iter()
                .map(|property| status_object_projection_value(head, property, Some(list)))
                .collect();
            Some(Expr::call(system_expr("List"), values))
        }
        [property] => Some(status_object_projection_value(head, property, None)),
        _ => None,
    }
}

fn status_object_projection_value(
    status: &Expr,
    property: &Expr,
    missing_property_context: Option<&Expr>,
) -> Expr {
    if missing_property_context.is_none() {
        if is_failure_expr(status) {
            if is_system_symbol_name(property, "All") {
                return status_object_all_value(status)
                    .unwrap_or_else(|| status_object_missing_expr(status, property.clone()));
            }

            if let Expr::Atom(Atom::String(name)) = property {
                return match name.as_str() {
                    "Properties" => failure_properties_value(status)
                        .unwrap_or_else(|| status_object_missing_expr(status, property.clone())),
                    "Message" => failure_virtual_message_value(status)
                        .unwrap_or_else(|| status_object_missing_expr(status, property.clone())),
                    "StyledMessage" => failure_virtual_styled_message_value(status)
                        .unwrap_or_else(|| status_object_missing_expr(status, property.clone())),
                    _ => status_object_property_value(status, property)
                        .unwrap_or_else(|| status_object_missing_expr(status, property.clone())),
                };
            }
        } else if matches!(property, Expr::Atom(Atom::String(name)) if name == "Properties") {
            return status_object_properties_value(status)
                .unwrap_or_else(|| status_object_missing_expr(status, property.clone()));
        }
    }

    status_object_property_value(status, property).unwrap_or_else(|| {
        let missing_property = if is_failure_expr(status) {
            missing_property_context.unwrap_or(property)
        } else {
            property
        };
        status_object_missing_expr(status, missing_property.clone())
    })
}

fn status_object_properties_value(status: &Expr) -> Option<Expr> {
    let (_, details) = status_object_parts(status)?;
    let properties = association_entries_from_expr(details)?
        .into_iter()
        .map(|(key, _)| key)
        .collect::<Vec<_>>();
    Some(Expr::call(system_expr("List"), properties))
}

fn failure_properties_value(status: &Expr) -> Option<Expr> {
    let (_, details) = status_object_parts(status)?;
    let mut properties = association_entries_from_expr(details)?
        .into_iter()
        .map(|(key, _)| key)
        .collect::<Vec<_>>();
    properties.push(Expr::string("Message"));
    properties.push(Expr::string("StyledMessage"));
    properties.sort_by(compare_expr_canonical);
    properties.dedup();
    Some(Expr::call(system_expr("List"), properties))
}

fn status_object_all_value(status: &Expr) -> Option<Expr> {
    let (_, details) = status_object_parts(status)?;
    Some(details.clone())
}

fn status_object_property_value(status: &Expr, property: &Expr) -> Option<Expr> {
    let (_, details) = status_object_parts(status)?;
    association_value_by_key(details, property)
}

fn failure_virtual_message_value(status: &Expr) -> Option<Expr> {
    let (tag, _) = status_object_parts(status)?;
    match tag {
        Expr::Atom(Atom::String(tag)) => Some(Expr::string(format!(
            "A failure of type \"{tag}\" occurred."
        ))),
        Expr::Atom(Atom::Symbol(_)) => Some(Expr::string("An unknown failure occurred.")),
        _ => None,
    }
}

fn failure_virtual_styled_message_value(status: &Expr) -> Option<Expr> {
    Some(Expr::call(
        system_expr("Style"),
        vec![
            failure_virtual_message_value(status)?,
            Expr::string("Message"),
        ],
    ))
}

fn status_object_missing_expr(status: &Expr, property: Expr) -> Expr {
    Expr::call(
        system_expr("Missing"),
        vec![
            Expr::string(if is_failure_expr(status) {
                "NotAvailable"
            } else {
                "KeyAbsent"
            }),
            property,
        ],
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

fn failure_tag_supports_projection(status: &Expr) -> bool {
    matches!(
        status_object_parts(status).map(|(tag, _)| tag),
        Some(Expr::Atom(Atom::String(_) | Atom::Symbol(_)))
    )
}
