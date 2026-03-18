use super::*;

impl Session {
    pub(super) fn eval_own_values(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [expr] => {
                if let Some(symbol) = self.lookup_message_symbol(expr) {
                    Ok(self.own_values_expr_for_symbol(&symbol))
                } else {
                    Ok(self.invalid_call_with_arg_form(
                        "OwnValues",
                        args,
                        "expected a symbol or symbol-name string argument",
                    ))
                }
            }
            _ => Ok(self.invalid_call_with_exact_arg_count("OwnValues", args, 1)),
        }
    }

    pub(super) fn eval_up_values(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [expr] => {
                if let Some(symbol) = self.lookup_message_symbol(expr) {
                    Ok(self.up_values_expr_for_symbol(&symbol))
                } else {
                    Ok(self.invalid_call_with_arg_form(
                        "UpValues",
                        args,
                        "expected a symbol or symbol-name string argument",
                    ))
                }
            }
            _ => Ok(self.invalid_call_with_exact_arg_count("UpValues", args, 1)),
        }
    }

    pub(super) fn eval_down_values(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [expr] => {
                if let Some(symbol) = self.lookup_message_symbol(expr) {
                    Ok(self.down_values_expr_for_symbol(&symbol))
                } else {
                    Ok(self.invalid_call_with_arg_form(
                        "DownValues",
                        args,
                        "expected a symbol or symbol-name string argument",
                    ))
                }
            }
            _ => Ok(self.invalid_call_with_exact_arg_count("DownValues", args, 1)),
        }
    }

    pub(super) fn eval_sub_values(&mut self, args: &[Expr]) -> Result<Expr, EvalError> {
        match args {
            [expr] => {
                if let Some(symbol) = self.lookup_message_symbol(expr) {
                    Ok(self.sub_values_expr_for_symbol(&symbol))
                } else {
                    Ok(self.invalid_call_with_arg_form(
                        "SubValues",
                        args,
                        "expected a symbol or symbol-name string argument",
                    ))
                }
            }
            _ => Ok(self.invalid_call_with_exact_arg_count("SubValues", args, 1)),
        }
    }
}
