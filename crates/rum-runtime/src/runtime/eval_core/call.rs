use super::*;

impl Session {
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

    pub(in crate::runtime) fn resume_call_with_dispatch_plan(
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
            plan.builtin_spec,
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
