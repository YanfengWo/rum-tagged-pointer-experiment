use super::*;

impl Session {
    pub(in crate::runtime) fn eval_enclose(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(1..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Enclose", args, 1, 3));
        }

        let handler = args.get(1).cloned();
        let form = args.get(2).cloned();
        let lexical_tag = if form.is_none() {
            Some(fresh_internal_enclose_tag_expr(self))
        } else {
            None
        };
        let body = lexical_tag.as_ref().map_or_else(
            || args[0].clone(),
            |tag| rewrite_untagged_confirms(args[0].clone(), tag),
        );

        let body_result = self.evaluate(body);
        let outcome = self.suspend_with_result_frame_lazy(body_result, || {
            let handler = handler.clone();
            let form = form.clone();
            let lexical_tag = lexical_tag.clone();
            move |session, outcome| {
                session.resume_enclose_after_body(handler, form, lexical_tag, outcome)
            }
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_enclose_after_body(handler, form, lexical_tag, outcome),
        }
    }

    pub(in crate::runtime) fn eval_confirm(
        &mut self,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if !(1..=3).contains(&args_slice.len()) {
            return Ok(self.invalid_call_with_arg_range("Confirm", args_slice, 1, 3));
        }

        let value_result = self.evaluate(args_slice[0].clone());
        let value = self.suspend_with_frame_lazy(value_result, || {
            let args = args.clone();
            move |session, value| session.resume_confirm_after_value(&args, value)
        })?;

        self.resume_confirm_after_value(args, value)
    }

    pub(in crate::runtime) fn eval_confirm_by(
        &mut self,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if !(2..=4).contains(&args_slice.len()) {
            return Ok(self.invalid_call_with_arg_range("ConfirmBy", args_slice, 2, 4));
        }

        let value_result = self.evaluate(args_slice[0].clone());
        let value = self.suspend_with_frame_lazy(value_result, || {
            let args = args.clone();
            move |session, value| session.resume_confirm_by_after_value(&args, value)
        })?;

        self.resume_confirm_by_after_value(args, value)
    }

    pub(in crate::runtime) fn eval_confirm_match(
        &mut self,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if !(2..=4).contains(&args_slice.len()) {
            return Ok(self.invalid_call_with_arg_range("ConfirmMatch", args_slice, 2, 4));
        }

        let value_result = self.evaluate(args_slice[0].clone());
        let value = self.suspend_with_frame_lazy(value_result, || {
            let args = args.clone();
            move |session, value| session.resume_confirm_match_after_value(&args, value)
        })?;

        self.resume_confirm_match_after_value(args, value)
    }

    pub(in crate::runtime) fn eval_confirm_quiet(
        &mut self,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if !(1..=4).contains(&args_slice.len()) {
            return Ok(self.invalid_call_with_arg_range("ConfirmQuiet", args_slice, 1, 4));
        }

        let mut selectors = match normalize_confirm_quiet_selectors(args_slice.get(1)) {
            Ok(selectors) => Some(selectors),
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form(
                    "ConfirmQuiet",
                    args_slice,
                    error.message(),
                ));
            }
        };

        let message_checkpoint = self.messages.len();
        let body_result = self.evaluate(args_slice[0].clone());
        let outcome = self.suspend_with_result_frame_lazy(body_result, || {
            let args = args.clone();
            let selectors = SuspendedConfirmQuietSelectors::freeze(
                selectors
                    .take()
                    .expect("confirm selectors are frozen only after input suspension"),
            );
            move |session, outcome| {
                session.resume_suspended_confirm_quiet_after_body(
                    &args,
                    &selectors,
                    message_checkpoint,
                    outcome,
                )
            }
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_confirm_quiet_after_body(
                args,
                selectors
                    .as_ref()
                    .expect("ordinary confirm selectors remain owned")
                    .view(),
                message_checkpoint,
                outcome,
            ),
        }
    }

    pub(in crate::runtime) fn eval_confirm_assert(
        &mut self,
        args: &SharedArgs,
    ) -> Result<Expr, EvalError> {
        let args_slice = args.as_ref();
        if !(1..=3).contains(&args_slice.len()) {
            return Ok(self.invalid_call_with_arg_range("ConfirmAssert", args_slice, 1, 3));
        }

        let test_result = self.evaluate(args_slice[0].clone());
        let test = self.suspend_with_frame_lazy(test_result, || {
            let args = args.clone();
            move |session, test| session.resume_confirm_assert_after_test(&args, test)
        })?;

        self.resume_confirm_assert_after_test(args, test)
    }

    fn resume_enclose_after_body(
        &mut self,
        handler: Option<Expr>,
        form: Option<Expr>,
        lexical_tag: Option<Expr>,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => Ok(value),
            Err(EvalError::Thrown {
                value,
                tag,
                explicit_tag,
            }) if is_failure_expr(&value) => self.resume_enclose_for_failure(
                handler,
                form,
                lexical_tag,
                value,
                tag,
                explicit_tag,
            ),
            Err(error) => Err(error),
        }
    }

    fn resume_enclose_for_failure(
        &mut self,
        handler: Option<Expr>,
        form: Option<Expr>,
        lexical_tag: Option<Expr>,
        value: Expr,
        tag: Expr,
        explicit_tag: bool,
    ) -> Result<Expr, EvalError> {
        if let Some(lexical_tag) = lexical_tag {
            if tag == lexical_tag {
                return self.finish_enclose_handler(handler, value);
            }

            return Err(EvalError::Thrown {
                value,
                tag,
                explicit_tag,
            });
        }

        let Some(form) = form else {
            return Err(EvalError::Thrown {
                value,
                tag,
                explicit_tag,
            });
        };

        let match_result = self.match_expr_owned(form, tag.clone(), SymbolExprMap::default());
        self.drive_internal_value(match_result, move |session, outcome| {
            if outcome.matched {
                session.finish_enclose_handler(handler, value)
            } else {
                Err(EvalError::Thrown {
                    value,
                    tag,
                    explicit_tag,
                })
            }
        })
    }

    fn finish_enclose_handler(
        &mut self,
        handler: Option<Expr>,
        failure: Expr,
    ) -> Result<Expr, EvalError> {
        match handler {
            None => Ok(failure),
            Some(Expr::Atom(Atom::String(property))) => {
                self.failure_property_value(&failure, &property)
            }
            Some(handler) => self.evaluate(Expr::call(handler, vec![failure])),
        }
    }

    fn resume_confirm_after_value(
        &mut self,
        args: &SharedArgs,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        if !is_confirmation_failure_like(&value) {
            return Ok(value);
        }

        let raw_expr = args[0].clone();
        let tag = confirmation_tag_from_args(args.as_ref(), 2);
        let info_result = evaluate_optional_confirmation_info(self, args.get(1).cloned());
        self.drive_internal_value(info_result, move |session, info| {
            session.finish_confirm_failure(raw_expr, value, info, tag)
        })
    }

    fn finish_confirm_failure(
        &mut self,
        raw_expr: Expr,
        value: Expr,
        info: Option<Expr>,
        tag: Expr,
    ) -> Result<Expr, EvalError> {
        if is_failure_expr(&value) {
            let message_result = self.failure_property_value(&value, "Message");
            return self.drive_internal_value(message_result, move |_, message| {
                Err(EvalError::Thrown {
                    value: build_confirm_failure_from_failure(raw_expr, message, info),
                    tag,
                    explicit_tag: true,
                })
            });
        }

        Err(EvalError::Thrown {
            value: build_confirm_failure(raw_expr, value, info),
            tag,
            explicit_tag: true,
        })
    }

    fn resume_confirm_by_after_value(
        &mut self,
        args: &SharedArgs,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        let test_call = Expr::call(args[1].clone(), vec![value.clone()]);
        let test_result = self.evaluate(test_call);
        let test = self.suspend_with_frame_lazy(test_result, || {
            let args = args.clone();
            let value = value.clone();
            move |session, test| session.resume_confirm_by_after_test(&args, value, test)
        })?;

        self.resume_confirm_by_after_test(args, value, test)
    }

    fn resume_confirm_by_after_test(
        &mut self,
        args: &SharedArgs,
        value: Expr,
        test: Expr,
    ) -> Result<Expr, EvalError> {
        if is_system_symbol_name(&test, "True") {
            return Ok(value);
        }

        let raw_expr = args[0].clone();
        let raw_function = args[1].clone();
        let tag = confirmation_tag_from_args(args.as_ref(), 3);
        let info_result = evaluate_optional_confirmation_info(self, args.get(2).cloned());
        self.drive_internal_value(info_result, move |session, info| {
            session.finish_confirm_by_failure(raw_expr, raw_function, info, tag)
        })
    }

    fn finish_confirm_by_failure(
        &mut self,
        raw_expr: Expr,
        raw_function: Expr,
        info: Option<Expr>,
        tag: Expr,
    ) -> Result<Expr, EvalError> {
        let failure = build_confirm_by_failure(raw_expr, raw_function, info);

        Err(EvalError::Thrown {
            value: failure,
            tag,
            explicit_tag: true,
        })
    }

    fn resume_confirm_match_after_value(
        &mut self,
        args: &SharedArgs,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        let raw_expr = args[0].clone();
        let raw_pattern = args[1].clone();
        let tag = confirmation_tag_from_args(args.as_ref(), 3);
        let info_expr = args.get(2).cloned();
        let match_result =
            self.match_expr_owned(raw_pattern.clone(), value.clone(), SymbolExprMap::default());

        self.drive_internal_value(match_result, move |session, outcome| {
            if outcome.matched {
                Ok(value)
            } else {
                let info_result = evaluate_optional_confirmation_info(session, info_expr);
                session.drive_internal_value(info_result, move |session, info| {
                    session.finish_confirm_match_failure(raw_expr, raw_pattern, info, tag)
                })
            }
        })
    }

    fn finish_confirm_match_failure(
        &mut self,
        raw_expr: Expr,
        raw_pattern: Expr,
        info: Option<Expr>,
        tag: Expr,
    ) -> Result<Expr, EvalError> {
        let failure = build_confirm_match_failure(raw_expr, raw_pattern, info);

        Err(EvalError::Thrown {
            value: failure,
            tag,
            explicit_tag: true,
        })
    }

    fn resume_confirm_quiet_after_body(
        &mut self,
        args: &SharedArgs,
        selectors: ConfirmQuietSelectorsView<'_>,
        message_checkpoint: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let matched_message =
            confirm_quiet_matched_message(&self.messages[message_checkpoint..], selectors);

        let Some(message) = matched_message else {
            return outcome;
        };

        suppress_confirm_quiet_messages(self, message_checkpoint, selectors);

        let raw_expr = args[0].clone();
        let tag = confirmation_tag_from_args(args.as_ref(), 3);
        let info_result = evaluate_optional_confirmation_info(self, args.get(2).cloned());
        self.drive_internal_value(info_result, move |session, info| {
            session.finish_confirm_quiet_failure(raw_expr, info, tag, message)
        })
    }

    fn resume_suspended_confirm_quiet_after_body(
        &mut self,
        args: &SharedArgs,
        selectors: &SuspendedConfirmQuietSelectors,
        message_checkpoint: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        self.resume_confirm_quiet_after_body(args, selectors.view(), message_checkpoint, outcome)
    }

    fn finish_confirm_quiet_failure(
        &mut self,
        raw_expr: Expr,
        info: Option<Expr>,
        tag: Expr,
        message: RuntimeMessage,
    ) -> Result<Expr, EvalError> {
        let failure = build_confirm_quiet_failure(raw_expr, info, message);

        Err(EvalError::Thrown {
            value: failure,
            tag,
            explicit_tag: true,
        })
    }

    fn resume_confirm_assert_after_test(
        &mut self,
        args: &SharedArgs,
        test: Expr,
    ) -> Result<Expr, EvalError> {
        if is_system_symbol_name(&test, "True") {
            return Ok(system_expr("Null"));
        }

        let raw_test = args[0].clone();
        let tag = confirmation_tag_from_args(args.as_ref(), 2);
        let info_result = evaluate_optional_confirmation_info(self, args.get(1).cloned());
        self.drive_internal_value(info_result, move |session, info| {
            session.finish_confirm_assert_failure(raw_test, info, tag)
        })
    }

    fn finish_confirm_assert_failure(
        &mut self,
        raw_test: Expr,
        info: Option<Expr>,
        tag: Expr,
    ) -> Result<Expr, EvalError> {
        let failure = build_confirm_assert_failure(raw_test, info);

        Err(EvalError::Thrown {
            value: failure,
            tag,
            explicit_tag: true,
        })
    }
}
