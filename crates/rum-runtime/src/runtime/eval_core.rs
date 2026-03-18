use super::*;

impl Session {
    pub(super) fn evaluate_once(&mut self, expr: Expr) -> Result<Expr, EvalError> {
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
        result
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
        let evaluated_head = self.evaluate(head)?;
        let plan = self.build_call_dispatch_plan(&evaluated_head);
        let mut state = CallDispatchState::new(args);

        for stage in plan.stages {
            if let Some(result) =
                self.run_call_dispatch_stage(stage, &evaluated_head, plan.attributes, &mut state)?
            {
                return Ok(result);
            }
        }

        Ok(Expr::call(evaluated_head, state.into_final_args()))
    }
}
