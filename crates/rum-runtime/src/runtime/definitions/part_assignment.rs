use super::super::*;

#[derive(Clone)]
struct PositionalPartAssignmentTarget {
    owner: Symbol,
    selectors: Vec<Expr>,
}

impl Session {
    pub(in crate::runtime) fn try_set_positional_part_lvalue(
        &mut self,
        lhs: Expr,
        rhs: Expr,
    ) -> Result<Option<Expr>, EvalError> {
        let Expr::Call { head, args } = &lhs else {
            return Ok(None);
        };
        if !is_system_symbol_name(head, "Part") {
            return Ok(None);
        }

        let Some((owner, selectors)) = args.split_first() else {
            self.push_positional_part_assignment_argform(
                "expected Part assignment to have a symbol owner and at least one selector",
            );
            return Ok(Some(rhs));
        };
        let Some(owner) = expr_as_symbol(owner).copied() else {
            self.push_positional_part_assignment_argform(format!(
                "expected Part assignment owner to be a symbol, got {owner}",
            ));
            return Ok(Some(rhs));
        };
        if selectors.is_empty() {
            self.push_positional_part_assignment_argform(
                "expected Part assignment to have at least one selector",
            );
            return Ok(Some(rhs));
        }

        let target = PositionalPartAssignmentTarget {
            owner,
            selectors: selectors.to_vec(),
        };
        let value = self.resume_positional_part_selector_evaluation(target, rhs, Vec::new(), 0)?;
        Ok(Some(value))
    }

    fn resume_positional_part_selector_evaluation(
        &mut self,
        target: PositionalPartAssignmentTarget,
        rhs: Expr,
        mut evaluated_selectors: Vec<Expr>,
        index: usize,
    ) -> Result<Expr, EvalError> {
        if index >= target.selectors.len() {
            return self.finish_positional_part_assignment(target, evaluated_selectors, rhs);
        }

        let selector_result = self.evaluate(target.selectors[index].clone());
        let selector = self.suspend_with_frame(selector_result, {
            let target = target.clone();
            let rhs = rhs.clone();
            let evaluated_selectors = evaluated_selectors.clone();
            move |session, selector| {
                let mut evaluated_selectors = evaluated_selectors;
                evaluated_selectors.push(selector);
                session.resume_positional_part_selector_evaluation(
                    target,
                    rhs,
                    evaluated_selectors,
                    index + 1,
                )
            }
        })?;

        evaluated_selectors.push(selector);
        self.resume_positional_part_selector_evaluation(target, rhs, evaluated_selectors, index + 1)
    }

    fn finish_positional_part_assignment(
        &mut self,
        target: PositionalPartAssignmentTarget,
        evaluated_selectors: Vec<Expr>,
        rhs: Expr,
    ) -> Result<Expr, EvalError> {
        // Selector evaluation can reassign, clear, protect, or remove the
        // owner. Re-fetch every target fact only after the final callback.
        let current = self
            .definitions
            .get(&target.owner)
            .and_then(|definition| definition.own_value.clone());

        if !self.ensure_symbol_not_protected_for_write("Set", &target.owner) {
            return Ok(rhs);
        }
        if self.removed_symbols.contains(&target.owner) {
            self.push_positional_part_assignment_argform(format!(
                "Part assignment owner {} has been removed",
                target.owner
            ));
            return Ok(rhs);
        }
        let Some(current) = current else {
            self.push_positional_part_assignment_argform(format!(
                "expected Part assignment owner {} to have a current List value",
                target.owner
            ));
            return Ok(rhs);
        };

        if let Some(selector) = evaluated_selectors
            .iter()
            .find(|selector| !is_positive_machine_integer_selector(selector))
        {
            self.push_positional_part_assignment_argform(format!(
                "expected a positive machine-sized integer Part selector, got {selector}",
            ));
            return Ok(rhs);
        }

        let Some(paths) =
            exact_position_paths_from_part_target_selector_exprs(&current, &evaluated_selectors)
        else {
            self.push_positional_part_assignment_argform(
                "expected an exact positive integer Part path",
            );
            return Ok(rhs);
        };
        let [path] = paths.as_slice() else {
            if !self.push_existing_part_selection_failure_messages(&current, &evaluated_selectors) {
                self.push_positional_part_assignment_argform(
                    "the requested Part path does not exist",
                );
            }
            return Ok(rhs);
        };
        if path.len() != evaluated_selectors.len() {
            self.push_positional_part_assignment_argform(
                "expected one exact position for each Part selector",
            );
            return Ok(rhs);
        }
        if let Err(non_list) = plain_list_ancestors_at_exact_path(&current, path) {
            self.push_positional_part_assignment_argform(format!(
                "expected a plain List at every level of the Part path, got {non_list}",
            ));
            return Ok(rhs);
        }

        // No evaluator call occurs from the structural rebuild through the
        // store. The generic rebuild is deliberately the baseline here.
        let updated = replace_exact_path(current, path, rhs.clone());
        self.installed_system_own_values.remove(&target.owner);
        let definition = self.definitions.entry(target.owner).or_default();
        definition.own_value = Some(updated);
        definition.delayed_own_value = None;
        self.bump_evaluation_epoch_for_symbol(&target.owner);
        Ok(rhs)
    }

    fn push_positional_part_assignment_argform(&mut self, detail: impl Into<String>) {
        self.push_message("Set", "argform", detail);
    }
}

fn is_positive_machine_integer_selector(selector: &Expr) -> bool {
    expr_as_signed_isize(selector).is_some_and(|index| index > 0)
}

fn plain_list_ancestors_at_exact_path(expr: &Expr, path: &[isize]) -> Result<(), Expr> {
    let mut current = expr;
    for component in path {
        let Expr::Call { head, args } = current else {
            return Err(current.clone());
        };
        if !is_system_symbol_name(head, heads::structure::LIST) {
            return Err(current.clone());
        }

        let Some(index) = component
            .checked_sub(1)
            .and_then(|index| usize::try_from(index).ok())
        else {
            return Err(current.clone());
        };
        let Some(child) = args.get(index) else {
            return Err(current.clone());
        };
        current = child;
    }

    Ok(())
}
