use super::*;

impl Session {
    pub(super) fn eval_input_string(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let prompt = match args {
            [] => String::new(),
            [Expr::Atom(Atom::String(prompt))] => prompt.clone(),
            [_] => {
                return Ok(self.invalid_call_with_arg_form(
                    "InputString",
                    args,
                    "expected zero arguments or one prompt string",
                ));
            }
            _ => return Ok(self.invalid_call_with_arg_range("InputString", args, 0, 1)),
        };

        if let Some(value) = self.queued_input_responses.pop_front() {
            self.clear_suspension_state();
            return Ok(Expr::string(value));
        }

        let request = RuntimeInputRequest::new(prompt);
        self.begin_input_suspension(request.clone());

        Err(EvalError::InputRequired { request })
    }
}
