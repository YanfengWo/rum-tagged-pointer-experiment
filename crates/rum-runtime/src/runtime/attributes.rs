use super::*;

impl Session {
    pub(super) fn eval_attributes(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [expr] => {
                if let Some(symbol) = self.lookup_attribute_target(expr, false) {
                    Ok(self.attribute_set_for_symbol(&symbol).expr())
                } else {
                    Ok(self.invalid_call_with_arg_form(
                        "Attributes",
                        args,
                        "expected a symbol or symbol-name string argument",
                    ))
                }
            }
            _ => Ok(self.invalid_call_with_exact_arg_count("Attributes", args, 1)),
        }
    }

    pub(super) fn eval_set_attributes(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_update_attributes("SetAttributes", args, AttributeUpdate::Insert)
    }

    pub(super) fn eval_clear_attributes(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        self.eval_update_attributes("ClearAttributes", args, AttributeUpdate::Remove)
    }

    pub(super) fn eval_update_attributes(
        &mut self,
        symbol: &str,
        args: &[Expr],
        update: AttributeUpdate,
    ) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count(symbol, args, 2));
        }

        let Some(targets) = self.lookup_attribute_targets(&args[0], true) else {
            return Ok(self.invalid_call_with_arg_form(
                symbol,
                args,
                "expected a symbol, symbol-name string, or list of those as the first argument",
            ));
        };

        let symbol_name = symbol.to_string();
        let resolved_result = self.evaluate(args[1].clone());
        let resolved_attributes = self.suspend_with_frame(resolved_result, {
            let targets = targets.clone();
            let original_args = args.to_vec();
            move |session, resolved| {
                session.resume_update_attributes(
                    &symbol_name,
                    original_args,
                    targets,
                    update,
                    resolved,
                )
            }
        })?;

        self.resume_update_attributes(symbol, args.to_vec(), targets, update, resolved_attributes)
    }

    fn resume_update_attributes(
        &mut self,
        symbol: &str,
        original_args: Vec<Expr>,
        targets: Vec<Symbol>,
        update: AttributeUpdate,
        resolved_attributes: Expr,
    ) -> Result<Expr, EvalError> {
        let Some(attributes) = parse_attribute_set_expr(&resolved_attributes) else {
            return Ok(self.invalid_call_with_arg_form(
                symbol,
                &original_args,
                "expected an attribute symbol or list of attribute symbols as the second argument",
            ));
        };

        for target in targets {
            let mut current = self.attribute_set_for_symbol(&target);

            for attribute in attributes.attributes() {
                match update {
                    AttributeUpdate::Insert => current.insert(attribute),
                    AttributeUpdate::Remove => current.remove(attribute),
                }
            }

            self.set_attribute_set_for_symbol(target, current);
        }

        Ok(system_expr("Null"))
    }
}
