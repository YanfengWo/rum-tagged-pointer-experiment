use super::*;

mod control_functional;
mod surface;
mod symbolic;
impl Session {
    pub(super) fn build_call_dispatch_plan(&self, head: &Expr) -> CallDispatchPlan {
        let attributes = self.attribute_set_for_head(head);
        let builtin_spec =
            expr_as_symbol(head).and_then(|symbol| self.builtin_dispatch_spec_for_symbol(symbol));
        let builtin_dispatch = builtin_spec.map(|spec| spec.dispatch);
        let builtin_handler = builtin_spec.and_then(|spec| spec.handler);

        if builtin_dispatch == Some(BuiltinDispatch::SpecialForm) {
            return CallDispatchPlan {
                attributes,
                builtin_handler,
                stages: vec![CallDispatchStage::SpecialForm],
            };
        }

        let mut stages = Vec::with_capacity(6);

        stages.push(CallDispatchStage::PrepareArguments);

        if attributes.contains(Attribute::Listable) {
            stages.push(CallDispatchStage::Listable);
        }

        if !attributes.contains(Attribute::HoldAllComplete) {
            stages.push(CallDispatchStage::UpValues);
        }

        stages.extend([CallDispatchStage::DownValues, CallDispatchStage::SubValues]);

        if builtin_dispatch == Some(BuiltinDispatch::Ordinary) {
            stages.push(CallDispatchStage::OrdinaryBuiltin);
        }

        if parse_function(head).is_some()
            || parse_composition(head).is_some()
            || is_failure_expr(head)
            || is_success_expr(head)
            || is_association_expr(head)
            || is_callable_operator_wrapper(head)
            || is_callable_failsafe_wrapper(head)
        {
            stages.push(CallDispatchStage::Function);
        }

        CallDispatchPlan {
            attributes,
            builtin_handler,
            stages,
        }
    }

    pub(super) fn run_call_dispatch_stage(
        &mut self,
        stage: CallDispatchStage,
        head: &Expr,
        attributes: AttributeSet,
        builtin_handler: Option<BuiltinHandler>,
        state: &mut CallDispatchState,
    ) -> Result<Option<Expr>, EvalError> {
        match stage {
            CallDispatchStage::SpecialForm => {
                self.try_dispatch_builtin_call(builtin_handler, head, state.raw_args.as_slice())
            }
            CallDispatchStage::PrepareArguments => {
                let raw_args = std::mem::take(&mut state.raw_args);
                let mut evaluated_args =
                    self.prepare_arguments_for_call(head.clone(), attributes, raw_args)?;
                evaluated_args =
                    self.normalize_call_args_by_attributes(head, evaluated_args, attributes);
                state.evaluated_args = Some(evaluated_args);
                Ok(None)
            }
            CallDispatchStage::Listable => {
                match self.dispatch_listable_stage(head, state.evaluated_args(), attributes)? {
                    ThreadingDispatch::NotApplicable => Ok(None),
                    ThreadingDispatch::Threaded(threaded) => {
                        let threaded_result = self.evaluate(threaded);
                        self.suspend_with_frame(threaded_result, |_, value| Ok(value))
                            .map(Some)
                    }
                    ThreadingDispatch::Mismatch => Ok(Some(Expr::call(
                        head.clone(),
                        state.evaluated_args().to_vec(),
                    ))),
                }
            }
            CallDispatchStage::UpValues => self.try_apply_up_values(head, state.evaluated_args()),
            CallDispatchStage::DownValues => {
                self.try_apply_down_values(head, state.evaluated_args())
            }
            CallDispatchStage::SubValues => self.try_apply_sub_values(head, state.evaluated_args()),
            CallDispatchStage::OrdinaryBuiltin => {
                self.try_dispatch_builtin_call(builtin_handler, head, state.evaluated_args())
            }
            CallDispatchStage::Function => self.try_apply_function(head, state.evaluated_args()),
        }
    }

    pub(super) fn try_dispatch_builtin_call(
        &mut self,
        builtin_handler: Option<BuiltinHandler>,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Option<Expr>, EvalError> {
        let Some(handler) = builtin_handler else {
            return Ok(None);
        };

        self.run_builtin_handler(handler, head, args).map(Some)
    }

    pub(super) fn run_builtin_handler(
        &mut self,
        handler: BuiltinHandler,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.run_builtin_handler_surface(handler, head, args)
    }
}
