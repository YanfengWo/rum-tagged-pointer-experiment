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
        let runtime_profile_total = runtime_path_timing_start("evaluate_inner_unstable.total");
        let result = match expr {
            Expr::Atom(Atom::Symbol(symbol)) => {
                let runtime_profile = runtime_path_timing_start("evaluate_inner_unstable.symbol");
                let result = self.evaluate_symbol(symbol);
                record_profile_runtime_path_timing(
                    runtime_profile,
                    runtime_profile_outcome_for_expr_result(&result),
                );
                result
            }
            Expr::Call { head, args } => {
                let runtime_profile = runtime_path_timing_start("evaluate_inner_unstable.call");
                let result = self.evaluate_call(head.clone(), args);
                record_profile_runtime_path_timing(
                    runtime_profile,
                    runtime_profile_outcome_for_expr_result(&result),
                );
                result
            }
            other => {
                let runtime_profile = runtime_path_timing_start("evaluate_inner_unstable.atom");
                let result = Ok(other);
                record_profile_runtime_path_timing(
                    runtime_profile,
                    runtime_profile_outcome_for_expr_result(&result),
                );
                result
            }
        };
        record_profile_runtime_path_timing(
            runtime_profile_total,
            runtime_profile_outcome_for_expr_result(&result),
        );
        result
    }

    pub(super) fn evaluate_atom_fast_path(&mut self, expr: Expr) -> Result<Expr, EvalError> {
        let kind = EvaluateAtomFastPathKind::classify(self, &expr);
        let runtime_profile_total = runtime_path_timing_start("evaluate_atom_fast_path.total");
        let runtime_profile_kind = runtime_path_timing_start(kind.profile_name());
        let result = match kind {
            EvaluateAtomFastPathKind::StableSymbol | EvaluateAtomFastPathKind::Atom => Ok(expr),
            EvaluateAtomFastPathKind::UnstableSymbol | EvaluateAtomFastPathKind::Call => {
                self.evaluate(expr)
            }
        };
        record_profile_runtime_path_timing(
            runtime_profile_kind,
            runtime_profile_outcome_for_expr_result(&result),
        );
        record_profile_runtime_path_timing(
            runtime_profile_total,
            runtime_profile_outcome_for_expr_result(&result),
        );
        result
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

        if let Some(value) = self
            .definitions
            .get(&symbol)
            .and_then(|definition| definition.own_value.clone())
        {
            return self.evaluate_own_value(value);
        }

        if let Some(value) = self
            .definitions
            .get(&symbol)
            .and_then(|definition| definition.delayed_own_value.clone())
        {
            return self.evaluate_definition_rhs(None, value);
        }

        if let Some(value) = self.dynamic_own_value_binding(&symbol) {
            match value {
                Some(value) => self.evaluate_own_value(value),
                None => Ok(Expr::Atom(Atom::Symbol(symbol))),
            }
        } else if self.removed_symbols.contains(&symbol) {
            Ok(removed_symbol_expr(&symbol))
        } else {
            Ok(Expr::Atom(Atom::Symbol(symbol)))
        }
    }

    fn evaluate_own_value(&mut self, value: Expr) -> Result<Expr, EvalError> {
        match &value {
            Expr::Atom(Atom::Integer(_) | Atom::Rational(_) | Atom::String(_)) => Ok(value),
            _ => self.evaluate(value),
        }
    }

    pub(in crate::runtime) fn push_dynamic_own_value_binding(
        &mut self,
        symbol: Symbol,
        value: Option<Expr>,
    ) -> usize {
        let depth = self.dynamic_own_values.len();
        self.dynamic_own_values
            .push(DynamicOwnValueFrame { symbol, value });
        depth
    }

    pub(in crate::runtime) fn truncate_dynamic_own_value_bindings(&mut self, depth: usize) {
        if depth >= self.dynamic_own_values.len() {
            return;
        }

        self.dynamic_own_values.truncate(depth);
    }

    pub(in crate::runtime) fn dynamic_own_value_binding(
        &self,
        symbol: &Symbol,
    ) -> Option<Option<Expr>> {
        self.dynamic_own_values
            .iter()
            .rev()
            .find(|frame| frame.symbol == *symbol)
            .map(|frame| frame.value.clone())
    }

    pub(in crate::runtime) fn dynamic_scope_symbol_state_is_clean(&self, symbol: &Symbol) -> bool {
        self.definitions.get(symbol).is_none()
            && !self.attributes.contains_key(symbol)
            && !self.options.contains_key(symbol)
            && !self.syntax_information.contains_key(symbol)
            && self
                .message_symbol_counts
                .get(symbol)
                .copied()
                .unwrap_or_default()
                == 0
    }

    pub(in crate::runtime) fn clear_dynamic_scope_symbol_side_effects(&mut self, symbol: &Symbol) {
        let current_had_up_values = self
            .definitions
            .get(symbol)
            .is_some_and(|definition| !definition.up_values.is_empty());
        if self.definitions.remove(symbol).is_some() {
            self.bump_evaluation_epoch_for_symbol(symbol);
        }
        if current_had_up_values {
            self.refresh_has_up_value_definitions();
        }

        if self.attributes.remove(symbol).is_some() {
            self.bump_attribute_revision();
        }
        self.options.remove(symbol);
        self.syntax_information.remove(symbol);

        if self
            .message_symbol_counts
            .get(symbol)
            .copied()
            .unwrap_or_default()
            > 0
        {
            self.message_definitions
                .retain(|key, _| key.symbol != *symbol);
            self.rebuild_message_symbol_counts();
        }
    }

    fn has_dynamic_own_value_binding(&self, symbol: &Symbol) -> bool {
        self.dynamic_own_values
            .iter()
            .rev()
            .any(|frame| frame.symbol == *symbol)
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
    ) -> (bool, StabilityBloom) {
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
            bloom.union_assign(arg_bloom);
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
        mut bloom: StabilityBloom,
    ) -> Option<(bool, StabilityBloom)> {
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
                bloom.union_assign(lhs_bloom);
                if !lhs_stable {
                    return Some((false, bloom));
                }
                let (rhs_stable, rhs_bloom) = self.cached_expr_stability_with_bloom(rhs);
                bloom.union_assign(rhs_bloom);
                Some((rhs_stable, bloom))
            }
            "RuleDelayed" => {
                let [lhs, _rhs] = args else {
                    return Some((false, bloom));
                };
                let (lhs_stable, lhs_bloom) = self.cached_expr_stability_with_bloom(lhs);
                bloom.union_assign(lhs_bloom);
                Some((lhs_stable, bloom))
            }
            _ => None,
        }
    }

    fn all_args_stability_with_bloom(
        &mut self,
        args: &[Expr],
        mut bloom: StabilityBloom,
    ) -> (bool, StabilityBloom) {
        for arg in args {
            let (arg_stable, arg_bloom) = self.cached_expr_stability_with_bloom(arg);
            bloom.union_assign(arg_bloom);
            if !arg_stable {
                return (false, bloom);
            }
        }

        (true, bloom)
    }

    fn failure_data_stability_with_bloom(
        &mut self,
        args: &[Expr],
        bloom: StabilityBloom,
    ) -> (bool, StabilityBloom) {
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
        mut bloom: StabilityBloom,
    ) -> (bool, StabilityBloom) {
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
            bloom.union_assign(arg_bloom);
            if !arg_stable {
                return (false, bloom);
            }
        }

        (true, bloom)
    }

    pub(super) fn symbol_is_atom_evaluation_stable(&mut self, symbol: &Symbol) -> bool {
        if self.has_dynamic_own_value_binding(symbol) {
            return false;
        }

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

        if self.has_dynamic_own_value_binding(symbol) {
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
            if self.symbol_call_args_have_no_evaluation_effect(symbol, args) {
                return false;
            }
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

    fn symbol_call_args_have_no_evaluation_effect(
        &mut self,
        symbol: &Symbol,
        args: &[Expr],
    ) -> bool {
        if self.has_up_value_definitions
            || symbol.context() == SYSTEM_CONTEXT && symbol.name() == heads::patterns::DEFAULT
        {
            return false;
        }

        let attributes = self.cached_attribute_set_for_symbol(symbol);
        if attributes.contains(Attribute::Listable)
            || attributes.contains(Attribute::Orderless)
            || attributes.contains(Attribute::Flat)
            || !matches!(attributes.hold_attribute(), HoldAttribute::None)
        {
            return false;
        }

        if self
            .cached_builtin_dispatch_spec_for_symbol(symbol)
            .is_some()
        {
            return false;
        }

        let Some(definition) = self.definitions.get(symbol) else {
            return false;
        };
        if !definition.up_values.is_empty()
            || !definition.sub_values.is_empty()
            || definition.down_values.is_empty()
        {
            return false;
        }

        definition
            .down_value_shallow_signatures
            .as_ref()
            .is_some_and(|signatures| {
                signatures
                    .iter()
                    .all(|signature| !signature.could_match_args(args))
            })
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

    pub(super) fn evaluate_call(
        &mut self,
        head: SharedExpr,
        args: SharedArgs,
    ) -> Result<Expr, EvalError> {
        let runtime_profile_total = runtime_path_timing_start("evaluate_call.total");
        let runtime_profile_head = runtime_path_timing_start("evaluate_call.head_eval");
        let head_kind = EvaluateAtomFastPathKind::classify(self, head.as_ref());
        let runtime_profile_fast_path = runtime_path_timing_start("evaluate_atom_fast_path.total");
        let runtime_profile_kind = runtime_path_timing_start(head_kind.profile_name());
        let evaluated_head_result = match head_kind {
            EvaluateAtomFastPathKind::StableSymbol | EvaluateAtomFastPathKind::Atom => Ok(head),
            EvaluateAtomFastPathKind::UnstableSymbol | EvaluateAtomFastPathKind::Call => {
                self.evaluate(head.into_owned()).map(SharedExpr::new)
            }
        };
        record_profile_runtime_path_timing(
            runtime_profile_kind,
            runtime_profile_outcome_for_shared_expr_result(&evaluated_head_result),
        );
        record_profile_runtime_path_timing(
            runtime_profile_fast_path,
            runtime_profile_outcome_for_shared_expr_result(&evaluated_head_result),
        );
        record_profile_runtime_path_timing(
            runtime_profile_head,
            runtime_profile_outcome_for_shared_expr_result(&evaluated_head_result),
        );
        let evaluated_head = match evaluated_head_result {
            Ok(value) => value,
            Err(EvalError::InputRequired { request }) => {
                let args_for_resume = args.clone();
                let result = self.suspend_with_frame(
                    Err(EvalError::InputRequired { request }),
                    move |session, value| {
                        session.resume_call_after_head(SharedExpr::new(value), args_for_resume)
                    },
                );
                record_profile_runtime_path_timing(
                    runtime_profile_total,
                    runtime_profile_outcome_for_expr_result(&result),
                );
                return result;
            }
            Err(error) => {
                let result = Err(error);
                record_profile_runtime_path_timing(
                    runtime_profile_total,
                    runtime_profile_outcome_for_expr_result(&result),
                );
                return result;
            }
        };
        let result = self.resume_call_after_head(evaluated_head, args);
        record_profile_runtime_path_timing(
            runtime_profile_total,
            runtime_profile_outcome_for_expr_result(&result),
        );
        result
    }

    pub(super) fn resume_call_after_head(
        &mut self,
        evaluated_head: SharedExpr,
        args: SharedArgs,
    ) -> Result<Expr, EvalError> {
        let runtime_profile_total = runtime_path_timing_start("evaluate_call.resume_after_head");
        let runtime_profile_plan = runtime_path_timing_start("evaluate_call.build_dispatch_plan");
        let plan = self.build_call_dispatch_plan(evaluated_head.as_ref());
        record_profile_runtime_path_timing(runtime_profile_plan, RuntimeProfileOutcome::Ok);
        let state = CallDispatchState::new(args);

        let runtime_profile_dispatch = runtime_path_timing_start("evaluate_call.dispatch");
        let result = self.resume_call_with_dispatch_plan(evaluated_head, plan, state, 0);
        record_profile_runtime_path_timing(
            runtime_profile_dispatch,
            runtime_profile_outcome_for_expr_result(&result),
        );
        record_profile_runtime_path_timing(
            runtime_profile_total,
            runtime_profile_outcome_for_expr_result(&result),
        );
        result
    }

    pub(super) fn resume_call_with_dispatch_plan(
        &mut self,
        evaluated_head: impl Into<SharedExpr>,
        plan: CallDispatchPlan,
        state: CallDispatchState,
        stage_index: usize,
    ) -> Result<Expr, EvalError> {
        let evaluated_head = evaluated_head.into();
        if stage_index >= plan.len() {
            return Ok(Expr::call_shared(
                evaluated_head,
                state.into_final_shared_args(),
            ));
        }

        let stage = plan.stage(stage_index);
        let mut state = state;
        if stage == CallDispatchStage::PrepareArguments && state.evaluated_args.is_some() {
            return self.resume_call_with_dispatch_plan(
                evaluated_head,
                plan,
                state,
                stage_index + 1,
            );
        }

        let runtime_profile_stage =
            runtime_path_timing_start(call_dispatch_stage_profile_name(stage));
        let stage_result = self.run_call_dispatch_stage(
            stage,
            evaluated_head.as_ref(),
            plan.attributes,
            plan.builtin_handler,
            &mut state,
        );
        record_profile_runtime_path_timing(
            runtime_profile_stage,
            runtime_profile_outcome_for_optional_result(&stage_result),
        );

        self.drive_internal_value(stage_result, move |session, outcome| match outcome {
            Some(result) => Ok(result),
            None => {
                session.resume_call_with_dispatch_plan(evaluated_head, plan, state, stage_index + 1)
            }
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EvaluateAtomFastPathKind {
    StableSymbol,
    UnstableSymbol,
    Atom,
    Call,
}

impl EvaluateAtomFastPathKind {
    fn classify(session: &mut Session, expr: &Expr) -> Self {
        match expr {
            Expr::Atom(Atom::Symbol(symbol))
                if session.symbol_is_atom_evaluation_stable(symbol) =>
            {
                Self::StableSymbol
            }
            Expr::Atom(Atom::Symbol(_)) => Self::UnstableSymbol,
            Expr::Atom(_) => Self::Atom,
            Expr::Call { .. } => Self::Call,
        }
    }

    fn profile_name(self) -> &'static str {
        match self {
            Self::StableSymbol => "evaluate_atom_fast_path.stable_symbol",
            Self::UnstableSymbol => "evaluate_atom_fast_path.unstable_symbol",
            Self::Atom => "evaluate_atom_fast_path.atom",
            Self::Call => "evaluate_atom_fast_path.call",
        }
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

fn call_dispatch_stage_profile_name(stage: CallDispatchStage) -> &'static str {
    match stage {
        CallDispatchStage::SpecialForm => "call.stage.special_form",
        CallDispatchStage::PrepareArguments => "call.stage.prepare_arguments",
        CallDispatchStage::Listable => "call.stage.listable",
        CallDispatchStage::UpValues => "call.stage.up_values",
        CallDispatchStage::DownValues => "call.stage.down_values",
        CallDispatchStage::SubValues => "call.stage.sub_values",
        CallDispatchStage::OrdinaryBuiltin => "call.stage.ordinary_builtin",
        CallDispatchStage::Function => "call.stage.function",
    }
}

fn runtime_profile_outcome_for_optional_result(
    result: &Result<Option<Expr>, EvalError>,
) -> RuntimeProfileOutcome {
    match result {
        Ok(Some(_)) => RuntimeProfileOutcome::Some,
        Ok(None) => RuntimeProfileOutcome::None,
        Err(EvalError::Return { .. }) => RuntimeProfileOutcome::Return,
        Err(EvalError::Thrown { .. }) | Err(EvalError::ThrownBypassingCatch { .. }) => {
            RuntimeProfileOutcome::Throw
        }
        Err(EvalError::InputRequired { .. }) => RuntimeProfileOutcome::InputRequired,
        Err(_) => RuntimeProfileOutcome::Error,
    }
}

fn runtime_profile_outcome_for_expr_result(
    result: &Result<Expr, EvalError>,
) -> RuntimeProfileOutcome {
    runtime_profile_outcome_for_result(result)
}

fn runtime_profile_outcome_for_shared_expr_result(
    result: &Result<SharedExpr, EvalError>,
) -> RuntimeProfileOutcome {
    runtime_profile_outcome_for_result(result)
}

fn runtime_profile_outcome_for_result<T>(result: &Result<T, EvalError>) -> RuntimeProfileOutcome {
    match result {
        Ok(_) => RuntimeProfileOutcome::Ok,
        Err(EvalError::Return { .. }) => RuntimeProfileOutcome::Return,
        Err(EvalError::Thrown { .. }) | Err(EvalError::ThrownBypassingCatch { .. }) => {
            RuntimeProfileOutcome::Throw
        }
        Err(EvalError::InputRequired { .. }) => RuntimeProfileOutcome::InputRequired,
        Err(_) => RuntimeProfileOutcome::Error,
    }
}
