use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KeySortOrderingDecision {
    InsertBefore,
    KeepScanning,
}

impl Session {
    pub(in crate::runtime) fn eval_key_sort_by(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [function] => Ok(keyed_operator_form_slot_first(
                "KeySortBy",
                function.clone(),
            )),
            [expr, function] => {
                let Some(entries) = keyed_entry_forms_from_expr(expr) else {
                    return Ok(self.invalid_call_with_message(
                        "KeySortBy",
                        args,
                        "invrl",
                        invalid_association_or_rule_list_message(expr),
                    ));
                };

                self.resume_key_sort_by_from_index(
                    head.clone(),
                    args.to_vec(),
                    function.clone(),
                    entries,
                    0,
                    Vec::new(),
                )
            }
            _ => Ok(self.invalid_call_with_message(
                "KeySortBy",
                args,
                "argt",
                one_or_two_argument_count_message("KeySortBy", args.len()),
            )),
        }
    }

    pub(in crate::runtime) fn resume_key_sort_with_ordering_from_index(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        ordering: Expr,
        entries: Vec<AssociationEntryForm>,
        index: usize,
        sorted: Vec<AssociationEntryForm>,
    ) -> Result<Expr, EvalError> {
        let Some(candidate) = entries.get(index).cloned() else {
            return Ok(render_association_entry_forms_preserving_heads(sorted));
        };

        self.resume_key_sort_with_ordering_insert(
            head, call_args, ordering, entries, index, sorted, candidate, 0,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_key_sort_with_ordering_insert(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        ordering: Expr,
        entries: Vec<AssociationEntryForm>,
        index: usize,
        sorted: Vec<AssociationEntryForm>,
        candidate: AssociationEntryForm,
        insert_index: usize,
    ) -> Result<Expr, EvalError> {
        let Some(existing) = sorted.get(insert_index).cloned() else {
            let mut next_sorted = sorted;
            next_sorted.push(candidate);
            return self.resume_key_sort_with_ordering_from_index(
                head,
                call_args,
                ordering,
                entries,
                index + 1,
                next_sorted,
            );
        };

        let compare_result = self.evaluate(Expr::call(
            ordering.clone(),
            vec![candidate.key.clone(), existing.key],
        ));
        let suspended_head = head.clone();
        let suspended_call_args = call_args.clone();
        let suspended_ordering = ordering.clone();
        let suspended_entries = entries.clone();
        let suspended_sorted = sorted.clone();
        let suspended_candidate = candidate.clone();
        let compare_value =
            self.suspend_with_frame(compare_result, move |session, compare_value| {
                session.resume_key_sort_with_ordering_after_compare(
                    suspended_head,
                    suspended_call_args,
                    suspended_ordering,
                    suspended_entries,
                    index,
                    suspended_sorted,
                    suspended_candidate,
                    insert_index,
                    compare_value,
                )
            })?;

        self.resume_key_sort_with_ordering_after_compare(
            head,
            call_args,
            ordering,
            entries,
            index,
            sorted,
            candidate,
            insert_index,
            compare_value,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_key_sort_with_ordering_after_compare(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        ordering: Expr,
        entries: Vec<AssociationEntryForm>,
        index: usize,
        sorted: Vec<AssociationEntryForm>,
        candidate: AssociationEntryForm,
        insert_index: usize,
        compare_value: Expr,
    ) -> Result<Expr, EvalError> {
        match key_sort_ordering_decision(&compare_value) {
            KeySortOrderingDecision::InsertBefore => {
                let mut next_sorted = sorted;
                next_sorted.insert(insert_index, candidate);
                self.resume_key_sort_with_ordering_from_index(
                    head,
                    call_args,
                    ordering,
                    entries,
                    index + 1,
                    next_sorted,
                )
            }
            KeySortOrderingDecision::KeepScanning => self.resume_key_sort_with_ordering_insert(
                head,
                call_args,
                ordering,
                entries,
                index,
                sorted,
                candidate,
                insert_index + 1,
            ),
        }
    }

    fn resume_key_sort_by_from_index(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        function: Expr,
        entries: Vec<AssociationEntryForm>,
        index: usize,
        mut decorated: Vec<(AssociationEntryForm, Expr)>,
    ) -> Result<Expr, EvalError> {
        let Some(entry) = entries.get(index).cloned() else {
            decorated.sort_by(|lhs, rhs| compare_expr_canonical(&lhs.1, &rhs.1));
            return Ok(render_association_entry_forms_preserving_heads(
                decorated.into_iter().map(|(entry, _)| entry).collect(),
            ));
        };

        let sort_key_result = self.evaluate(Expr::call(function.clone(), vec![entry.key.clone()]));
        let suspended_head = head.clone();
        let suspended_call_args = call_args.clone();
        let suspended_function = function.clone();
        let suspended_entries = entries.clone();
        let suspended_decorated = decorated.clone();
        let suspended_entry = entry.clone();
        let sort_key = self.suspend_with_frame(sort_key_result, move |session, sort_key| {
            session.resume_key_sort_by_with_value(
                suspended_head,
                suspended_call_args,
                suspended_function,
                suspended_entries,
                index,
                suspended_decorated,
                suspended_entry,
                sort_key,
            )
        })?;

        self.resume_key_sort_by_with_value(
            head, call_args, function, entries, index, decorated, entry, sort_key,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn resume_key_sort_by_with_value(
        &mut self,
        head: Expr,
        call_args: Vec<Expr>,
        function: Expr,
        entries: Vec<AssociationEntryForm>,
        index: usize,
        mut decorated: Vec<(AssociationEntryForm, Expr)>,
        entry: AssociationEntryForm,
        sort_key: Expr,
    ) -> Result<Expr, EvalError> {
        decorated.push((entry, sort_key));
        self.resume_key_sort_by_from_index(head, call_args, function, entries, index + 1, decorated)
    }
}

fn key_sort_ordering_decision(value: &Expr) -> KeySortOrderingDecision {
    match value {
        Expr::Atom(Atom::Integer(integer)) if integer.as_i64() == Some(1) => {
            KeySortOrderingDecision::InsertBefore
        }
        Expr::Atom(Atom::Integer(integer))
            if integer.as_i64() == Some(0) || integer.as_i64() == Some(-1) =>
        {
            KeySortOrderingDecision::KeepScanning
        }
        _ => match truthiness_symbol(value) {
            Some(true) => KeySortOrderingDecision::InsertBefore,
            Some(false) => KeySortOrderingDecision::KeepScanning,
            None => KeySortOrderingDecision::InsertBefore,
        },
    }
}
