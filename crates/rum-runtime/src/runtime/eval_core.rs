use super::*;

impl Session {
    pub(super) fn evaluate_once_unstable_after_input_check_with_cycle_tracking(
        &mut self,
        expr: Expr,
        track_cycle: bool,
    ) -> Result<Expr, EvalError> {
        if track_cycle
            && self
                .evaluation_stack
                .iter()
                .any(|active| active.epoch == self.evaluation_epoch && active.expr == expr)
        {
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
            self.evaluation_stack.push(EvaluationStackEntry {
                expr: expr.clone(),
                epoch: self.evaluation_epoch,
            });
        }
        self.evaluation_depth += 1;
        let result = self.evaluate_inner_unstable(expr);
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

    pub(super) fn evaluate_inner_unstable(&mut self, expr: Expr) -> Result<Expr, EvalError> {
        match expr {
            Expr::Atom(Atom::Symbol(symbol)) => self.evaluate_symbol(symbol),
            Expr::Call { head, args } => self.evaluate_call(head.into_owned(), args.into_vec()),
            other => Ok(other),
        }
    }

    pub(super) fn evaluate_atom_fast_path(&mut self, expr: Expr) -> Result<Expr, EvalError> {
        match &expr {
            Expr::Atom(Atom::Symbol(symbol)) if self.symbol_is_atom_evaluation_stable(symbol) => {
                Ok(expr)
            }
            Expr::Atom(Atom::Symbol(_)) => self.evaluate(expr),
            Expr::Atom(_) => Ok(expr),
            _ => self.evaluate(expr),
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

    /// Uncached stability walk that also reports the bloom over every symbol
    /// whose definition state the verdict depends on. On an unstable verdict
    /// the walk stops at the deciding subterm; the symbols visited up to that
    /// point are exactly the ones that can flip the verdict back.
    pub(super) fn call_stability_uncached_with_bloom(
        &mut self,
        head: &SharedExpr,
        args: &SharedArgs,
    ) -> (bool, u128) {
        let (head_stable, mut bloom) = self.cached_expr_stability_with_bloom(head);
        if !head_stable {
            return (false, bloom);
        }

        if let Some((stable, data_bloom)) =
            self.data_constructor_stability_with_bloom(head, args, bloom)
        {
            return (stable, data_bloom);
        }

        if self.call_head_has_evaluation_effect(head, args) {
            return (false, bloom);
        }

        for arg in args.iter() {
            let (arg_stable, arg_bloom) = self.cached_expr_stability_with_bloom(arg);
            bloom |= arg_bloom;
            if !arg_stable {
                return (false, bloom);
            }
        }

        (true, bloom)
    }

    fn data_constructor_stability_with_bloom(
        &mut self,
        head: &Expr,
        args: &[Expr],
        mut bloom: u128,
    ) -> Option<(bool, u128)> {
        let symbol = expr_as_symbol(head)?;
        if symbol.context() != SYSTEM_CONTEXT {
            return None;
        }

        match symbol.name() {
            "Association" => Some(self.association_data_stability_with_bloom(args, bloom)),
            "Failure" => Some(self.failure_data_stability_with_bloom(args, bloom)),
            "Missing" | "Success" | "TerminatedEvaluation" => {
                Some(self.all_args_stability_with_bloom(args, bloom))
            }
            "HoldAllComplete" | "HoldComplete" => Some((true, bloom)),
            "Function" | "Hold" | "HoldAll" | "HoldPattern" => {
                Some((hold_all_literal_args_are_stable(args), bloom))
            }
            "Rule" => {
                let [lhs, rhs] = args else {
                    return Some((false, bloom));
                };
                let (lhs_stable, lhs_bloom) = self.cached_expr_stability_with_bloom(lhs);
                bloom |= lhs_bloom;
                if !lhs_stable {
                    return Some((false, bloom));
                }
                let (rhs_stable, rhs_bloom) = self.cached_expr_stability_with_bloom(rhs);
                bloom |= rhs_bloom;
                Some((rhs_stable, bloom))
            }
            "RuleDelayed" => {
                let [lhs, _rhs] = args else {
                    return Some((false, bloom));
                };
                let (lhs_stable, lhs_bloom) = self.cached_expr_stability_with_bloom(lhs);
                bloom |= lhs_bloom;
                Some((lhs_stable, bloom))
            }
            _ => None,
        }
    }

    fn all_args_stability_with_bloom(&mut self, args: &[Expr], mut bloom: u128) -> (bool, u128) {
        for arg in args {
            let (arg_stable, arg_bloom) = self.cached_expr_stability_with_bloom(arg);
            bloom |= arg_bloom;
            if !arg_stable {
                return (false, bloom);
            }
        }

        (true, bloom)
    }

    fn failure_data_stability_with_bloom(&mut self, args: &[Expr], bloom: u128) -> (bool, u128) {
        let [_, details] = args else {
            return self.all_args_stability_with_bloom(args, bloom);
        };
        if failure_details_would_normalize(details) {
            return (false, bloom);
        }

        self.all_args_stability_with_bloom(args, bloom)
    }

    fn association_data_stability_with_bloom(
        &mut self,
        args: &[Expr],
        mut bloom: u128,
    ) -> (bool, u128) {
        let mut keys = Vec::with_capacity(args.len());
        for arg in args {
            let Some(entry) = parse_association_entry_form(arg) else {
                return (false, bloom);
            };
            if keys.iter().any(|key| key == &entry.key) {
                return (false, bloom);
            }
            keys.push(entry.key);

            let (arg_stable, arg_bloom) = self.cached_expr_stability_with_bloom(arg);
            bloom |= arg_bloom;
            if !arg_stable {
                return (false, bloom);
            }
        }

        (true, bloom)
    }

    pub(super) fn symbol_is_atom_evaluation_stable(&mut self, symbol: &Symbol) -> bool {
        if let Some(entry) = self.stable_symbol_cache.get(symbol) {
            if entry.generation == self.symbol_cache_generation {
                return entry.stable;
            }
        }

        let stable = self.symbol_is_atom_evaluation_stable_uncached(symbol);
        self.stable_symbol_cache.insert(
            *symbol,
            SymbolStabilityCacheEntry {
                generation: self.symbol_cache_generation,
                stable,
            },
        );
        stable
    }

    fn symbol_is_atom_evaluation_stable_uncached(&self, symbol: &Symbol) -> bool {
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

    fn call_head_has_evaluation_effect(&mut self, head: &Expr, args: &[Expr]) -> bool {
        if let Some(symbol) = expr_as_symbol(head) {
            return self.symbol_call_head_has_evaluation_effect(symbol, args.is_empty());
        }

        let attributes = self.cached_attribute_set_for_head(head);
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

        true
    }

    fn symbol_call_head_has_evaluation_effect(
        &mut self,
        symbol: &Symbol,
        args_empty: bool,
    ) -> bool {
        let key = SymbolCallEffectKey {
            symbol: symbol.clone(),
            args_empty,
        };
        if let Some(entry) = self.symbol_call_effect_cache.get(&key) {
            if entry.generation == self.symbol_cache_generation {
                return entry.has_effect;
            }
        }

        let has_effect = self.symbol_call_head_has_evaluation_effect_uncached(symbol, args_empty);
        self.symbol_call_effect_cache.insert(
            key,
            CallEffectCacheEntry {
                generation: self.symbol_cache_generation,
                has_effect,
            },
        );
        has_effect
    }

    fn symbol_call_head_has_evaluation_effect_uncached(
        &mut self,
        symbol: &Symbol,
        args_empty: bool,
    ) -> bool {
        let attributes = self.cached_attribute_set_for_symbol(symbol);
        if attributes.contains(Attribute::Listable)
            || attributes.contains(Attribute::Orderless)
            || attributes.contains(Attribute::Flat)
        {
            return true;
        }

        if self
            .cached_builtin_dispatch_spec_for_symbol(symbol)
            .is_some()
        {
            return true;
        }

        self.definitions.get(symbol).is_some_and(|definition| {
            !definition.down_values.is_empty()
                || !definition.sub_values.is_empty()
                || (!args_empty && !definition.up_values.is_empty())
        })
    }

    pub(super) fn evaluate_call(&mut self, head: Expr, args: Vec<Expr>) -> Result<Expr, EvalError> {
        let evaluated_head_result = self.evaluate_atom_fast_path(head);
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
        if stage_index >= plan.len() {
            return Ok(Expr::call(evaluated_head, state.into_final_args()));
        }

        let stage = plan.stage(stage_index);
        if stage == CallDispatchStage::PrepareArguments && state.evaluated_args.is_some() {
            return self.resume_call_with_dispatch_plan(
                evaluated_head,
                plan,
                state,
                stage_index + 1,
            );
        }

        let mut state = state;
        let stage_result = self.run_call_dispatch_stage(
            stage,
            &evaluated_head,
            plan.attributes,
            plan.builtin_handler,
            &mut state,
        );

        self.drive_internal_value(stage_result, move |session, outcome| match outcome {
            Some(result) => Ok(result),
            None => {
                session.resume_call_with_dispatch_plan(evaluated_head, plan, state, stage_index + 1)
            }
        })
    }
}

fn hold_all_literal_args_are_stable(args: &[Expr]) -> bool {
    args.iter().all(|arg| !is_direct_hold_escape_wrapper(arg))
}

fn is_direct_hold_escape_wrapper(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Call { head, args }
            if args.len() == 1
                && (is_system_symbol_name(head, "Evaluate")
                    || is_system_symbol_name(head, "Unevaluated"))
    )
}

fn failure_details_would_normalize(expr: &Expr) -> bool {
    if association_entries_from_expr(expr).is_some() {
        return false;
    }

    if parse_association_entry_form(expr).is_some() {
        return true;
    }

    let Expr::Call { head, args } = expr else {
        return false;
    };
    is_system_symbol_name(head, heads::structure::LIST)
        && args.iter().all(immediate_rule_entry_expr)
}

fn immediate_rule_entry_expr(expr: &Expr) -> bool {
    matches!(
        expr,
        Expr::Call { head, args }
            if is_system_symbol_name(head, "Rule") && args.len() == 2
    )
}
