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
                let result =
                    self.suspend_with_frame_lazy(Err(EvalError::InputRequired { request }), || {
                        move |session, value| {
                            session.resume_call_after_head(SharedExpr::new(value), args_for_resume)
                        }
                    });
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
        let result = self.resume_call_with_dispatch_plan(evaluated_head, plan, state, 0, None);
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
        unevaluated_restore: Option<Box<UnevaluatedRestorePlan>>,
    ) -> Result<Expr, EvalError> {
        let evaluated_head = evaluated_head.into();
        if stage_index >= plan.len() {
            let final_args = match unevaluated_restore {
                Some(restore) => {
                    let mut restored_args = restore.prepared_args.into_vec();
                    for (position, wrapper) in restore.wrappers {
                        restored_args[position] = wrapper;
                    }
                    self.normalize_call_args_shared(
                        evaluated_head.as_ref(),
                        SharedArgs::new(restored_args),
                        plan.attributes,
                    )
                }
                None => state.into_final_shared_args(),
            };
            return Ok(Expr::call_shared(evaluated_head, final_args));
        }

        let stage = plan.stage(stage_index);
        let mut state = state;
        if stage == CallDispatchStage::PrepareArguments && state.evaluated_args.is_some() {
            return self.resume_call_with_dispatch_plan(
                evaluated_head,
                plan,
                state,
                stage_index + 1,
                unevaluated_restore,
            );
        }

        let runtime_profile_stage =
            runtime_path_timing_start(call_dispatch_stage_profile_name(stage));
        let mut unevaluated_restore = unevaluated_restore;
        let stage_result = if stage == CallDispatchStage::PrepareArguments {
            let raw_args = std::mem::take(&mut state.raw_args);
            self.prepare_arguments_for_call_with_unevaluated_restore(
                evaluated_head.as_ref().clone(),
                plan.attributes,
                raw_args,
            )
            .map(|prepared| {
                state.evaluated_args = Some(self.normalize_call_args_shared(
                    evaluated_head.as_ref(),
                    prepared.evaluated_args,
                    plan.attributes,
                ));
                unevaluated_restore = prepared.unevaluated_restore;
                None
            })
        } else {
            self.run_call_dispatch_stage(
                stage,
                evaluated_head.as_ref(),
                plan.attributes,
                plan.builtin_spec,
                &mut state,
            )
        };
        record_profile_runtime_path_timing(
            runtime_profile_stage,
            runtime_profile_outcome_for_optional_result(&stage_result),
        );

        self.drive_internal_value(stage_result, move |session, outcome| match outcome {
            Some(result) => Ok(result),
            None => session.resume_call_with_dispatch_plan(
                evaluated_head,
                plan,
                state,
                stage_index + 1,
                unevaluated_restore,
            ),
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
