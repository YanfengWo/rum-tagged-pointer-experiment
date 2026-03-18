use super::*;

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

        let selectors = if let Some(selector_expr) = args.get(2) {
            let Some(selectors) = parse_message_selectors(selector_expr) else {
                return Ok(self.invalid_call_with_arg_form(
                    "Check",
                    args,
                    "expected the optional message selector to be a symbol, a \"symbol::tag\" string, or a list of those",
                ));
            };
            Some(selectors)
        } else {
            None
        };

        let message_checkpoint = self.messages.len();
        let handled_error_checkpoint = self.handled_non_loader_eval_errors.len();
        let value = match self.evaluate(args[0].clone()) {
            Ok(value) => value,
            Err(err) => {
                self.emit_non_loader_eval_message(handled_error_checkpoint, &err);

                let matched = selectors.as_ref().map_or_else(
                    || self.messages.len() > message_checkpoint,
                    |selectors| self.messages_since_match(message_checkpoint, selectors),
                );

                if matched {
                    return self.evaluate(args[1].clone());
                }

                return Err(err);
            }
        };

        let matched = selectors.as_ref().map_or_else(
            || self.messages.len() > message_checkpoint,
            |selectors| self.messages_since_match(message_checkpoint, selectors),
        );

        self.handled_non_loader_eval_errors
            .truncate(handled_error_checkpoint);

        if matched {
            self.evaluate(args[1].clone())
        } else {
            Ok(value)
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

        let selectors = if let Some(selector_expr) = args.get(1) {
            let Some(selectors) = parse_message_selectors(selector_expr) else {
                return Ok(self.invalid_call_with_arg_form(
                    "Quiet",
                    args,
                    "expected the optional message selector to be a symbol, a \"symbol::tag\" string, or a list of those",
                ));
            };
            Some(selectors)
        } else {
            None
        };

        let message_checkpoint = self.messages.len();
        let handled_error_checkpoint = self.handled_non_loader_eval_errors.len();
        let result = match self.evaluate(args[0].clone()) {
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

        if result.is_ok() {
            self.handled_non_loader_eval_errors
                .truncate(handled_error_checkpoint);
        }

        result
    }

    pub(super) fn eval_if(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if !(2..=4).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("If", args, 2, 4));
        }

        match truthiness_symbol(&args[0]) {
            Some(true) => self.evaluate(args[1].clone()),
            Some(false) => {
                if args.len() >= 3 {
                    self.evaluate(args[2].clone())
                } else {
                    Ok(system_expr("Null"))
                }
            }
            None if args.len() == 4 => self.evaluate(args[3].clone()),
            None => Ok(Expr::call(system_expr("If"), args.to_vec())),
        }
    }

    pub(super) fn eval_which(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.is_empty() || args.len() % 2 != 0 {
            return Ok(self.invalid_call_with_arg_form(
                "Which",
                args,
                "expected one or more condition/result pairs",
            ));
        }

        let mut index = 0;

        while index < args.len() {
            let condition = self.evaluate(args[index].clone())?;

            match truthiness_symbol(&condition) {
                Some(true) => return self.evaluate(args[index + 1].clone()),
                Some(false) => index += 2,
                None => {
                    let mut residual = args.to_vec();
                    residual[index] = condition;
                    return Ok(Expr::call(system_expr("Which"), residual));
                }
            }
        }

        Ok(system_expr("Null"))
    }
}
