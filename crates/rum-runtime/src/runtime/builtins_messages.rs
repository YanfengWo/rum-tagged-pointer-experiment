use super::*;

impl Session {
    pub(super) fn eval_message(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let Some(spec) = args.first() else {
            return Ok(self.invalid_call_with_message(
                "Message",
                args,
                "argx",
                "expected at least 1 argument",
            ));
        };

        let Some(key) = self.message_name_key_from_expr(spec) else {
            return Ok(self.invalid_call_with_arg_form(
                "Message",
                args,
                "expected the first argument to be a MessageName expression",
            ));
        };

        let rendered_text = self.render_message_text(&key, &args[1..]);
        let symbol_name = key.symbol.to_string();
        let tag = key.tag.clone();

        self.drive_internal_value::<String, Expr, _>(rendered_text, move |session, text| {
            session.push_message(&symbol_name, &tag, text);
            Ok(system_expr("Null"))
        })
    }

    pub(super) fn eval_message_name(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("MessageName", args, 2));
        }

        let Some(key) = self.resolve_message_name_args(&args[0], &args[1]) else {
            return Ok(self.invalid_call_with_arg_form(
                "MessageName",
                args,
                "expected a symbol or symbol-name string, plus a message tag string or symbol",
            ));
        };

        Ok(message_name_expr(key.symbol, key.tag))
    }

    pub(super) fn eval_messages(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [] => Ok(self.message_names_expr_for_symbol(None)),
            [expr] => {
                if let Some(key) = self.message_name_key_from_expr(expr) {
                    let items = if self.message_definitions.contains_key(&key) {
                        vec![message_name_expr(key.symbol, key.tag)]
                    } else {
                        Vec::new()
                    };
                    Ok(Expr::call(system_expr("List"), items))
                } else if let Some(symbol) = self.lookup_message_symbol(expr) {
                    Ok(self.message_names_expr_for_symbol(Some(&symbol)))
                } else {
                    Ok(self.invalid_call_with_arg_form(
                        "Messages",
                        args,
                        "expected no arguments, a symbol or symbol-name string, or a MessageName expression",
                    ))
                }
            }
            _ => Ok(self.invalid_call_with_message(
                "Messages",
                args,
                "argx",
                format!("expected 0 or 1 argument(s), got {}", args.len()),
            )),
        }
    }

    pub(super) fn eval_message_definitions(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [] => Ok(self.message_definitions_expr_for_symbol(None)),
            [expr] => {
                if let Some(key) = self.message_name_key_from_expr(expr) {
                    Ok(self.message_definitions_expr_for_key(&key))
                } else if let Some(symbol) = self.lookup_message_symbol(expr) {
                    Ok(self.message_definitions_expr_for_symbol(Some(&symbol)))
                } else {
                    Ok(self.invalid_call_with_arg_form(
                        "MessageDefinitions",
                        args,
                        "expected no arguments, a symbol or symbol-name string, or a MessageName expression",
                    ))
                }
            }
            _ => Ok(self.invalid_call_with_message(
                "MessageDefinitions",
                args,
                "argx",
                format!("expected 0 or 1 argument(s), got {}", args.len()),
            )),
        }
    }

    pub(super) fn eval_message_tags(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [] => Ok(self.message_tags_expr_for_symbol(None)),
            [expr] => {
                if let Some(key) = self.message_name_key_from_expr(expr) {
                    Ok(self.message_tags_expr_for_key(&key))
                } else if let Some(symbol) = self.lookup_message_symbol(expr) {
                    Ok(self.message_tags_expr_for_symbol(Some(&symbol)))
                } else {
                    Ok(self.invalid_call_with_arg_form(
                        "MessageTags",
                        args,
                        "expected no arguments, a symbol or symbol-name string, or a MessageName expression",
                    ))
                }
            }
            _ => Ok(self.invalid_call_with_message(
                "MessageTags",
                args,
                "argx",
                format!("expected 0 or 1 argument(s), got {}", args.len()),
            )),
        }
    }
}
