use super::*;

impl Session {
    pub(super) fn eval_information(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        let [target] = args else {
            return Ok(self.invalid_call_with_arg_range("Information", args, 1, 1));
        };

        let Some(symbol) = self.normalize_symbol_target_expr(target, true) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(self.information_data_expr_for_symbol(&symbol))
    }

    pub(super) fn eval_value_q(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        let [target] = args else {
            return Ok(self.invalid_call_with_exact_arg_count("ValueQ", args, 1));
        };

        let original = target.clone();
        let snapshot = self.capture_evaluation_rollback_snapshot();
        let outcome = self.evaluate(original.clone());
        self.restore_evaluation_rollback_snapshot(snapshot);

        let value_q = match outcome {
            Ok(value) => value != original,
            Err(EvalError::InputRequired { .. }) => true,
            Err(_) => false,
        };

        Ok(system_expr(if value_q { "True" } else { "False" }))
    }

    pub(super) fn eval_own_values(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [expr] => match self.normalize_definition_inspection_target(expr) {
                Ok(symbol) => Ok(self.own_values_expr_for_symbol(&symbol)),
                Err(error) => {
                    Ok(self.invalid_call_with_arg_form("OwnValues", args, error.message()))
                }
            },
            _ => Ok(self.invalid_call_with_exact_arg_count("OwnValues", args, 1)),
        }
    }

    pub(super) fn eval_up_values(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [expr] => match self.normalize_definition_inspection_target(expr) {
                Ok(symbol) => Ok(self.up_values_expr_for_symbol(&symbol)),
                Err(error) => {
                    Ok(self.invalid_call_with_arg_form("UpValues", args, error.message()))
                }
            },
            _ => Ok(self.invalid_call_with_exact_arg_count("UpValues", args, 1)),
        }
    }

    pub(super) fn eval_down_values(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [expr] => match self.normalize_definition_inspection_target(expr) {
                Ok(symbol) => Ok(self.down_values_expr_for_symbol(&symbol)),
                Err(error) => {
                    Ok(self.invalid_call_with_arg_form("DownValues", args, error.message()))
                }
            },
            _ => Ok(self.invalid_call_with_exact_arg_count("DownValues", args, 1)),
        }
    }

    pub(super) fn eval_sub_values(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [expr] => match self.normalize_definition_inspection_target(expr) {
                Ok(symbol) => Ok(self.sub_values_expr_for_symbol(&symbol)),
                Err(error) => {
                    Ok(self.invalid_call_with_arg_form("SubValues", args, error.message()))
                }
            },
            _ => Ok(self.invalid_call_with_exact_arg_count("SubValues", args, 1)),
        }
    }

    pub(super) fn eval_default_values(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [expr] => match self.normalize_definition_inspection_target(expr) {
                Ok(symbol) => Ok(self.default_values_expr_for_symbol(&symbol)),
                Err(error) => {
                    Ok(self.invalid_call_with_arg_form("DefaultValues", args, error.message()))
                }
            },
            _ => Ok(self.invalid_call_with_exact_arg_count("DefaultValues", args, 1)),
        }
    }

    pub(super) fn default_values_expr_for_symbol(&self, symbol: &Symbol) -> Expr {
        let default_symbol = Symbol::system("Default");
        let items = self
            .definitions
            .get(&default_symbol)
            .into_iter()
            .flat_map(|definition| definition.down_values.iter())
            .filter(|entry| default_value_lhs_targets_symbol(&entry.lhs, symbol))
            .map(|entry| {
                Expr::call(
                    system_expr("RuleDelayed"),
                    vec![
                        Expr::call(system_expr("HoldPattern"), vec![entry.lhs.clone()]),
                        entry.rhs.clone(),
                    ],
                )
            })
            .collect::<Vec<_>>();

        Expr::call(system_expr("List"), items)
    }

    fn information_data_expr_for_symbol(&self, symbol: &Symbol) -> Expr {
        let entries = vec![
            information_rule("ObjectType", Expr::string("Symbol")),
            information_rule("Usage", self.usage_message_expr_for_symbol(symbol)),
            information_rule("Documentation", system_expr("None")),
            information_values_rule("OwnValues", symbol, self.own_values_expr_for_symbol(symbol)),
            information_values_rule("UpValues", symbol, self.up_values_expr_for_symbol(symbol)),
            information_values_rule(
                "DownValues",
                symbol,
                self.down_values_expr_for_symbol(symbol),
            ),
            information_values_rule("SubValues", symbol, self.sub_values_expr_for_symbol(symbol)),
            information_values_rule(
                "DefaultValues",
                symbol,
                self.default_values_expr_for_symbol(symbol),
            ),
            information_rule("NValues", system_expr("None")),
            information_rule("FormatValues", system_expr("None")),
            information_rule(
                "Options",
                none_if_empty_list(self.options_for_symbol(symbol).expr()),
            ),
            information_rule("Attributes", self.attribute_set_for_symbol(symbol).expr()),
            information_rule("FullName", Expr::string(symbol.fully_qualified_name())),
        ];

        Expr::call(
            system_expr("InformationData"),
            vec![Expr::call(system_expr("Association"), entries)],
        )
    }

    fn usage_message_expr_for_symbol(&self, symbol: &Symbol) -> Expr {
        let key = MessageNameKey {
            symbol: symbol.clone(),
            tag: "usage".to_string(),
        };

        self.message_definitions
            .get(&key)
            .and_then(|definition| {
                definition
                    .immediate
                    .clone()
                    .or_else(|| definition.delayed.clone())
            })
            .unwrap_or_else(|| Expr::string(symbol.fully_qualified_name()))
    }
}

fn information_rule(name: &str, value: Expr) -> Expr {
    Expr::call(system_expr("Rule"), vec![Expr::string(name), value])
}

fn information_values_rule(name: &str, symbol: &Symbol, values: Expr) -> Expr {
    information_rule(name, information_value_form_or_none(name, symbol, values))
}

fn information_value_form_or_none(name: &str, symbol: &Symbol, values: Expr) -> Expr {
    match &values {
        Expr::Call { head, args } if is_system_symbol_name(head, "List") && args.is_empty() => {
            system_expr("None")
        }
        _ => Expr::call(
            Expr::Atom(Atom::Symbol(Symbol::new(
                "Information`",
                "InformationValueForm",
            ))),
            vec![
                system_expr(name),
                Expr::Atom(Atom::Symbol(symbol.clone())),
                values,
            ],
        ),
    }
}

fn none_if_empty_list(expr: Expr) -> Expr {
    match &expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "List") && args.is_empty() => {
            system_expr("None")
        }
        _ => expr,
    }
}
