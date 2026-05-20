use super::*;

impl Session {
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
}
