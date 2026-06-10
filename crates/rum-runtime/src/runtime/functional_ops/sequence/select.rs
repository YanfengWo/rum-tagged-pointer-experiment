use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SelectLimit {
    All,
    Count(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SelectSource {
    Call { head: Expr, items: Vec<Expr> },
    Association(Vec<(Expr, Expr)>),
}

impl SelectLimit {
    fn reached(self, count: usize) -> bool {
        match self {
            Self::All => false,
            Self::Count(limit) => count >= limit,
        }
    }
}

impl Session {
    pub(in crate::runtime) fn eval_select(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [criterion] => Ok(unary_operator_form_slot_first("Select", criterion)),
            [criterion, limit_expr] if sequence_like_from_expr(criterion).is_none() => {
                let Some(limit) = parse_select_limit(limit_expr) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "Select",
                        args,
                        "expected a sequence expression or a non-negative integer or Infinity selection limit",
                    ));
                };
                Ok(select_operator_form(criterion.clone(), limit))
            }
            [expr, criterion] => {
                let Some(source) = sequence_like_from_expr(expr) else {
                    return Ok(Expr::call(system_expr("Select"), args.to_vec()));
                };

                self.resume_select_from_index(
                    source,
                    criterion.clone(),
                    SelectLimit::All,
                    0,
                    Vec::new(),
                )
            }
            [expr, criterion, limit_expr] => {
                let Some(source) = sequence_like_from_expr(expr) else {
                    return Ok(Expr::call(system_expr("Select"), args.to_vec()));
                };
                let Some(limit) = parse_select_limit(limit_expr) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "Select",
                        args,
                        "expected a non-negative integer or Infinity selection limit",
                    ));
                };

                self.resume_select_from_index(source, criterion.clone(), limit, 0, Vec::new())
            }
            _ => Ok(self.invalid_call_with_arg_range("Select", args, 1, 3)),
        }
    }

    fn resume_select_from_index(
        &mut self,
        source: SelectSource,
        criterion: Expr,
        limit: SelectLimit,
        index: usize,
        selected: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        if limit.reached(selected.len()) || index >= source_len(&source) {
            return Ok(render_select_result(source, selected));
        }
        if index >= self.resource_limits.iteration_work_limit {
            return Err(EvalError::IterationWorkLimitExceeded {
                limit: self.resource_limits.iteration_work_limit,
            });
        }

        let Some(item) = source_item_value(&source, index) else {
            return Ok(render_select_result(source, selected));
        };
        let call = Expr::call(criterion.clone(), vec![item]);
        let suspended_source = source.clone();
        let suspended_criterion = criterion.clone();
        let suspended_selected = selected.clone();
        let value_result = self.evaluate(call);
        let value = self.suspend_with_frame(value_result, move |session, value| {
            session.resume_select_with_value(
                suspended_source,
                suspended_criterion,
                limit,
                index,
                suspended_selected,
                value,
            )
        })?;

        self.resume_select_with_value(source, criterion, limit, index, selected, value)
    }

    fn resume_select_with_value(
        &mut self,
        source: SelectSource,
        criterion: Expr,
        limit: SelectLimit,
        index: usize,
        mut selected: Vec<Expr>,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        if truthiness_symbol(&value) == Some(true) {
            let next_len = selected.len() + 1;
            self.enforce_active_memory_limit_for_projected_bytes(
                generated_expr_nodes_memory_proxy_bytes(next_len + 1),
            )?;
            self.enforce_generated_element_limit_for_count(next_len)?;
            selected.push(source_item_expr(&source, index));
        }

        self.resume_select_from_index(source, criterion, limit, index + 1, selected)
    }
}

fn parse_select_limit(expr: &Expr) -> Option<SelectLimit> {
    if is_system_symbol_name(expr, "Infinity") {
        return Some(SelectLimit::All);
    }

    expr_as_non_negative_usize(expr).map(SelectLimit::Count)
}

fn select_operator_form(criterion: Expr, limit: SelectLimit) -> Expr {
    let limit_expr = match limit {
        SelectLimit::All => system_expr("Infinity"),
        SelectLimit::Count(limit) => Expr::integer(BigInt::from(limit)),
    };
    function_body_operator(Expr::call(
        system_expr("Select"),
        vec![slot_expr(), criterion, limit_expr],
    ))
}

fn sequence_like_from_expr(expr: &Expr) -> Option<SelectSource> {
    if let Some(entries) = association_entries_from_expr(expr) {
        return Some(SelectSource::Association(entries));
    }

    let Expr::Call { head, args } = expr else {
        return None;
    };

    Some(SelectSource::Call {
        head: head.as_ref().clone(),
        items: args.to_vec(),
    })
}

fn source_len(source: &SelectSource) -> usize {
    match source {
        SelectSource::Call { items, .. } => items.len(),
        SelectSource::Association(entries) => entries.len(),
    }
}

fn source_item_value(source: &SelectSource, index: usize) -> Option<Expr> {
    match source {
        SelectSource::Call { items, .. } => items.get(index).cloned(),
        SelectSource::Association(entries) => entries.get(index).map(|(_, value)| value.clone()),
    }
}

fn source_item_expr(source: &SelectSource, index: usize) -> Expr {
    match source {
        SelectSource::Call { items, .. } => items[index].clone(),
        SelectSource::Association(entries) => {
            let (key, value) = &entries[index];
            Expr::call(system_expr("Rule"), vec![key.clone(), value.clone()])
        }
    }
}

fn render_select_result(source: SelectSource, selected: Vec<Expr>) -> Expr {
    match source {
        SelectSource::Call { head, .. } => Expr::call(head, selected),
        SelectSource::Association(_) => render_association(
            selected
                .into_iter()
                .filter_map(|expr| {
                    let Expr::Call { head, args } = expr else {
                        return None;
                    };
                    if !is_system_symbol_name(&head, "Rule") || args.len() != 2 {
                        return None;
                    }
                    Some((args[0].clone(), args[1].clone()))
                })
                .collect(),
        ),
    }
}
