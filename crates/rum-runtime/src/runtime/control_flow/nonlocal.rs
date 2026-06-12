use super::*;

impl Session {
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
                explicit_tag: false,
            }),
            [value, tag] => Err(EvalError::Thrown {
                value: value.clone(),
                tag: tag.clone(),
                explicit_tag: true,
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

    fn resume_catch_after_body(
        &mut self,
        form: Option<Expr>,
        handler: Option<Expr>,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => Ok(value),
            Err(EvalError::ThrownBypassingCatch {
                value,
                tag,
                explicit_tag,
                cleanup_value,
                cleanup_tag,
                cleanup_explicit_tag,
            }) => self.resume_catch_for_bypassing_throw(
                form,
                value,
                tag,
                explicit_tag,
                cleanup_value,
                cleanup_tag,
                cleanup_explicit_tag,
            ),
            Err(EvalError::Thrown {
                value,
                tag,
                explicit_tag,
            }) => self.resume_catch_for_throw(form, handler, value, tag, explicit_tag),
            Err(error) => Err(error),
        }
    }

    fn resume_catch_for_throw(
        &mut self,
        form: Option<Expr>,
        handler: Option<Expr>,
        value: Expr,
        tag: Expr,
        explicit_tag: bool,
    ) -> Result<Expr, EvalError> {
        let Some(form) = form else {
            return self.finish_catch_match(handler, value, tag);
        };

        let form_result = self.resolve_matcher_pattern_expr_owned(form);
        let form = self.suspend_with_frame(form_result, {
            let handler = handler.clone();
            let value = value.clone();
            let tag = tag.clone();
            move |session, form| {
                session.finish_catch_for_throw_with_form(form, handler, value, tag, explicit_tag)
            }
        })?;

        self.finish_catch_for_throw_with_form(form, handler, value, tag, explicit_tag)
    }

    fn finish_catch_for_throw_with_form(
        &mut self,
        form: Expr,
        handler: Option<Expr>,
        value: Expr,
        tag: Expr,
        explicit_tag: bool,
    ) -> Result<Expr, EvalError> {
        let match_result = self.match_expr_owned(form, tag.clone(), SymbolExprMap::default());
        self.drive_internal_value(match_result, move |session, outcome| {
            if outcome.matched {
                session.finish_catch_match(handler, value, tag)
            } else {
                Err(EvalError::Thrown {
                    value,
                    tag,
                    explicit_tag,
                })
            }
        })
    }

    fn resume_catch_for_bypassing_throw(
        &mut self,
        form: Option<Expr>,
        value: Expr,
        tag: Expr,
        explicit_tag: bool,
        cleanup_value: Expr,
        cleanup_tag: Expr,
        cleanup_explicit_tag: bool,
    ) -> Result<Expr, EvalError> {
        let Some(form) = form else {
            return Err(EvalError::Thrown {
                value,
                tag,
                explicit_tag,
            });
        };

        let form_result = self.resolve_matcher_pattern_expr_owned(form);
        let form = self.suspend_with_frame(form_result, {
            let value = value.clone();
            let tag = tag.clone();
            let cleanup_value = cleanup_value.clone();
            let cleanup_tag = cleanup_tag.clone();
            move |session, form| {
                session.finish_catch_for_bypassing_throw_with_form(
                    form,
                    value,
                    tag,
                    explicit_tag,
                    cleanup_value,
                    cleanup_tag,
                    cleanup_explicit_tag,
                )
            }
        })?;

        self.finish_catch_for_bypassing_throw_with_form(
            form,
            value,
            tag,
            explicit_tag,
            cleanup_value,
            cleanup_tag,
            cleanup_explicit_tag,
        )
    }

    fn finish_catch_for_bypassing_throw_with_form(
        &mut self,
        form: Expr,
        value: Expr,
        tag: Expr,
        explicit_tag: bool,
        cleanup_value: Expr,
        cleanup_tag: Expr,
        cleanup_explicit_tag: bool,
    ) -> Result<Expr, EvalError> {
        let match_result = self.match_expr_owned(form, tag.clone(), SymbolExprMap::default());
        self.drive_internal_value(match_result, move |_, outcome| {
            if outcome.matched {
                Err(EvalError::Thrown {
                    value,
                    tag,
                    explicit_tag,
                })
            } else {
                Err(EvalError::ThrownBypassingCatch {
                    value,
                    tag,
                    explicit_tag,
                    cleanup_value,
                    cleanup_tag,
                    cleanup_explicit_tag,
                })
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
