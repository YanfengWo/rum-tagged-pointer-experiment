use super::*;

mod branching_loops;
mod confirmation;
mod status;

use self::confirmation::{build_failsafe_failure, is_confirmation_failure_like};
pub(super) use self::status::{apply_status_object_to_args, is_failure_expr, is_success_expr};
use self::status::{failure_property_value, is_failure_q_expr};

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

    pub(super) fn eval_check_abort(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(2..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("CheckAbort", args, 2, 3));
        }

        let propagate_expr = args.get(2).cloned().or_else(|| {
            self.option_rule_expr_for_symbol(
                &Symbol::system("CheckAbort"),
                &Symbol::system("PropagateAborts"),
            )
        });
        let propagate = match parse_check_abort_propagation(propagate_expr.as_ref()) {
            Ok(mode) => mode,
            Err(detail) => return Ok(self.invalid_call_with_arg_form("CheckAbort", args, detail)),
        };
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
                self.resume_with_cleanup_after_primary(cleanup.clone(), primary_result)
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
        if !(1..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Failsafe", args, 1, 3));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }

    pub(super) fn eval_assert(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [test] => {
                if !self.asserts_enabled {
                    return Ok(system_expr("Null"));
                }

                let test_result = self.evaluate(test.clone());
                let outcome = self.suspend_with_result_frame(test_result, {
                    let test = test.clone();
                    move |session, outcome| session.resume_assert_after_test(test, None, outcome)
                });

                match outcome {
                    Err(EvalError::InputRequired { request }) => {
                        Err(EvalError::InputRequired { request })
                    }
                    outcome => self.resume_assert_after_test(test.clone(), None, outcome),
                }
            }
            [test, tag] => {
                if !self.asserts_enabled {
                    return Ok(system_expr("Null"));
                }

                let test_result = self.evaluate(test.clone());
                let outcome = self.suspend_with_result_frame(test_result, {
                    let test = test.clone();
                    let tag = tag.clone();
                    move |session, outcome| {
                        session.resume_assert_after_test(test, Some(tag), outcome)
                    }
                });

                match outcome {
                    Err(EvalError::InputRequired { request }) => {
                        Err(EvalError::InputRequired { request })
                    }
                    outcome => {
                        self.resume_assert_after_test(test.clone(), Some(tag.clone()), outcome)
                    }
                }
            }
            _ => Ok(self.invalid_call_with_arg_range("Assert", args, 1, 2)),
        }
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
        tag: Option<Expr>,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) if is_system_symbol_name(&value, "True") => Ok(system_expr("Null")),
            Ok(value) => {
                self.push_message(
                    "Assert",
                    "fail",
                    render_assert_failure_message(&test, tag.as_ref(), &value),
                );
                Ok(system_expr("Null"))
            }
            Err(error) => Err(error),
        }
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
                self.resume_with_cleanup_after_primary(cleanup, primary_result)
            }
            Err(error) => self.resume_with_cleanup_after_primary(cleanup, Err(error)),
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
        match cleanup_outcome {
            Ok(_) => primary_outcome,
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
}

fn is_failsafe_failure_like(expr: &Expr) -> bool {
    is_confirmation_failure_like(expr)
}

fn render_assert_failure_message(test: &Expr, tag: Option<&Expr>, value: &Expr) -> String {
    match tag {
        Some(tag) => format!("assertion `{tag}` failed: {test} evaluated to {value}"),
        None => format!("assertion failed: {test} evaluated to {value}"),
    }
}

fn parse_check_abort_propagation(
    option_expr: Option<&Expr>,
) -> Result<PropagateAbortsMode, &'static str> {
    let Some(option_expr) = option_expr else {
        return Ok(PropagateAbortsMode::Automatic);
    };

    let Expr::Call { head, args } = option_expr else {
        return Err(
            "expected PropagateAborts -> True|False|Automatic as the optional third argument",
        );
    };

    if !is_system_symbol_name(head, "Rule") || args.len() != 2 {
        return Err(
            "expected PropagateAborts -> True|False|Automatic as the optional third argument",
        );
    }

    if !is_system_symbol_name(&args[0], "PropagateAborts") {
        return Err(
            "expected PropagateAborts -> True|False|Automatic as the optional third argument",
        );
    }

    if is_system_symbol_name(&args[1], "True") {
        Ok(PropagateAbortsMode::True)
    } else if is_system_symbol_name(&args[1], "False") {
        Ok(PropagateAbortsMode::False)
    } else if is_system_symbol_name(&args[1], "Automatic") {
        Ok(PropagateAbortsMode::Automatic)
    } else {
        Err("expected PropagateAborts -> True|False|Automatic as the optional third argument")
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
        vec![Expr::call(test, args.to_vec()), success, fallback],
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
