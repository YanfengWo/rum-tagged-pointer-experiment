use super::*;

impl Session {
    pub(in crate::runtime) fn try_apply_function(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Option<Expr>, EvalError> {
        let applied = if let Some(function) = parse_function(head) {
            match function {
                FunctionSpec::SlotBased { body, .. } => substitute_slots(body, args),
                FunctionSpec::Explicit { params, body, .. } => {
                    if params.len() != args.len() {
                        self.push_message(
                            "Function",
                            "argx",
                            format!(
                                "expected exactly {} argument(s), got {}",
                                params.len(),
                                args.len()
                            ),
                        );
                        return Ok(Some(Expr::call(head.clone(), args.to_vec())));
                    }

                    let bindings = params.into_iter().zip(args.iter().cloned()).collect();
                    substitute_lexical_bindings(body, &bindings, &BTreeSet::new())
                }
            }
        } else if let Some(composition) = parse_composition(head) {
            apply_composition_to_args(&composition, args)
        } else if let Some(status) = self.apply_status_object_to_args(head, args)? {
            status
        } else if let Some(value) = apply_association_to_args(head, args) {
            value
        } else if let Some(invalid_key_application) =
            self.invalid_callable_key_application(head, args)
        {
            return Ok(Some(invalid_key_application));
        } else if let Some(wrapper) = apply_callable_operator_wrapper_to_args(head, args) {
            wrapper
        } else if let Some(symbol) = invalid_callable_operator_wrapper_argx_symbol(head, args) {
            self.push_message(
                symbol,
                "argx",
                format!(
                    "{head} called with {} arguments; 1 argument is expected.",
                    args.len()
                ),
            );
            return Ok(Some(Expr::call(head.clone(), args.to_vec())));
        } else if let Some(wrapper) = apply_callable_failsafe_wrapper_to_args(head, args) {
            wrapper
        } else {
            return Ok(None);
        };

        let applied_result = self.evaluate(applied);
        let outcome = self.suspend_with_result_frame(applied_result, |session, outcome| {
            session.finish_function_application(outcome)
        });

        match outcome {
            Err(EvalError::InputRequired { request }) => Err(EvalError::InputRequired { request }),
            outcome => self.finish_function_application(outcome).map(Some),
        }
    }

    fn finish_function_application(
        &mut self,
        outcome: Result<Expr, EvalError>,
    ) -> Result<Expr, EvalError> {
        match outcome {
            Ok(value) => Ok(value),
            Err(EvalError::Return { value }) => Ok(value),
            Err(error) => Err(error),
        }
    }

    fn invalid_callable_key_application(&mut self, head: &Expr, args: &[Expr]) -> Option<Expr> {
        let Expr::Call {
            head: key_head,
            args: key_args,
        } = head
        else {
            return None;
        };
        if !is_system_symbol_name(key_head, "Key") || key_args.len() != 1 {
            return None;
        }

        let held = Expr::call(head.clone(), args.to_vec());
        let [expr] = args else {
            self.push_message(
                "Key",
                "argx",
                format!(
                    "{head} called with {} arguments; 1 argument is expected.",
                    args.len()
                ),
            );
            return Some(held);
        };

        if association_entries_from_expr(expr).is_none() {
            self.push_message("Key", "invak", invalid_association_message(expr));
            return Some(held);
        }

        None
    }
}
