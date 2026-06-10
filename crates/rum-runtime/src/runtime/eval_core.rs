use super::*;

impl Session {
    pub(super) fn evaluate_once(&mut self, expr: Expr) -> Result<Expr, EvalError> {
        self.enforce_active_resource_limits_for_expr(&expr)?;

        let track_cycle = should_track_evaluation_cycle(&expr);
        if track_cycle && self.evaluation_stack.iter().any(|active| active == &expr) {
            return Err(EvalError::EvaluationCycleDetected {
                expr: expr.to_string(),
            });
        }

        if self.evaluation_depth >= self.resource_limits.recursion_limit {
            return Err(EvalError::RecursionLimitExceeded {
                limit: self.resource_limits.recursion_limit,
            });
        }

        if track_cycle {
            self.evaluation_stack.push(expr.clone());
        }
        self.evaluation_depth += 1;
        let result = self.evaluate_inner(expr);
        self.evaluation_depth -= 1;
        if track_cycle {
            self.evaluation_stack.pop();
        }
        match result {
            Ok(value) => {
                self.enforce_active_resource_limits_for_expr(&value)?;
                Ok(value)
            }
            Err(error) => Err(error),
        }
    }

    pub(super) fn evaluate_inner(&mut self, expr: Expr) -> Result<Expr, EvalError> {
        if self.expr_is_evaluation_stable(&expr) {
            return Ok(expr);
        }

        match expr {
            Expr::Atom(Atom::Symbol(symbol)) => self.evaluate_symbol(symbol),
            Expr::Call { head, args } => self.evaluate_call(*head, args),
            other => Ok(other),
        }
    }

    pub(super) fn evaluate_symbol(&mut self, symbol: Symbol) -> Result<Expr, EvalError> {
        if symbol.context() == SYSTEM_CONTEXT {
            match symbol.name() {
                "$Context" => return Ok(Expr::string(self.current_context.clone())),
                "$ContextPath" => {
                    let contexts = self
                        .context_path
                        .iter()
                        .cloned()
                        .map(Expr::string)
                        .collect::<Vec<_>>();
                    return Ok(Expr::call(system_expr("List"), contexts));
                }
                "$Packages" => {
                    let packages = self
                        .loaded_packages
                        .iter()
                        .cloned()
                        .map(Expr::string)
                        .collect::<Vec<_>>();
                    return Ok(Expr::call(system_expr("List"), packages));
                }
                "$InputFileName" => {
                    return Ok(Expr::string(
                        self.load_stack
                            .last()
                            .map(|path| path.display().to_string())
                            .unwrap_or_default(),
                    ));
                }
                "$Path" => {
                    let paths = self
                        .package_search_paths
                        .iter()
                        .map(|path| Expr::string(path.display().to_string()))
                        .collect::<Vec<_>>();
                    return Ok(Expr::call(system_expr("List"), paths));
                }
                "$MessageList" => {
                    let messages = self.current_message_list();
                    return Ok(Expr::call(system_expr("List"), messages));
                }
                _ => {}
            }
        }

        if self.declared_stubs.contains(&symbol)
            && !self
                .loaded_packages
                .iter()
                .any(|loaded| loaded == symbol.context())
        {
            let _ = self.eval_needs(&[Expr::string(symbol.context().to_string())])?;
        }

        let definition = self.definitions.get(&symbol).cloned();

        match definition {
            Some(SymbolDefinition {
                own_value: Some(value),
                ..
            }) => self.evaluate(value),
            Some(SymbolDefinition {
                delayed_own_value: Some(value),
                ..
            }) => self.evaluate_definition_rhs(None, value),
            _ if self.removed_symbols.contains(&symbol) => Ok(removed_symbol_expr(&symbol)),
            _ => Ok(Expr::Atom(Atom::Symbol(symbol))),
        }
    }

    fn current_message_list(&self) -> Vec<Expr> {
        if let Some(checkpoint) = self
            .current_message_list_checkpoint
            .filter(|_| self.current_message_list_dirty)
        {
            return self.messages[checkpoint..]
                .iter()
                .cloned()
                .map(held_runtime_message_name_expr)
                .collect();
        }

        self.last_message_list
            .iter()
            .cloned()
            .map(held_runtime_message_name_expr)
            .collect()
    }

    pub(super) fn expr_is_evaluation_stable(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Atom(Atom::Symbol(symbol)) => self.symbol_is_atom_evaluation_stable(symbol),
            Expr::Atom(_) => true,
            Expr::Call { head, args } => {
                self.expr_is_evaluation_stable(head)
                    && !self.call_head_has_evaluation_effect(head, args)
                    && args.iter().all(|arg| self.expr_is_evaluation_stable(arg))
            }
        }
    }

    fn symbol_is_atom_evaluation_stable(&self, symbol: &Symbol) -> bool {
        if symbol.context() == SYSTEM_CONTEXT
            && matches!(
                symbol.name(),
                "$Context"
                    | "$ContextPath"
                    | "$Packages"
                    | "$InputFileName"
                    | "$Path"
                    | "$MessageList"
            )
        {
            return false;
        }

        if self.declared_stubs.contains(symbol)
            && !self
                .loaded_packages
                .iter()
                .any(|loaded| loaded == symbol.context())
        {
            return false;
        }

        if self.removed_symbols.contains(symbol) {
            return false;
        }

        self.definitions.get(symbol).is_none_or(|definition| {
            definition.own_value.is_none()
                && definition.delayed_own_value.is_none()
                && definition.up_values.is_empty()
        })
    }

    fn call_head_has_evaluation_effect(&self, head: &Expr, args: &[Expr]) -> bool {
        let attributes = self.attribute_set_for_head(head);
        if attributes.contains(Attribute::Listable)
            || attributes.contains(Attribute::Orderless)
            || attributes.contains(Attribute::Flat)
        {
            return true;
        }

        if parse_function(head).is_some()
            || parse_composition(head).is_some()
            || is_failure_expr(head)
            || is_success_expr(head)
            || is_association_expr(head)
            || is_callable_operator_wrapper(head)
            || is_callable_failsafe_wrapper(head)
        {
            return true;
        }

        let Some(symbol) = expr_as_symbol(head) else {
            return true;
        };

        if self.builtin_dispatch_spec_for_symbol(symbol).is_some() {
            return true;
        }

        self.definitions.get(symbol).is_some_and(|definition| {
            !definition.down_values.is_empty()
                || !definition.sub_values.is_empty()
                || (!args.is_empty() && !definition.up_values.is_empty())
        })
    }

    pub(super) fn evaluate_call(&mut self, head: Expr, args: Vec<Expr>) -> Result<Expr, EvalError> {
        let evaluated_head_result = self.evaluate(head);
        let evaluated_head = match evaluated_head_result {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                let args_for_resume = args.clone();
                return self.suspend_with_frame(
                    Err(EvalError::InputRequired { request }),
                    move |session, value| session.resume_call_after_head(value, args_for_resume),
                );
            }
            Err(error) => return Err(error),
        };
        self.resume_call_after_head(evaluated_head, args)
    }

    pub(super) fn resume_call_after_head(
        &mut self,
        evaluated_head: Expr,
        args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let plan = self.build_call_dispatch_plan(&evaluated_head);
        let state = CallDispatchState::new(args);

        self.resume_call_with_dispatch_plan(evaluated_head, plan, state, 0)
    }

    pub(super) fn resume_call_with_dispatch_plan(
        &mut self,
        evaluated_head: Expr,
        plan: CallDispatchPlan,
        state: CallDispatchState,
        stage_index: usize,
    ) -> Result<Expr, EvalError> {
        if stage_index >= plan.stages.len() {
            return Ok(Expr::call(evaluated_head, state.into_final_args()));
        }

        let stage = plan.stages[stage_index];
        if stage == CallDispatchStage::PrepareArguments && state.evaluated_args.is_some() {
            return self.resume_call_with_dispatch_plan(
                evaluated_head,
                plan,
                state,
                stage_index + 1,
            );
        }

        let mut state = state;
        let stage_result =
            self.run_call_dispatch_stage(stage, &evaluated_head, plan.attributes, &mut state);

        self.drive_internal_value(stage_result, move |session, outcome| match outcome {
            Some(result) => Ok(result),
            None => {
                session.resume_call_with_dispatch_plan(evaluated_head, plan, state, stage_index + 1)
            }
        })
    }
}
