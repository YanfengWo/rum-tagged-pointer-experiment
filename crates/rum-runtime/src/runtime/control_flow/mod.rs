use super::*;

mod branching_loops;
mod confirmation;
mod status;

use self::confirmation::{build_failsafe_failure, is_confirmation_failure_like};
use self::status::is_failure_q_expr;
pub(super) use self::status::{is_failure_expr, is_success_expr};

#[derive(Clone, Debug, PartialEq, Eq)]
enum ConfirmQuietSelectors {
    All,
    Exact(Vec<MessageSelector>),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PropagateAbortsMode {
    Automatic,
    False,
    True,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CheckAbortPropagation {
    mode: PropagateAbortsMode,
    invalid_value: Option<Expr>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct CheckAbortPropagationSource {
    value: Expr,
    evaluate_rhs: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CheckAbortOptionError {
    NonOption(Expr),
    InvalidOptionName(Expr),
    UnknownOption(Symbol),
}

impl Session {
    pub(super) fn eval_check(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(2..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_message(
                "Check",
                args,
                "argx",
                format!("expected 2 or 3 argument(s), got {}", args.len()),
            ));
        }

        let selectors = match self.normalize_optional_message_selectors(args.get(2)) {
            Ok(selectors) => selectors,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("Check", args, error.message()));
            }
        };

        let fallback = args[1].clone();
        let message_checkpoint = self.messages.len();
        let handled_error_checkpoint = self.handled_non_loader_eval_errors.len();
        let primary_result = self.evaluate(args[0].clone());
        let outcome = self.suspend_with_result_frame(primary_result, {
            let selectors = selectors.clone();
            let fallback = fallback.clone();
            move |session, outcome| {
                session.resume_check_after_primary(
                    selectors,
                    fallback,
                    message_checkpoint,
                    handled_error_checkpoint,
                    outcome,
                )
            }
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_check_after_primary(
                selectors,
                fallback,
                message_checkpoint,
                handled_error_checkpoint,
                outcome,
            ),
        }
    }

    pub(super) fn eval_quiet(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_message(
                "Quiet",
                args,
                "argx",
                format!("expected 1 or 2 argument(s), got {}", args.len()),
            ));
        }

        let selectors = match self.normalize_optional_message_selectors(args.get(1)) {
            Ok(selectors) => selectors,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form("Quiet", args, error.message()));
            }
        };

        let message_checkpoint = self.messages.len();
        let handled_error_checkpoint = self.handled_non_loader_eval_errors.len();
        let primary_result = self.evaluate(args[0].clone());
        let result = self.suspend_with_result_frame(primary_result, {
            let selectors = selectors.clone();
            move |session, outcome| {
                session.resume_quiet_after_primary(
                    selectors,
                    message_checkpoint,
                    handled_error_checkpoint,
                    outcome,
                )
            }
        });

        match result {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_quiet_after_primary(
                selectors,
                message_checkpoint,
                handled_error_checkpoint,
                outcome,
            ),
        }
    }

    pub(super) fn eval_abort(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if !args.is_empty() {
            return Ok(self.invalid_call_with_exact_arg_count("Abort", args, 0));
        }

        if self.abort_protect_depth > 0 {
            self.abort_protect_pending += 1;
            return Ok(system_expr("$Aborted"));
        }

        Err(EvalError::Abort)
    }

    pub(super) fn eval_exit(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let code = match args {
            [] => 0,
            [code] => {
                let Some(code) = expr_as_signed_isize(code) else {
                    let call = self.invalid_call("Exit", args);
                    return Ok(self.invalid_call_with_message(
                        "Exit",
                        args,
                        "intm",
                        format!("Machine-sized integer expected at position 1 in {call}."),
                    ));
                };
                code as i64
            }
            _ => {
                return Ok(self.invalid_call_with_message(
                    "Exit",
                    args,
                    "argt",
                    format!(
                        "Exit called with {} arguments; 0 or 1 arguments are expected.",
                        args.len()
                    ),
                ));
            }
        };

        Err(EvalError::Exit { code })
    }

    pub(super) fn eval_check_abort(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() < 2 {
            let tag = if args.is_empty() { "argrx" } else { "argr" };
            return Ok(self.invalid_call_with_message(
                "CheckAbort",
                args,
                tag,
                format!("expected at least 2 argument(s), got {}", args.len()),
            ));
        }

        let default_option = self.option_rule_expr_for_symbol(
            &Symbol::system("CheckAbort"),
            &Symbol::system("PropagateAborts"),
        )?;
        let default_option_exprs: Vec<Expr> = default_option.into_iter().collect();
        let default_propagation =
            match parse_check_abort_propagation_source(&default_option_exprs, false) {
                Ok(Some(source)) => check_abort_propagation_from_value(source.value),
                Ok(None) => default_check_abort_propagation(),
                Err(error) => return Ok(self.invalid_check_abort_option_call(args, error)),
            };

        let propagation_source = match parse_check_abort_propagation_source(&args[2..], true) {
            Ok(source) => source,
            Err(error) => return Ok(self.invalid_check_abort_option_call(args, error)),
        };
        let Some(source) = propagation_source else {
            return self.resume_check_abort_with_propagation(args.to_vec(), default_propagation);
        };
        if source.evaluate_rhs {
            let value_result = self.evaluate(source.value.clone());
            let value = self.suspend_with_frame(value_result, {
                let args = args.to_vec();
                move |session, value| {
                    session.resume_check_abort_with_propagation(
                        args,
                        check_abort_propagation_from_value(value),
                    )
                }
            })?;

            return self.resume_check_abort_with_propagation(
                args.to_vec(),
                check_abort_propagation_from_value(value),
            );
        }

        self.resume_check_abort_with_propagation(
            args.to_vec(),
            check_abort_propagation_from_value(source.value),
        )
    }

    fn resume_check_abort_with_propagation(
        &mut self,
        args: Vec<Expr>,
        propagation: CheckAbortPropagation,
    ) -> Result<Expr, EvalError> {
        if let Some(invalid_value) = propagation.invalid_value.as_ref() {
            self.push_message(
                "CheckAbort",
                "opttfa",
                format!(
                    "The value of the option PropagateAborts -> {invalid_value} should be True, False or Automatic."
                ),
            );
        }
        let propagate = propagation.mode;
        let pending_checkpoint = self.abort_protect_pending;

        let primary_result = self.evaluate(args[0].clone());
        let outcome = self.suspend_with_result_frame(primary_result, {
            let args = args.to_vec();
            move |session, outcome| {
                session.resume_check_abort_after_primary(
                    args,
                    propagate,
                    pending_checkpoint,
                    outcome,
                )
            }
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_check_abort_after_primary(
                args.to_vec(),
                propagate,
                pending_checkpoint,
                outcome,
            ),
        }
    }

    pub(super) fn eval_abort_protect(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [body] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("AbortProtect", args, 1));
        };

        let pending_checkpoint = self.abort_protect_pending;
        self.abort_protect_depth += 1;
        let body_result = self.evaluate(body.clone());
        let outcome = self.suspend_with_result_frame(body_result, move |session, outcome| {
            session.resume_abort_protect_after_body(pending_checkpoint, outcome)
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_abort_protect_after_body(pending_checkpoint, outcome),
        }
    }

    pub(super) fn eval_with_cleanup(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [expr, cleanup] => {
                let primary_result = self.evaluate(expr.clone());
                self.drive_with_cleanup_primary(cleanup.clone(), primary_result)
            }
            [init, expr, cleanup] => {
                let init_result = self.evaluate(init.clone());
                let outcome = self.suspend_with_result_frame(init_result, {
                    let expr = expr.clone();
                    let cleanup = cleanup.clone();
                    move |session, outcome| {
                        session.resume_with_cleanup_after_init(expr, cleanup, outcome)
                    }
                });

                match outcome {
                    Err(EvalError::InputRequired { request }) => {
                        Err(EvalError::InputRequired { request })
                    }
                    outcome => {
                        self.resume_with_cleanup_after_init(expr.clone(), cleanup.clone(), outcome)
                    }
                }
            }
            _ => Ok(self.invalid_call_with_arg_range("WithCleanup", args, 2, 3)),
        }
    }

    pub(super) fn eval_failsafe(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(super) fn eval_assert(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let assert_args = args.to_vec();
        let test = args.first().cloned();

        let function_result = self.evaluate(system_expr("$AssertFunction"));
        let function = self.suspend_with_frame(function_result, {
            let test = test.clone();
            let assert_args = assert_args.clone();
            move |session, function| {
                session.resume_assert_after_function(test, assert_args, function)
            }
        })?;

        self.resume_assert_after_function(test, assert_args, function)
    }

    pub(super) fn eval_block_random(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(1..=2).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("BlockRandom", args, 1, 2));
        }

        let body_result = self.evaluate(args[0].clone());
        self.suspend_with_frame(body_result, |_, value| Ok(value))
    }

    fn resume_check_after_primary(
        &mut self,
        selectors: Option<Vec<MessageSelector>>,
        fallback: Expr,
        message_checkpoint: usize,
        handled_error_checkpoint: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => {
                let matched = selectors.as_ref().map_or_else(
                    || self.messages.len() > message_checkpoint,
                    |selectors| self.messages_since_match(message_checkpoint, selectors),
                );

                self.handled_non_loader_eval_errors
                    .truncate(handled_error_checkpoint);

                if matched {
                    self.evaluate(fallback)
                } else {
                    Ok(value)
                }
            }
            Err(err) => {
                self.emit_non_loader_eval_message(handled_error_checkpoint, &err);

                let matched = selectors.as_ref().map_or_else(
                    || self.messages.len() > message_checkpoint,
                    |selectors| self.messages_since_match(message_checkpoint, selectors),
                );

                if matched {
                    self.evaluate(fallback)
                } else {
                    Err(err)
                }
            }
        }
    }

    fn resume_check_abort_after_primary(
        &mut self,
        args: Vec<Expr>,
        propagate: PropagateAbortsMode,
        pending_checkpoint: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(_value) if self.abort_protect_pending > pending_checkpoint => {
                let fallback_result = self.evaluate(args[1].clone());
                let outcome = self.suspend_with_result_frame(fallback_result, {
                    move |session, outcome| {
                        session.resume_check_abort_after_fallback(
                            propagate,
                            true,
                            pending_checkpoint,
                            outcome,
                        )
                    }
                });

                match outcome {
                    Err(EvalError::InputRequired { request }) => {
                        Err(EvalError::InputRequired { request })
                    }
                    outcome => self.resume_check_abort_after_fallback(
                        propagate,
                        true,
                        pending_checkpoint,
                        outcome,
                    ),
                }
            }
            Ok(value) => Ok(value),
            Err(error @ (EvalError::Abort | EvalError::ProtectedAbort)) => {
                let protected = matches!(error, EvalError::ProtectedAbort);
                let fallback_result = self.evaluate(args[1].clone());
                let outcome = self.suspend_with_result_frame(fallback_result, {
                    move |session, outcome| {
                        session.resume_check_abort_after_fallback(
                            propagate,
                            protected,
                            pending_checkpoint,
                            outcome,
                        )
                    }
                });

                match outcome {
                    Err(EvalError::InputRequired { request }) => {
                        Err(EvalError::InputRequired { request })
                    }
                    outcome => self.resume_check_abort_after_fallback(
                        propagate,
                        protected,
                        pending_checkpoint,
                        outcome,
                    ),
                }
            }
            Err(error) => Err(error),
        }
    }

    fn resume_check_abort_after_fallback(
        &mut self,
        propagate: PropagateAbortsMode,
        protected: bool,
        pending_checkpoint: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => {
                if protected && propagate != PropagateAbortsMode::True {
                    self.abort_protect_pending = pending_checkpoint;
                }

                if propagate == PropagateAbortsMode::True {
                    if self.abort_protect_depth > 0 {
                        if self.abort_protect_pending <= pending_checkpoint {
                            self.abort_protect_pending = pending_checkpoint.saturating_add(1);
                        }
                        return Ok(system_expr("$Aborted"));
                    }

                    Err(if protected {
                        EvalError::ProtectedAbort
                    } else {
                        EvalError::Abort
                    })
                } else {
                    Ok(value)
                }
            }
            Err(error) => {
                if protected {
                    self.abort_protect_pending = pending_checkpoint;
                }

                Err(error)
            }
        }
    }

    fn resume_abort_protect_after_body(
        &mut self,
        pending_checkpoint: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        if self.abort_protect_depth > 0 {
            self.abort_protect_depth -= 1;
        }

        let delayed_abort = self.abort_protect_pending > pending_checkpoint
            || matches!(outcome, Err(EvalError::Abort | EvalError::ProtectedAbort));
        self.abort_protect_pending = pending_checkpoint;

        if delayed_abort && self.abort_protect_depth > 0 {
            self.abort_protect_pending = pending_checkpoint.saturating_add(1);
            return match outcome {
                Ok(value) => Ok(value),
                Err(EvalError::Abort | EvalError::ProtectedAbort) => Ok(system_expr("$Aborted")),
                Err(error) => Err(error),
            };
        }

        match outcome {
            Ok(value) => {
                if delayed_abort {
                    Err(EvalError::Abort)
                } else {
                    Ok(value)
                }
            }
            Err(EvalError::ProtectedAbort) => Err(EvalError::Abort),
            Err(EvalError::Abort) => Err(EvalError::Abort),
            Err(error) => Err(error),
        }
    }

    fn resume_assert_after_test(
        &mut self,
        test: Expr,
        assert_args: Vec<Expr>,
        assert_function: Option<Expr>,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) if is_system_symbol_name(&value, "True") => Ok(system_expr("Null")),
            Ok(value) => {
                for (message_tag, text) in assert_test_value_messages(&test, &value) {
                    self.push_message("Assert", message_tag, text);
                }

                if let Some(assert_function) = assert_function {
                    return self.finish_assert_function(assert_function, assert_args);
                }

                for (message_tag, text) in assert_failure_messages(&test, &assert_args) {
                    self.push_message("Assert", message_tag, text);
                }
                Ok(system_expr("Null"))
            }
            Err(error) => Err(error),
        }
    }

    fn resume_assert_after_function(
        &mut self,
        test: Option<Expr>,
        assert_args: Vec<Expr>,
        function: Expr,
    ) -> Result<Expr, EvalError> {
        let assert_function = if is_system_symbol_name(&function, "$AssertFunction")
            || is_system_symbol_name(&function, "None")
        {
            None
        } else {
            Some(function)
        };
        if !self.asserts_enabled && assert_function.is_none() {
            return Ok(Expr::call(system_expr("Assert"), assert_args));
        }
        let Some(test) = test else {
            self.push_message(
                "Assert",
                "asrte",
                format!(
                    "Assertion {} had no test.",
                    render_input_like_expr(&Expr::call(system_expr("Assert"), assert_args))
                ),
            );
            return Ok(system_expr("Null"));
        };

        let test_result = self.evaluate(test.clone());
        let outcome = self.suspend_with_result_frame(test_result, {
            let test = test.clone();
            let assert_args = assert_args.clone();
            let assert_function = assert_function.clone();
            move |session, outcome| {
                session.resume_assert_after_test(test, assert_args, assert_function, outcome)
            }
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_assert_after_test(test, assert_args, assert_function, outcome),
        }
    }

    fn finish_assert_function(
        &mut self,
        assert_function: Expr,
        assert_args: Vec<Expr>,
    ) -> Result<Expr, EvalError> {
        let payload = Expr::call(
            system_expr("HoldComplete"),
            vec![Expr::call(system_expr("Assert"), assert_args)],
        );
        let result = self.evaluate(Expr::call(assert_function, vec![payload]));
        self.drive_internal_value(result, |_, _| Ok(system_expr("Null")))
    }

    fn resume_with_cleanup_after_init(
        &mut self,
        expr: Expr,
        cleanup: Expr,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(_) => {
                let primary_result = self.evaluate(expr);
                self.drive_with_cleanup_primary(cleanup, primary_result)
            }
            Err(error) => self.resume_with_cleanup_after_primary(cleanup, Err(error)),
        }
    }

    fn drive_with_cleanup_primary(
        &mut self,
        cleanup: Expr,
        primary_result: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let outcome = self.suspend_with_result_frame(primary_result, {
            let cleanup = cleanup.clone();
            move |session, primary_outcome| {
                session.resume_with_cleanup_after_primary(cleanup, primary_outcome)
            }
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.resume_with_cleanup_after_primary(cleanup, outcome),
        }
    }

    fn resume_with_cleanup_after_primary(
        &mut self,
        cleanup: Expr,
        primary_outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let cleanup_result = self.evaluate(cleanup.clone());
        match cleanup_result {
            Ok(value) => self.resume_with_cleanup_after_cleanup(primary_outcome, Ok(value)),
            Err(EvalError::InputRequired { request }) => {
                self.push_suspension_frame(move |session, cleanup_outcome| {
                    session.resume_with_cleanup_after_cleanup(primary_outcome, cleanup_outcome)
                });
                Err(EvalError::InputRequired { request })
            }
            Err(error) => self.resume_with_cleanup_after_cleanup(primary_outcome, Err(error)),
        }
    }

    fn resume_with_cleanup_after_cleanup(
        &mut self,
        primary_outcome: Result<Expr, EvalError>,
        cleanup_outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        if let (
            Err(EvalError::Thrown {
                value,
                tag,
                explicit_tag,
            }),
            Err(EvalError::Thrown {
                value: cleanup_value,
                tag: cleanup_tag,
                explicit_tag: cleanup_explicit_tag,
            }),
        ) = (&primary_outcome, &cleanup_outcome)
        {
            if tag == cleanup_tag {
                return Err(EvalError::ThrownBypassingCatch {
                    value: value.clone(),
                    tag: tag.clone(),
                    explicit_tag: *explicit_tag,
                    cleanup_value: cleanup_value.clone(),
                    cleanup_tag: cleanup_tag.clone(),
                    cleanup_explicit_tag: *cleanup_explicit_tag,
                });
            }
        }

        match cleanup_outcome {
            Ok(_) | Err(EvalError::Return { .. }) => primary_outcome,
            Err(error) => Err(error),
        }
    }

    fn resume_quiet_after_primary(
        &mut self,
        selectors: Option<Vec<MessageSelector>>,
        message_checkpoint: usize,
        handled_error_checkpoint: usize,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        let outcome = match outcome {
            Ok(value) => Ok(value),
            Err(err) => {
                self.emit_non_loader_eval_message(handled_error_checkpoint, &err);
                Err(err)
            }
        };

        if let Some(selectors) = selectors.as_ref() {
            self.suppress_messages_since(message_checkpoint, selectors);
        } else {
            self.messages.truncate(message_checkpoint);
        }

        if outcome.is_ok() {
            self.handled_non_loader_eval_errors
                .truncate(handled_error_checkpoint);
        }

        outcome
    }

    fn invalid_check_abort_option_call(
        &mut self,
        args: &[Expr],
        error: CheckAbortOptionError,
    ) -> Expr {
        let held_call = Expr::call(system_expr("CheckAbort"), args.to_vec());
        match error {
            CheckAbortOptionError::NonOption(offender) => self.invalid_call_with_message(
                "CheckAbort",
                args,
                "nonopt",
                format!(
                    "Options expected (instead of {offender}) beyond position 2 in {held_call}. An option must be a rule or a list of rules."
                ),
            ),
            CheckAbortOptionError::InvalidOptionName(offender) => self.invalid_call_with_message(
                "CheckAbort",
                args,
                "optrs",
                format!(
                    "Option specification {offender} in {held_call} is not a rule for a symbol or string."
                ),
            ),
            CheckAbortOptionError::UnknownOption(option_name) => self.invalid_call_with_message(
                "CheckAbort",
                args,
                "optx",
                format!("Unknown option {option_name} in {held_call}."),
            ),
        }
    }
}

fn is_failsafe_failure_like(expr: &Expr) -> bool {
    is_confirmation_failure_like(expr)
}

fn assert_test_value_messages(test: &Expr, value: &Expr) -> Vec<(&'static str, String)> {
    let mut messages = Vec::new();
    let test_text = render_input_like_expr(test);

    if !is_system_symbol_name(value, "False") {
        messages.push((
            "asrttf",
            format!(
                "Assertion test {test_text} evaluated to {} that is neither True nor False.",
                render_input_like_expr(value)
            ),
        ));
    }

    messages
}

fn assert_failure_messages(test: &Expr, assert_args: &[Expr]) -> Vec<(&'static str, String)> {
    let test_text = render_input_like_expr(test);
    vec![match assert_args {
        [_] => ("asrtf", format!("Assertion {test_text} failed.")),
        _ => (
            "asrtfe",
            format!(
                "Assertion {test_text} in {} failed.",
                render_input_like_expr(&Expr::call(system_expr("Assert"), assert_args.to_vec()))
            ),
        ),
    }]
}

fn parse_check_abort_propagation_source(
    option_exprs: &[Expr],
    evaluate_rule_rhs: bool,
) -> Result<Option<CheckAbortPropagationSource>, CheckAbortOptionError> {
    let mut source = None;
    for option_expr in option_exprs {
        parse_check_abort_propagation_source_expr(option_expr, evaluate_rule_rhs, &mut source)?;
    }

    Ok(source)
}

fn default_check_abort_propagation() -> CheckAbortPropagation {
    CheckAbortPropagation {
        mode: PropagateAbortsMode::Automatic,
        invalid_value: None,
    }
}

fn parse_check_abort_propagation_source_expr(
    option_expr: &Expr,
    evaluate_rule_rhs: bool,
    source: &mut Option<CheckAbortPropagationSource>,
) -> Result<(), CheckAbortOptionError> {
    let Expr::Call { head, args } = option_expr else {
        return Err(CheckAbortOptionError::NonOption(option_expr.clone()));
    };

    if is_system_symbol_name(head, heads::structure::LIST) {
        for item in args {
            if let Err(error) =
                parse_check_abort_propagation_source_expr(item, evaluate_rule_rhs, source)
            {
                return Err(match error {
                    CheckAbortOptionError::NonOption(_) => {
                        CheckAbortOptionError::NonOption(option_expr.clone())
                    }
                    other => other,
                });
            }
        }
        return Ok(());
    }

    if !(is_system_symbol_name(head, "Rule") || is_system_symbol_name(head, "RuleDelayed"))
        || args.len() != 2
    {
        return Err(CheckAbortOptionError::NonOption(option_expr.clone()));
    }

    let option_name = check_abort_option_name(&args[0])
        .ok_or_else(|| CheckAbortOptionError::InvalidOptionName(option_expr.clone()))?;
    if option_name.name() != "PropagateAborts" {
        return Err(CheckAbortOptionError::UnknownOption(option_name));
    }

    if source.is_none() {
        *source = Some(CheckAbortPropagationSource {
            value: args[1].clone(),
            evaluate_rhs: evaluate_rule_rhs && is_system_symbol_name(head, "Rule"),
        });
    }

    Ok(())
}

fn check_abort_propagation_from_value(value: Expr) -> CheckAbortPropagation {
    let (mode, invalid_value) = if is_system_symbol_name(&value, "True") {
        (PropagateAbortsMode::True, None)
    } else if is_system_symbol_name(&value, "False") {
        (PropagateAbortsMode::False, None)
    } else if is_system_symbol_name(&value, "Automatic") {
        (PropagateAbortsMode::Automatic, None)
    } else {
        (PropagateAbortsMode::Automatic, Some(value))
    };

    CheckAbortPropagation {
        mode,
        invalid_value,
    }
}

fn check_abort_option_name(expr: &Expr) -> Option<Symbol> {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => Some(symbol.clone()),
        Expr::Atom(Atom::String(name)) => resolve_unqualified_symbol(name),
        _ => None,
    }
}

pub(super) fn apply_callable_failsafe_wrapper_to_args(head: &Expr, args: &[Expr]) -> Option<Expr> {
    let spec = parse_failsafe_wrapper(head)?;

    if spec.test.is_none() {
        return Some(
            args.iter()
                .find(|arg| is_failsafe_failure_like(arg))
                .cloned()
                .unwrap_or_else(|| Expr::call(spec.function.clone(), args.to_vec())),
        );
    }

    let test = spec.test?.clone();
    let success = Expr::call(spec.function.clone(), args.to_vec());
    let fallback = match spec.fail_function {
        Some(fail_function) => Expr::call(fail_function.clone(), args.to_vec()),
        None => build_failsafe_failure(spec.function.clone(), test.clone(), args.to_vec()),
    };

    Some(Expr::call(
        system_expr("If"),
        vec![
            Expr::call(system_expr("TrueQ"), vec![Expr::call(test, args.to_vec())]),
            success,
            fallback,
        ],
    ))
}

pub(super) fn is_callable_failsafe_wrapper(expr: &Expr) -> bool {
    parse_failsafe_wrapper(expr).is_some()
}

struct FailsafeWrapperSpec<'a> {
    function: &'a Expr,
    test: Option<&'a Expr>,
    fail_function: Option<&'a Expr>,
}

fn parse_failsafe_wrapper(expr: &Expr) -> Option<FailsafeWrapperSpec<'_>> {
    let Expr::Call { head, args } = expr else {
        return None;
    };

    if !is_system_symbol_name(head, "Failsafe") {
        return None;
    }

    match args.as_slice() {
        [function] => Some(FailsafeWrapperSpec {
            function,
            test: None,
            fail_function: None,
        }),
        [function, test] => Some(FailsafeWrapperSpec {
            function,
            test: Some(test),
            fail_function: None,
        }),
        [function, test, fail_function] => Some(FailsafeWrapperSpec {
            function,
            test: Some(test),
            fail_function: Some(fail_function),
        }),
        _ => None,
    }
}
