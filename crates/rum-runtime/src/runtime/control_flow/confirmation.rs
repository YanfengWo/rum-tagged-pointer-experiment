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
        let outcome = self.suspend_with_result_frame(body_result, {
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

    pub(in crate::runtime) fn eval_confirm(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(1..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Confirm", args, 1, 3));
        }

        let value_result = self.evaluate(args[0].clone());
        let value = self.suspend_with_frame(value_result, {
            let args = args.to_vec();
            move |session, value| session.resume_confirm_after_value(args, value)
        })?;

        self.resume_confirm_after_value(args.to_vec(), value)
    }

    pub(in crate::runtime) fn eval_confirm_by(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(2..=4).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("ConfirmBy", args, 2, 4));
        }

        let value_result = self.evaluate(args[0].clone());
        let value = self.suspend_with_frame(value_result, {
            let args = args.to_vec();
            move |session, value| session.resume_confirm_by_after_value(args, value)
        })?;

        self.resume_confirm_by_after_value(args.to_vec(), value)
    }

    pub(in crate::runtime) fn eval_confirm_match(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(2..=4).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("ConfirmMatch", args, 2, 4));
        }

        let value_result = self.evaluate(args[0].clone());
        let value = self.suspend_with_frame(value_result, {
            let args = args.to_vec();
            move |session, value| session.resume_confirm_match_after_value(args, value)
        })?;

        self.resume_confirm_match_after_value(args.to_vec(), value)
    }

    pub(in crate::runtime) fn eval_confirm_quiet(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(1..=4).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("ConfirmQuiet", args, 1, 4));
        }

        let selectors = match normalize_confirm_quiet_selectors(args.get(1)) {
            Ok(selectors) => selectors,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("ConfirmQuiet", args, error.message()));
            }
        };

        let message_checkpoint = self.messages.len();
        let body_result = self.evaluate(args[0].clone());
        let outcome = self.suspend_with_result_frame(body_result, {
            let args = args.to_vec();
            let selectors = selectors.clone();
            move |session, outcome| {
                session.resume_confirm_quiet_after_body(
                    args,
                    selectors,
                    message_checkpoint,
                    outcome,
                )
            }
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_confirm_quiet_after_body(
                args.to_vec(),
                selectors,
                message_checkpoint,
                outcome,
            ),
        }
    }

    pub(in crate::runtime) fn eval_confirm_assert(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(1..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("ConfirmAssert", args, 1, 3));
        }

        let test_result = self.evaluate(args[0].clone());
        let test = self.suspend_with_frame(test_result, {
            let args = args.to_vec();
            move |session, test| session.resume_confirm_assert_after_test(args, test)
        })?;

        self.resume_confirm_assert_after_test(args.to_vec(), test)
    }

    pub(in crate::runtime) fn eval_return(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [] => Err(EvalError::Return {
                value: system_expr("Null"),
            }),
            [value] => Err(EvalError::Return {
                value: value.clone(),
            }),
            _ => Ok(self.invalid_call_with_arg_range("Return", args, 0, 1)),
        }
    }

    pub(in crate::runtime) fn eval_throw(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [value] => Err(EvalError::Thrown {
                value: value.clone(),
                tag: system_expr("None"),
            }),
            [value, tag] => Err(EvalError::Thrown {
                value: value.clone(),
                tag: tag.clone(),
            }),
            _ => Ok(self.invalid_call_with_arg_range("Throw", args, 1, 2)),
        }
    }

    pub(in crate::runtime) fn eval_break(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !args.is_empty() {
            return Ok(self.invalid_call_with_exact_arg_count("Break", args, 0));
        }

        Err(EvalError::Break)
    }

    pub(in crate::runtime) fn eval_continue(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !args.is_empty() {
            return Ok(self.invalid_call_with_exact_arg_count("Continue", args, 0));
        }

        Err(EvalError::Continue)
    }

    pub(in crate::runtime) fn eval_catch(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(1..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Catch", args, 1, 3));
        }

        let form = args.get(1).cloned();
        let handler = args.get(2).cloned();
        let body_result = self.evaluate(args[0].clone());
        let outcome = self.suspend_with_result_frame(body_result, {
            let form = form.clone();
            let handler = handler.clone();
            move |session, outcome| session.resume_catch_after_body(form, handler, outcome)
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_catch_after_body(form, handler, outcome),
        }
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
            Err(EvalError::Thrown { value, tag }) if is_failure_expr(&value) => {
                self.resume_enclose_for_failure(handler, form, lexical_tag, value, tag)
            }
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
    ) -> Result<Expr, EvalError> {
        if let Some(lexical_tag) = lexical_tag {
            if tag == lexical_tag {
                return self.finish_enclose_handler(handler, value);
            }

            return Err(EvalError::Thrown { value, tag });
        }

        let Some(form) = form else {
            return Err(EvalError::Thrown { value, tag });
        };

        let match_result = self.match_expr_owned(form, tag.clone(), BTreeMap::new());
        self.drive_internal_value(match_result, move |session, outcome| {
            if outcome.matched {
                session.finish_enclose_handler(handler, value)
            } else {
                Err(EvalError::Thrown { value, tag })
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
        args: Vec<Expr>,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        if !is_confirmation_failure_like(&value) {
            return Ok(value);
        }

        let raw_expr = args[0].clone();
        let tag = confirmation_tag_from_args(&args, 2);
        let info_result = evaluate_optional_confirmation_info(self, args.get(1).cloned());
        self.drive_internal_value(info_result, move |session, info| {
            session.finish_confirm_failure(args, raw_expr, value, info, tag)
        })
    }

    fn finish_confirm_failure(
        &mut self,
        args: Vec<Expr>,
        raw_expr: Expr,
        value: Expr,
        info: Option<Expr>,
        tag: Expr,
    ) -> Result<Expr, EvalError> {
        let failure = if info.is_none() && is_failure_expr(&value) {
            value
        } else {
            build_confirmation_failure(vec![
                ("Information", info.unwrap_or_else(|| system_expr("Null"))),
                ("Expression", raw_expr),
            ])
        };

        let _ = args;
        Err(EvalError::Thrown {
            value: failure,
            tag,
        })
    }

    fn resume_confirm_by_after_value(
        &mut self,
        args: Vec<Expr>,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        let test_call = Expr::call(args[1].clone(), vec![value.clone()]);
        let test_result = self.evaluate(test_call);
        let test = self.suspend_with_frame(test_result, {
            let args = args.clone();
            let value = value.clone();
            move |session, test| session.resume_confirm_by_after_test(args, value, test)
        })?;

        self.resume_confirm_by_after_test(args, value, test)
    }

    fn resume_confirm_by_after_test(
        &mut self,
        args: Vec<Expr>,
        value: Expr,
        test: Expr,
    ) -> Result<Expr, EvalError> {
        if is_system_symbol_name(&test, "True") {
            return Ok(value);
        }

        let raw_expr = args[0].clone();
        let raw_function = args[1].clone();
        let tag = confirmation_tag_from_args(&args, 3);
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
        let failure = build_confirmation_failure(vec![
            ("Information", info.unwrap_or_else(|| system_expr("Null"))),
            ("Expression", raw_expr),
            ("Function", raw_function),
        ]);

        Err(EvalError::Thrown {
            value: failure,
            tag,
        })
    }

    fn resume_confirm_match_after_value(
        &mut self,
        args: Vec<Expr>,
        value: Expr,
    ) -> Result<Expr, EvalError> {
        let raw_expr = args[0].clone();
        let raw_pattern = args[1].clone();
        let tag = confirmation_tag_from_args(&args, 3);
        let match_result =
            self.match_expr_owned(raw_pattern.clone(), value.clone(), BTreeMap::new());

        self.drive_internal_value(match_result, move |session, outcome| {
            if outcome.matched {
                Ok(value)
            } else {
                let info_result =
                    evaluate_optional_confirmation_info(session, args.get(2).cloned());
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
        let failure = build_confirmation_failure(vec![
            ("Information", info.unwrap_or_else(|| system_expr("Null"))),
            ("Expression", raw_expr),
            ("Pattern", raw_pattern),
        ]);

        Err(EvalError::Thrown {
            value: failure,
            tag,
        })
    }

    fn resume_confirm_quiet_after_body(
        &mut self,
        args: Vec<Expr>,
        selectors: ConfirmQuietSelectors,
        message_checkpoint: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let matched_message =
            confirm_quiet_matched_message(&self.messages[message_checkpoint..], &selectors);

        let Some(message) = matched_message else {
            return outcome;
        };

        suppress_confirm_quiet_messages(self, message_checkpoint, &selectors);

        let raw_expr = args[0].clone();
        let tag = confirmation_tag_from_args(&args, 3);
        let info_result = evaluate_optional_confirmation_info(self, args.get(2).cloned());
        self.drive_internal_value(info_result, move |session, info| {
            session.finish_confirm_quiet_failure(raw_expr, info, tag, message)
        })
    }

    fn finish_confirm_quiet_failure(
        &mut self,
        raw_expr: Expr,
        info: Option<Expr>,
        tag: Expr,
        message: RuntimeMessage,
    ) -> Result<Expr, EvalError> {
        let failure = build_confirmation_failure(vec![
            ("Information", info.unwrap_or_else(|| system_expr("Null"))),
            ("Expression", raw_expr),
            (
                "HeldMessage",
                Expr::call(
                    system_expr("Hold"),
                    vec![runtime_message_name_expr(message)],
                ),
            ),
        ]);

        Err(EvalError::Thrown {
            value: failure,
            tag,
        })
    }

    fn resume_confirm_assert_after_test(
        &mut self,
        args: Vec<Expr>,
        test: Expr,
    ) -> Result<Expr, EvalError> {
        if is_system_symbol_name(&test, "True") {
            return Ok(system_expr("Null"));
        }

        let raw_test = args[0].clone();
        let tag = confirmation_tag_from_args(&args, 2);
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
        let failure = build_confirmation_failure(vec![
            ("Information", info.unwrap_or_else(|| system_expr("Null"))),
            ("Test", raw_test),
        ]);

        Err(EvalError::Thrown {
            value: failure,
            tag,
        })
    }

    fn resume_catch_after_body(
        &mut self,
        form: Option<Expr>,
        handler: Option<Expr>,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => Ok(value),
            Err(EvalError::Thrown { value, tag }) => {
                self.resume_catch_for_throw(form, handler, value, tag)
            }
            Err(error) => Err(error),
        }
    }

    fn resume_catch_for_throw(
        &mut self,
        form: Option<Expr>,
        handler: Option<Expr>,
        value: Expr,
        tag: Expr,
    ) -> Result<Expr, EvalError> {
        let Some(form) = form else {
            return self.finish_catch_match(handler, value, tag);
        };

        let match_result = self.match_expr_owned(form, tag.clone(), BTreeMap::new());
        self.drive_internal_value(match_result, move |session, outcome| {
            if outcome.matched {
                session.finish_catch_match(handler, value, tag)
            } else {
                Err(EvalError::Thrown { value, tag })
            }
        })
    }

    fn finish_catch_match(
        &mut self,
        handler: Option<Expr>,
        value: Expr,
        tag: Expr,
    ) -> Result<Expr, EvalError> {
        match handler {
            Some(handler) => self.evaluate(Expr::call(handler, vec![value, tag])),
            None => Ok(value),
        }
    }
}

fn normalize_confirm_quiet_selectors(
    selector_expr: Option<&Expr>,
) -> Result<ConfirmQuietSelectors, MessageSelectorNormalizationError> {
    match selector_expr {
        None => Ok(ConfirmQuietSelectors::All),
        Some(expr) if is_system_symbol_name(expr, "All") => Ok(ConfirmQuietSelectors::All),
        Some(expr) => parse_message_selectors(expr)
            .map(ConfirmQuietSelectors::Exact)
            .ok_or(MessageSelectorNormalizationError::InvalidSelectorSpec),
    }
}

fn confirm_quiet_matched_message(
    messages: &[RuntimeMessage],
    selectors: &ConfirmQuietSelectors,
) -> Option<RuntimeMessage> {
    messages
        .iter()
        .find(|message| match selectors {
            ConfirmQuietSelectors::All => true,
            ConfirmQuietSelectors::Exact(selectors) => {
                selectors.iter().any(|selector| selector.matches(message))
            }
        })
        .cloned()
}

fn suppress_confirm_quiet_messages(
    session: &mut Session,
    checkpoint: usize,
    selectors: &ConfirmQuietSelectors,
) {
    match selectors {
        ConfirmQuietSelectors::All => session.messages.truncate(checkpoint),
        ConfirmQuietSelectors::Exact(selectors) => {
            session.suppress_messages_since(checkpoint, selectors);
        }
    }
}

fn evaluate_optional_confirmation_info(
    session: &mut Session,
    info_expr: Option<Expr>,
) -> Result<Option<Expr>, EvalError> {
    match info_expr {
        None => Ok(None),
        Some(expr) if is_internal_confirmation_auto_info(&expr) => Ok(None),
        Some(expr) => session.evaluate(expr).map(Some),
    }
}

fn confirmation_tag_from_args(args: &[Expr], index: usize) -> Expr {
    args.get(index)
        .cloned()
        .unwrap_or_else(default_confirmation_tag_expr)
}

fn default_confirmation_tag_expr() -> Expr {
    system_expr("InternalConfirmTag")
}

fn fresh_internal_enclose_tag_expr(session: &mut Session) -> Expr {
    let seed = Symbol::new(SYSTEM_CONTEXT, "InternalEncloseTag");
    Expr::Atom(Atom::Symbol(session.allocate_fresh_symbol(&seed)))
}

fn is_internal_confirmation_auto_info(expr: &Expr) -> bool {
    is_system_symbol_name(expr, "InternalConfirmAutomaticInfo")
}

fn build_confirmation_failure(fields: Vec<(&str, Expr)>) -> Expr {
    Expr::call(
        system_expr("Failure"),
        vec![
            Expr::string("ConfirmationFailed"),
            render_association(
                fields
                    .into_iter()
                    .map(|(key, value)| (Expr::string(key), value))
                    .collect(),
            ),
        ],
    )
}

pub(super) fn build_failsafe_failure(_function: Expr, _test: Expr, args: Vec<Expr>) -> Expr {
    let held_args = Expr::call(system_expr("Hold"), args.clone());
    let held_message_parameters = Expr::call(
        system_expr("List"),
        args.iter()
            .cloned()
            .map(|arg| Expr::call(system_expr("HoldForm"), vec![arg]))
            .collect(),
    );

    Expr::call(
        system_expr("Failure"),
        vec![
            Expr::string("FailsafeFailed"),
            render_association(vec![
                (
                    Expr::string("MessageTemplate"),
                    Expr::string("The test failed on the arguments `1`."),
                ),
                (Expr::string("MessageParameters"), held_message_parameters),
                (Expr::string("Arguments"), held_args),
            ]),
        ],
    )
}

pub(super) fn is_confirmation_failure_like(expr: &Expr) -> bool {
    is_failure_q_expr(expr)
        || is_system_symbol_name(expr, "$Canceled")
        || matches!(expr, Expr::Call { head, args } if is_system_symbol_name(head, "Missing") && !args.is_empty())
}

fn rewrite_untagged_confirms(expr: Expr, tag: &Expr) -> Expr {
    match expr {
        Expr::Call { head, args } if is_system_symbol_name(&head, "Enclose") => {
            Expr::call(head.as_ref().clone(), args)
        }
        Expr::Call { head, args } => {
            let rewritten_head = rewrite_untagged_confirms((*head).clone(), tag);
            let rewritten_args = args
                .into_iter()
                .map(|arg| rewrite_untagged_confirms(arg, tag))
                .collect::<Vec<_>>();

            let name = expr_as_symbol(&rewritten_head).map(|symbol| symbol.name().to_string());
            match (name.as_deref(), rewritten_args.as_slice()) {
                (Some("Confirm"), [expr]) => Expr::call(
                    rewritten_head,
                    vec![
                        expr.clone(),
                        system_expr("InternalConfirmAutomaticInfo"),
                        tag.clone(),
                    ],
                ),
                (Some("Confirm"), [expr, info]) => Expr::call(
                    rewritten_head,
                    vec![expr.clone(), info.clone(), tag.clone()],
                ),
                (Some("ConfirmBy"), [expr, function]) => Expr::call(
                    rewritten_head,
                    vec![
                        expr.clone(),
                        function.clone(),
                        system_expr("InternalConfirmAutomaticInfo"),
                        tag.clone(),
                    ],
                ),
                (Some("ConfirmBy"), [expr, function, info]) => Expr::call(
                    rewritten_head,
                    vec![expr.clone(), function.clone(), info.clone(), tag.clone()],
                ),
                (Some("ConfirmMatch"), [expr, pattern]) => Expr::call(
                    rewritten_head,
                    vec![
                        expr.clone(),
                        pattern.clone(),
                        system_expr("InternalConfirmAutomaticInfo"),
                        tag.clone(),
                    ],
                ),
                (Some("ConfirmMatch"), [expr, pattern, info]) => Expr::call(
                    rewritten_head,
                    vec![expr.clone(), pattern.clone(), info.clone(), tag.clone()],
                ),
                (Some("ConfirmQuiet"), [expr]) => Expr::call(
                    rewritten_head,
                    vec![
                        expr.clone(),
                        system_expr("All"),
                        system_expr("InternalConfirmAutomaticInfo"),
                        tag.clone(),
                    ],
                ),
                (Some("ConfirmQuiet"), [expr, selectors]) => Expr::call(
                    rewritten_head,
                    vec![
                        expr.clone(),
                        selectors.clone(),
                        system_expr("InternalConfirmAutomaticInfo"),
                        tag.clone(),
                    ],
                ),
                (Some("ConfirmQuiet"), [expr, selectors, info]) => Expr::call(
                    rewritten_head,
                    vec![expr.clone(), selectors.clone(), info.clone(), tag.clone()],
                ),
                (Some("ConfirmAssert"), [test]) => Expr::call(
                    rewritten_head,
                    vec![
                        test.clone(),
                        system_expr("InternalConfirmAutomaticInfo"),
                        tag.clone(),
                    ],
                ),
                (Some("ConfirmAssert"), [test, info]) => Expr::call(
                    rewritten_head,
                    vec![test.clone(), info.clone(), tag.clone()],
                ),
                _ => Expr::call(rewritten_head, rewritten_args),
            }
        }
        other => other,
    }
}
