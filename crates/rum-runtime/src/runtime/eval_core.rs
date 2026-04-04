use super::*;

impl Session {
    pub(super) fn evaluate_once(&mut self, expr: Expr) -> Result<Expr, EvalError> {
        self.enforce_active_resource_limits_for_expr(&expr)?;

        if self.evaluation_stack.iter().any(|active| active == &expr) {
            return Err(EvalError::EvaluationCycleDetected {
                expr: expr.to_string(),
            });
        }

        if self.evaluation_depth >= self.resource_limits.recursion_limit {
            return Err(EvalError::RecursionLimitExceeded {
                limit: self.resource_limits.recursion_limit,
            });
        }

        self.evaluation_stack.push(expr.clone());
        self.evaluation_depth += 1;
        let result = self.evaluate_inner(expr);
        self.evaluation_depth -= 1;
        self.evaluation_stack.pop();
        match result {
            Ok(value) => {
                self.enforce_active_resource_limits_for_expr(&value)?;
                Ok(value)
            }
            Err(error) => Err(error),
        }
    }

    pub(super) fn evaluate_inner(&mut self, expr: Expr) -> Result<Expr, EvalError> {
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
                "$Path" => {
                    let paths = self
                        .package_search_paths
                        .iter()
                        .map(|path| Expr::string(path.display().to_string()))
                        .collect::<Vec<_>>();
                    return Ok(Expr::call(system_expr("List"), paths));
                }
                "$MessageList" => {
                    let messages = self
                        .last_message_list
                        .iter()
                        .cloned()
                        .map(runtime_message_name_expr)
                        .collect::<Vec<_>>();
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
            }) => self.evaluate(value),
            _ => Ok(Expr::Atom(Atom::Symbol(symbol))),
        }
    }

    pub(super) fn evaluate_call(&mut self, head: Expr, args: Vec<Expr>) -> Result<Expr, EvalError> {
        let args_for_resume = args.clone();
        let evaluated_head_result = self.evaluate(head);
        let evaluated_head = self
            .suspend_with_frame(evaluated_head_result, move |session, value| {
                session.resume_call_after_head(value, args_for_resume)
            })?;
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
