use super::*;

impl Session {
    pub(in crate::runtime) fn eval_assert(&mut self, args: &SharedArgs) -> Result<Expr, EvalError> {
        let test = args.first().cloned();

        let function_result = self.evaluate(system_expr("$AssertFunction"));
        let function = self.suspend_with_frame_lazy(function_result, || {
            let test = test.clone();
            let args = args.clone();
            move |session, function| {
                session.resume_assert_after_function(test, args.as_ref().to_vec(), function)
            }
        })?;

        self.resume_assert_after_function(test, args.as_ref().to_vec(), function)
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
        let outcome = self.suspend_with_result_frame_lazy(test_result, || {
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
