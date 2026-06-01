use super::*;

impl Session {
    pub(in crate::runtime) fn eval_merge(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [combiner] => Ok(keyed_operator_form_slot_first("Merge", combiner.clone())),
            [associations_expr, combiner] => {
                let associations =
                    match parse_keyed_entries_list_expr_with_diagnostics(associations_expr) {
                        Ok(associations) => associations,
                        Err(KeyedEntriesListError::NotList) => {
                            return Ok(self.invalid_call_with_message(
                                "Merge",
                                args,
                                "list1",
                                invalid_keyed_collection_list_message(associations_expr),
                            ));
                        }
                        Err(KeyedEntriesListError::InvalidElement(expr)) => {
                            return Ok(self.invalid_call_with_message(
                                "Merge",
                                args,
                                "list1",
                                invalid_keyed_collection_list_message(&expr),
                            ));
                        }
                    };

                if associations.is_empty() {
                    return Ok(render_association(Vec::new()));
                };

                let mut grouped: Vec<(Expr, Vec<Expr>)> = Vec::new();
                for association in associations {
                    for (key, value) in association {
                        if let Some((_, values)) =
                            grouped.iter_mut().find(|(existing, _)| existing == &key)
                        {
                            values.push(value);
                        } else {
                            grouped.push((key, vec![value]));
                        }
                    }
                }

                self.resume_merge_from_index(
                    head.clone(),
                    args.to_vec(),
                    combiner.clone(),
                    grouped,
                    0,
                    Vec::new(),
                )
            }
            _ => Ok(self.invalid_call_with_message(
                "Merge",
                args,
                "argt",
                one_or_two_argument_count_message("Merge", args.len()),
            )),
        }
    }

    pub(in crate::runtime) fn eval_counts_by(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [function] => Ok(keyed_operator_form_slot_first("CountsBy", function.clone())),
            [expr, function] => {
                let Some(items) = list_items(expr) else {
                    return Ok(self.invalid_call_with_message(
                        "CountsBy",
                        args,
                        "list",
                        list_expected_message(1, expr),
                    ));
                };

                self.resume_counts_by_from_index(
                    head.clone(),
                    args.to_vec(),
                    function.clone(),
                    items.to_vec(),
                    0,
                    Vec::new(),
                )
            }
            _ => Ok(self.invalid_call_with_message(
                "CountsBy",
                args,
                "argt",
                one_or_two_argument_count_message("CountsBy", args.len()),
            )),
        }
    }

    fn resume_merge_from_index(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        combiner: Expr,
        grouped: Vec<(Expr, Vec<Expr>)>,
        index: usize,
        merged: Vec<(Expr, Expr)>,
    ) -> Result<Expr, EvalError> {
        let Some((key, values)) = grouped.get(index).cloned() else {
            return Ok(render_association(merged));
        };

        let combined_result = self.evaluate(Expr::call(
            combiner.clone(),
            vec![Expr::call(system_expr("List"), values)],
        ));
        let suspended_head = head.clone();
        let suspended_call_args = call_args.clone();
        let suspended_combiner = combiner.clone();
        let suspended_grouped = grouped.clone();
        let suspended_merged = merged.clone();
        let suspended_key = key.clone();
        let combined = self.suspend_with_frame(combined_result, move |session, combined| {
            session.resume_merge_with_value(
                suspended_head,
                suspended_call_args,
                suspended_combiner,
                suspended_grouped,
                index,
                suspended_merged,
                suspended_key,
                combined,
            )
        })?;

        self.resume_merge_with_value(
            head, call_args, combiner, grouped, index, merged, key, combined,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_merge_with_value(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        combiner: Expr,
        grouped: Vec<(Expr, Vec<Expr>)>,
        index: usize,
        mut merged: Vec<(Expr, Expr)>,
        key: Expr,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        merged.push((key, value));
        self.resume_merge_from_index(head, call_args, combiner, grouped, index + 1, merged)
    }

    fn resume_counts_by_from_index(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        function: Expr,
        items: Vec<Expr>,
        index: usize,
        counts: Vec<(Expr, usize)>,
    ) -> Result<Expr, EvalError> {
        let Some(item) = items.get(index).cloned() else {
            return Ok(render_association(
                counts
                    .into_iter()
                    .map(|(key, count)| (key, Expr::integer(BigInt::from(count))))
                    .collect(),
            ));
        };

        let key_result = self.evaluate(Expr::call(function.clone(), vec![item]));
        let suspended_head = head.clone();
        let suspended_call_args = call_args.clone();
        let suspended_function = function.clone();
        let suspended_items = items.clone();
        let suspended_counts = counts.clone();
        let key = self.suspend_with_frame(key_result, move |session, key| {
            session.resume_counts_by_with_value(
                suspended_head,
                suspended_call_args,
                suspended_function,
                suspended_items,
                index,
                suspended_counts,
                key,
            )
        })?;

        self.resume_counts_by_with_value(head, call_args, function, items, index, counts, key)
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_counts_by_with_value(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        function: Expr,
        items: Vec<Expr>,
        index: usize,
        mut counts: Vec<(Expr, usize)>,
        key: Expr,
    ) -> Result<Expr, EvalError> {
        increment_count(&mut counts, key);
        self.resume_counts_by_from_index(head, call_args, function, items, index + 1, counts)
    }
}
