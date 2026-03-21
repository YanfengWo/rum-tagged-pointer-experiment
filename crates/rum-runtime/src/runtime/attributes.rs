use super::*;

#[derive(Clone, Debug, PartialEq, Eq)]
struct AttributeUpdateConfig {
    targets: Vec<Symbol>,
    attributes: AttributeSet,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AttributeUpdateNormalizationError {
    InvalidTargets,
    InvalidAttributes,
}

impl AttributeUpdateNormalizationError {
    fn message(self) -> &'static str {
        match self {
            Self::InvalidTargets => {
                "expected a symbol, symbol-name string, or list of those as the first argument"
            }
            Self::InvalidAttributes => {
                "expected an attribute symbol or list of attribute symbols as the second argument"
            }
        }
    }
}

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

        let targets = match self.normalize_attribute_update_targets(&args[0]) {
            Ok(targets) => targets,
            Err(error) => {
                return Ok(self.invalid_call_with_arg_form(symbol, args, error.message()));
            }
        };

        let symbol_name = symbol.to_string();
        let resolved_result = self.evaluate(args[1].clone());
        let resolved_attributes = self.suspend_with_frame(resolved_result, {
            let targets = targets.clone();
            let original_args = args.to_vec();
            move |session, resolved| {
                session.finish_update_attributes_after_resolved_attributes(
                    &symbol_name,
                    original_args,
                    targets,
                    update,
                    resolved,
                )
            }
        })?;

        self.finish_update_attributes_after_resolved_attributes(
            symbol,
            args.to_vec(),
            targets,
            update,
            resolved_attributes,
        )
    }

    fn finish_update_attributes_after_resolved_attributes(
        &mut self,
        symbol: &str,
        original_args: Vec<Expr>,
        targets: Vec<Symbol>,
        update: AttributeUpdate,
        resolved_attributes: Expr,
    ) -> Result<Expr, EvalError> {
        let config =
            match self.compile_attribute_update_from_resolved_args(targets, &resolved_attributes) {
                Ok(config) => config,
                Err(error) => {
                    return Ok(self.invalid_call_with_arg_form(
                        symbol,
                        &original_args,
                        error.message(),
                    ));
                }
            };

        for target in config.targets {
            let mut current = self.attribute_set_for_symbol(&target);

            for attribute in config.attributes.attributes() {
                match update {
                    AttributeUpdate::Insert => current.insert(attribute),
                    AttributeUpdate::Remove => current.remove(attribute),
                }
            }

            self.set_attribute_set_for_symbol(target, current);
        }

        Ok(system_expr("Null"))
    }

    fn normalize_attribute_update_targets(
        &self,
        expr: &Expr,
    ) -> Result<Vec<Symbol>, AttributeUpdateNormalizationError> {
        self.lookup_attribute_targets(expr, true)
            .ok_or(AttributeUpdateNormalizationError::InvalidTargets)
    }

    fn compile_attribute_update_from_resolved_args(
        &self,
        targets: Vec<Symbol>,
        resolved_attributes: &Expr,
    ) -> Result<AttributeUpdateConfig, AttributeUpdateNormalizationError> {
        let attributes = parse_attribute_set_expr(resolved_attributes)
            .ok_or(AttributeUpdateNormalizationError::InvalidAttributes)?;

        Ok(AttributeUpdateConfig {
            targets,
            attributes,
        })
    }
}
