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
            BuiltinHandler::Plus => self.eval_plus(head, args),
            BuiltinHandler::Times => self.eval_times(head, args),
            BuiltinHandler::Power => self.eval_power(head, args),
            BuiltinHandler::Subtract => self.eval_subtract(head, args),
            BuiltinHandler::Divide => self.eval_divide(head, args),
            BuiltinHandler::Mod => self.eval_mod(head, args),
            BuiltinHandler::Quotient => self.eval_quotient(head, args),
            BuiltinHandler::PowerMod => self.eval_power_mod(head, args),
            BuiltinHandler::GCD => self.eval_gcd(head, args),
            BuiltinHandler::LCM => self.eval_lcm(head, args),
            BuiltinHandler::Expand => self.eval_expand(head, args),
            BuiltinHandler::Coefficient => self.eval_coefficient(head, args),
            BuiltinHandler::CoefficientList => self.eval_coefficient_list(head, args),
            BuiltinHandler::Together => self.eval_together(head, args),
            BuiltinHandler::Factor => self.eval_factor(head, args),
            BuiltinHandler::D => self.eval_d(head, args),
            BuiltinHandler::StringJoin => self.eval_string_join(head, args),
            BuiltinHandler::StringLength => self.eval_string_length(head, args),
            BuiltinHandler::Characters => self.eval_characters(head, args),
            BuiltinHandler::CharacterRange => self.eval_character_range(head, args),
            BuiltinHandler::ToString => self.eval_to_string(head, args),
            BuiltinHandler::StringRepeat => self.eval_string_repeat(head, args),
            BuiltinHandler::StringContainsQ => self.eval_string_contains_q(head, args),
            BuiltinHandler::StringStartsQ => self.eval_string_starts_q(head, args),
            BuiltinHandler::StringEndsQ => self.eval_string_ends_q(head, args),
            BuiltinHandler::StringPosition => self.eval_string_position(head, args),
            BuiltinHandler::StringSplit => self.eval_string_split(head, args),
            BuiltinHandler::StringTrim => self.eval_string_trim(head, args),
            BuiltinHandler::StringReplace => self.eval_string_replace(head, args),
            BuiltinHandler::StringCases => self.eval_string_cases(head, args),
            BuiltinHandler::StringTake => self.eval_string_take(head, args),
            BuiltinHandler::StringDrop => self.eval_string_drop(head, args),
            BuiltinHandler::StringPart => self.eval_string_part(head, args),
            BuiltinHandler::Set => self.eval_set(args),
            BuiltinHandler::SetDelayed => self.eval_set_delayed(args),
            BuiltinHandler::TagSet => self.eval_tag_set(args),
            BuiltinHandler::TagSetDelayed => self.eval_tag_set_delayed(args),
            BuiltinHandler::TagUnset => self.eval_tag_unset(args),
            BuiltinHandler::Rule => self.eval_rule(args),
            BuiltinHandler::RuleDelayed => self.eval_rule_delayed(args),
            BuiltinHandler::MatchQ => self.eval_match_q(args),
            BuiltinHandler::Cases => self.eval_cases(args),
            BuiltinHandler::Level => self.eval_level(head, args),
            BuiltinHandler::Position => self.eval_position(args),
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
                let evaluated_result = self.evaluate(args[0].clone());
                self.suspend_with_frame(evaluated_result, |_, value| Ok(value))?
            } else {
                Expr::call(head.clone(), args.to_vec())
            }),
            BuiltinHandler::Sequence => Ok(sequence_expr(args.to_vec())),
            BuiltinHandler::Thread => self.eval_thread(head, args),
            BuiltinHandler::InputString => self.eval_input_string(args),
        }
    }
}
