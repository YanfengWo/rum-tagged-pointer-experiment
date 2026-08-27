use super::*;

impl Session {
    pub(in crate::runtime) fn eval_with_cleanup(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [expr, cleanup] => {
                let primary_result = self.evaluate(expr.clone());
                self.drive_with_cleanup_primary(cleanup.clone(), primary_result)
            }
            [init, expr, cleanup] => {
                let init_result = self.evaluate(init.clone());
                let outcome = self.suspend_with_result_frame_lazy(init_result, || {
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
        let outcome = self.suspend_with_result_frame_lazy(primary_result, || {
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

        if matches!(
            primary_outcome,
            Err(EvalError::Abort | EvalError::ProtectedAbort)
        ) && matches!(
            cleanup_outcome,
            Err(EvalError::Thrown { .. } | EvalError::ThrownBypassingCatch { .. })
        ) {
            return primary_outcome;
        }

        match cleanup_outcome {
            Ok(_) | Err(EvalError::Return { .. }) => primary_outcome,
            Err(error) => Err(error),
        }
    }
}
