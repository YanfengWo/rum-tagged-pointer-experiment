use super::*;

impl Session {
    pub(in crate::runtime) fn eval_key_map(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [function, expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("KeyMap", args, 2));
        };

        let Some(entries) = association_entries_from_expr(expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(render_association(
            entries
                .into_iter()
                .map(|(key, value)| (Expr::call(function.clone(), vec![key]), value))
                .collect(),
        ))
    }

    pub(in crate::runtime) fn eval_key_value_map(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [function, expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("KeyValueMap", args, 2));
        };

        let Some(entries) = association_entries_from_expr(expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::call(
            system_expr("List"),
            entries
                .into_iter()
                .map(|(key, value)| Expr::call(function.clone(), vec![key, value]))
                .collect(),
        ))
    }

    pub(in crate::runtime) fn eval_key_sort(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("KeySort", args, 1));
        };

        let Some(mut entries) = association_entries_from_expr(expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        entries.sort_by(|lhs, rhs| compare_expr_canonical(&lhs.0, &rhs.0));
        Ok(render_association(entries))
    }

    pub(in crate::runtime) fn eval_association_thread(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [keys_expr, values_expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("AssociationThread", args, 2));
        };

        let Some(keys) = list_items(keys_expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let Some(values) = list_items(values_expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        if keys.len() != values.len() {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }

        Ok(render_association(
            keys.iter().cloned().zip(values.iter().cloned()).collect(),
        ))
    }

    pub(in crate::runtime) fn eval_key_union(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() < 2 {
            return Ok(self.invalid_call_with_message(
                "KeyUnion",
                args,
                "argx",
                "expected at least 2 association arguments",
            ));
        }

        let Some(associations) = parse_association_args(args) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let mut seen = std::collections::HashSet::new();
        let mut entries = Vec::new();
        for association in associations {
            for (key, value) in association {
                if seen.insert(key.clone()) {
                    entries.push((key, value));
                }
            }
        }

        Ok(render_association(entries))
    }

    pub(in crate::runtime) fn eval_key_intersection(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() < 2 {
            return Ok(self.invalid_call_with_message(
                "KeyIntersection",
                args,
                "argx",
                "expected at least 2 association arguments",
            ));
        }

        let Some(mut associations) = parse_association_args(args) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let first = associations.remove(0);
        let other_key_sets = associations
            .into_iter()
            .map(|entries| {
                entries
                    .into_iter()
                    .map(|(key, _)| key)
                    .collect::<std::collections::HashSet<_>>()
            })
            .collect::<Vec<_>>();

        Ok(render_association(
            first
                .into_iter()
                .filter(|(key, _)| other_key_sets.iter().all(|set| set.contains(key)))
                .collect(),
        ))
    }

    pub(in crate::runtime) fn eval_key_complement(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() < 2 {
            return Ok(self.invalid_call_with_message(
                "KeyComplement",
                args,
                "argx",
                "expected at least 2 association arguments",
            ));
        }

        let Some(mut associations) = parse_association_args(args) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let first = associations.remove(0);
        let excluded = associations
            .into_iter()
            .flat_map(|entries| entries.into_iter().map(|(key, _)| key))
            .collect::<std::collections::HashSet<_>>();

        Ok(render_association(
            first
                .into_iter()
                .filter(|(key, _)| !excluded.contains(key))
                .collect(),
        ))
    }

    pub(in crate::runtime) fn eval_associate_to(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr, spec] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("AssociateTo", args, 2));
        };

        let Some(mut entries) = association_entries_from_expr(expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let Some(updates) = parse_association_update_spec(spec) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        for (key, value) in updates {
            entries.retain(|(existing, _)| existing != &key);
            entries.push((key, value));
        }

        Ok(render_association(entries))
    }

    pub(in crate::runtime) fn eval_key_drop_from(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr, key_spec] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("KeyDropFrom", args, 2));
        };

        let Some(entries) = association_entries_from_expr(expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let Some(selected_keys) = parse_key_spec(key_spec) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(render_association(
            entries
                .into_iter()
                .filter(|(key, _)| !selected_keys.iter().any(|candidate| candidate == key))
                .collect(),
        ))
    }

    pub(in crate::runtime) fn eval_catenate(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [associations_expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Catenate", args, 1));
        };

        let Some(associations) = parse_association_list_expr(associations_expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(render_association(
            associations.into_iter().flatten().collect(),
        ))
    }

    pub(in crate::runtime) fn eval_counts(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("Counts", args, 1));
        };

        let Some(items) = list_items(expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let mut counts: Vec<(Expr, usize)> = Vec::new();
        for item in items {
            increment_count(&mut counts, item.clone());
        }

        Ok(render_association(
            counts
                .into_iter()
                .map(|(key, count)| (key, Expr::integer(BigInt::from(count))))
                .collect(),
        ))
    }

    pub(in crate::runtime) fn eval_position_index(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [expr] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("PositionIndex", args, 1));
        };

        let Some(items) = list_items(expr) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let mut grouped: Vec<(Expr, Vec<Expr>)> = Vec::new();
        for (index, item) in items.iter().enumerate() {
            let position = Expr::integer(BigInt::from(index + 1));
            if let Some((_, positions)) = grouped.iter_mut().find(|(key, _)| key == item) {
                positions.push(position);
            } else {
                grouped.push((item.clone(), vec![position]));
            }
        }

        Ok(render_association(
            grouped
                .into_iter()
                .map(|(key, positions)| (key, Expr::call(system_expr("List"), positions)))
                .collect(),
        ))
    }
}
