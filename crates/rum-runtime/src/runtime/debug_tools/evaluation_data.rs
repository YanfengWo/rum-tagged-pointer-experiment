use super::*;

impl Session {
    pub(in crate::runtime) fn eval_evaluation_data(
        &mut self,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let (expr, properties) = match args {
            [expr] => (expr, None),
            [expr, properties] => (expr, Some(properties)),
            _ => return Ok(self.invalid_call_with_arg_range("EvaluationData", args, 1, 2)),
        };

        let checkpoint = DebugCaptureCheckpoint::new(self);
        let prior_tracing = self.tracing_enabled;
        self.tracing_enabled = true;
        let started = Instant::now();
        let input_string = expr.to_string();
        let input_expr = expr.clone();

        let body_result = self.evaluate(expr.clone());
        let outcome = self.suspend_with_result_frame_lazy(body_result, || {
            let input_string = input_string.clone();
            let input_expr = input_expr.clone();
            move |session, outcome| {
                session.finish_evaluation_data_capture(
                    checkpoint,
                    prior_tracing,
                    started,
                    &input_expr,
                    &input_string,
                    outcome,
                )
            }
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => {
                let data = self.finish_evaluation_data_capture(
                    checkpoint,
                    prior_tracing,
                    started,
                    &input_expr,
                    &input_string,
                    outcome,
                )?;
                self.project_evaluation_data_properties(data, properties.cloned())
            }
        }
    }

    fn finish_evaluation_data_capture(
        &mut self,
        checkpoint: DebugCaptureCheckpoint,
        prior_tracing: bool,
        started: Instant,
        input_expr: &Expr,
        input_string: &str,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        self.tracing_enabled = prior_tracing;

        if let Err(error) = &outcome {
            if Self::terminated_evaluation_expr_for_error(error).is_some() {
                self.emit_non_loader_eval_message(checkpoint.handled_error_checkpoint, error);
            }
            self.emit_top_level_control_flow_messages(error);
        }

        let messages = self.messages.split_off(checkpoint.message_checkpoint);
        let prints = self.prints.split_off(checkpoint.print_checkpoint);
        let events = self.trace_events[checkpoint.trace_checkpoint..].to_vec();
        let depths = self.trace_event_depths[checkpoint.trace_checkpoint..].to_vec();
        let trace_root = self.structured_trace_expr_from_events(input_expr, &events, &depths);

        if !prior_tracing {
            self.truncate_trace_from(checkpoint.trace_checkpoint);
        }
        self.handled_non_loader_eval_errors
            .truncate(checkpoint.handled_error_checkpoint);

        let elapsed = elapsed_duration_expr(started.elapsed());
        let (result_expr, success, failure_type) = match outcome {
            Ok(value) => (value, true, system_expr("None")),
            Err(error) if Self::terminated_evaluation_expr_for_error(&error).is_some() => (
                Self::terminated_evaluation_expr_for_error(&error)
                    .expect("termination value should exist after guard"),
                false,
                Expr::string(
                    Self::terminated_evaluation_reason_for_error(&error)
                        .expect("termination reason should exist after guard")
                        .tag(),
                ),
            ),
            Err(EvalError::Abort | EvalError::ProtectedAbort) => {
                (system_expr("$Aborted"), false, Expr::string("Abort"))
            }
            Err(EvalError::TimeConstraintExceeded) => (
                system_expr("$Aborted"),
                false,
                Expr::string("TimeConstraint"),
            ),
            Err(EvalError::MemoryConstraintExceeded { .. }) => (
                system_expr("$Aborted"),
                false,
                Expr::string("MemoryConstraint"),
            ),
            Err(EvalError::Break) => (system_expr("$Failed"), false, Expr::string("Break")),
            Err(EvalError::Continue) => (system_expr("$Failed"), false, Expr::string("Continue")),
            Err(EvalError::Return { .. }) => {
                (system_expr("$Failed"), false, Expr::string("Return"))
            }
            Err(EvalError::Thrown { .. } | EvalError::ThrownBypassingCatch { .. }) => {
                (system_expr("$Failed"), false, Expr::string("Throw"))
            }
            Err(EvalError::InputRequired { request }) => {
                return Err(EvalError::InputRequired { request });
            }
            Err(EvalError::Exit { code }) => return Err(EvalError::Exit { code }),
            Err(_) => (system_expr("$Failed"), false, Expr::string("Error")),
        };

        Ok(render_association(vec![
            (
                Expr::string("Input"),
                Expr::call(system_expr("Hold"), vec![input_expr.clone()]),
            ),
            (Expr::string("Result"), result_expr),
            (Expr::string("Success"), debug_bool_expr(success)),
            (Expr::string("FailureType"), failure_type),
            (
                Expr::string("Messages"),
                Expr::call(
                    system_expr("List"),
                    messages
                        .iter()
                        .cloned()
                        .map(runtime_message_name_expr)
                        .collect(),
                ),
            ),
            (
                Expr::string("MessagesExpressions"),
                Expr::call(
                    system_expr("List"),
                    messages
                        .iter()
                        .cloned()
                        .map(runtime_message_emit_expr)
                        .collect(),
                ),
            ),
            (
                Expr::string("MessagesText"),
                Expr::call(
                    system_expr("List"),
                    messages
                        .iter()
                        .map(|message| Expr::string(message.text.clone()))
                        .collect(),
                ),
            ),
            (
                Expr::string("OutputLog"),
                Expr::call(
                    system_expr("List"),
                    prints.into_iter().map(Expr::string).collect(),
                ),
            ),
            (Expr::string("Trace"), trace_root),
            (Expr::string("Timing"), elapsed.clone()),
            (Expr::string("AbsoluteTiming"), elapsed),
            (
                Expr::string("InputString"),
                Expr::string(input_string.to_string()),
            ),
        ]))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DebugCaptureCheckpoint {
    message_checkpoint: usize,
    print_checkpoint: usize,
    trace_checkpoint: usize,
    handled_error_checkpoint: usize,
}

impl DebugCaptureCheckpoint {
    fn new(session: &Session) -> Self {
        Self {
            message_checkpoint: session.messages.len(),
            print_checkpoint: session.prints.len(),
            trace_checkpoint: session.trace_events.len(),
            handled_error_checkpoint: session.handled_non_loader_eval_errors.len(),
        }
    }
}
