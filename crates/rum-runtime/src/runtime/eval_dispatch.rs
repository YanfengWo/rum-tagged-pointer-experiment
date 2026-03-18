use super::*;

impl Session {
    pub(super) fn build_call_dispatch_plan(&self, head: &Expr) -> CallDispatchPlan {
        let attributes = self.attribute_set_for_head(head);
        let builtin_dispatch = expr_as_symbol(head)
            .and_then(|symbol| self.builtin_dispatch_spec_for_symbol(symbol))
            .map(|spec| spec.dispatch);

        if builtin_dispatch == Some(BuiltinDispatch::SpecialForm) {
            return CallDispatchPlan {
                attributes,
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

        if parse_function(head).is_some() {
            stages.push(CallDispatchStage::Function);
        }

        CallDispatchPlan { attributes, stages }
    }

    pub(super) fn run_call_dispatch_stage(
        &mut self,
        stage: CallDispatchStage,
        head: &Expr,
        attributes: AttributeSet,
        state: &mut CallDispatchState,
    ) -> Result<Option<Expr>, EvalError> {
        match stage {
            CallDispatchStage::SpecialForm => self.try_dispatch_builtin_call(
                BuiltinDispatch::SpecialForm,
                head,
                state.raw_args.as_slice(),
            ),
            CallDispatchStage::PrepareArguments => {
                let raw_args = std::mem::take(&mut state.raw_args);
                let mut evaluated_args =
                    self.evaluate_call_args_with_attributes(attributes, raw_args)?;
                evaluated_args =
                    self.normalize_call_args_by_attributes(head, evaluated_args, attributes);
                state.evaluated_args = Some(evaluated_args);
                Ok(None)
            }
            CallDispatchStage::Listable => {
                match self.dispatch_listable_stage(head, state.evaluated_args(), attributes)? {
                    ThreadingDispatch::NotApplicable => Ok(None),
                    ThreadingDispatch::Threaded(threaded) => self.evaluate(threaded).map(Some),
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
            CallDispatchStage::OrdinaryBuiltin => self.try_dispatch_builtin_call(
                BuiltinDispatch::Ordinary,
                head,
                state.evaluated_args(),
            ),
            CallDispatchStage::Function => self.try_apply_function(head, state.evaluated_args()),
        }
    }

    pub(super) fn try_dispatch_builtin_call(
        &mut self,
        dispatch: BuiltinDispatch,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Option<Expr>, EvalError> {
        let Some(symbol) = expr_as_symbol(head) else {
            return Ok(None);
        };

        let Some(spec) = self.builtin_dispatch_spec_for_symbol(symbol) else {
            return Ok(None);
        };

        if spec.dispatch != dispatch {
            return Ok(None);
        }

        let Some(handler) = spec.handler else {
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
        match handler {
            BuiltinHandler::Identity => Ok(if args.len() == 1 {
                args[0].clone()
            } else {
                Expr::call(head.clone(), args.to_vec())
            }),
            BuiltinHandler::Set => self.eval_set(args),
            BuiltinHandler::SetDelayed => self.eval_set_delayed(args),
            BuiltinHandler::TagSet => self.eval_tag_set(args),
            BuiltinHandler::TagSetDelayed => self.eval_tag_set_delayed(args),
            BuiltinHandler::TagUnset => self.eval_tag_unset(args),
            BuiltinHandler::Rule => self.eval_rule(args),
            BuiltinHandler::RuleDelayed => self.eval_rule_delayed(args),
            BuiltinHandler::MatchQ => self.eval_match_q(args),
            BuiltinHandler::Cases => self.eval_cases(args),
            BuiltinHandler::Module => self.eval_module(args),
            BuiltinHandler::Block => self.eval_block(args),
            BuiltinHandler::ReplaceAll => self.eval_replace_all(args),
            BuiltinHandler::ReplaceRepeated => self.eval_replace_repeated(args),
            BuiltinHandler::FindFile => self.eval_find_file(args),
            BuiltinHandler::Get => self.eval_get(args),
            BuiltinHandler::Needs => self.eval_needs(args),
            BuiltinHandler::Context => self.eval_context(args),
            BuiltinHandler::Begin => self.eval_begin(args),
            BuiltinHandler::End => self.eval_end(args),
            BuiltinHandler::BeginPackage => self.eval_begin_package(args),
            BuiltinHandler::EndPackage => self.eval_end_package(args),
            BuiltinHandler::Message => self.eval_message(args),
            BuiltinHandler::MessageName => self.eval_message_name(args),
            BuiltinHandler::Messages => self.eval_messages(args),
            BuiltinHandler::MessageDefinitions => self.eval_message_definitions(args),
            BuiltinHandler::MessageTags => self.eval_message_tags(args),
            BuiltinHandler::OwnValues => self.eval_own_values(args),
            BuiltinHandler::UpValues => self.eval_up_values(args),
            BuiltinHandler::DownValues => self.eval_down_values(args),
            BuiltinHandler::SubValues => self.eval_sub_values(args),
            BuiltinHandler::Attributes => self.eval_attributes(args),
            BuiltinHandler::SetAttributes => self.eval_set_attributes(args),
            BuiltinHandler::ClearAttributes => self.eval_clear_attributes(args),
            BuiltinHandler::Unset => self.eval_unset(args),
            BuiltinHandler::Clear => self.eval_clear(args),
            BuiltinHandler::ClearAll => self.eval_clear_all(args),
            BuiltinHandler::NormalizeRules => self.eval_normalize_rules(args),
            BuiltinHandler::Check => self.eval_check(args),
            BuiltinHandler::Quiet => self.eval_quiet(args),
            BuiltinHandler::If => self.eval_if(args),
            BuiltinHandler::Which => self.eval_which(args),
            BuiltinHandler::With => self.eval_with(args),
            BuiltinHandler::HoldLiteral => Ok(Expr::call(head.clone(), args.to_vec())),
            BuiltinHandler::Unevaluated => Ok(if args.len() == 1 {
                args[0].clone()
            } else {
                Expr::call(head.clone(), args.to_vec())
            }),
            BuiltinHandler::ReleaseHold => self.eval_release_hold(args),
            BuiltinHandler::Evaluate => Ok(if args.len() == 1 {
                self.evaluate(args[0].clone())?
            } else {
                Expr::call(head.clone(), args.to_vec())
            }),
            BuiltinHandler::Sequence => Ok(sequence_expr(args.to_vec())),
            BuiltinHandler::Thread => self.eval_thread(head, args),
        }
    }
}
