use super::*;

impl Session {
    pub(in crate::runtime) fn eval_string_reverse(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("StringReverse", args, 1));
        }

        let Some(value) = expr_as_string(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::string(value.chars().rev().collect::<String>()))
    }

    pub(in crate::runtime) fn eval_to_lower_case(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("ToLowerCase", args, 1));
        }

        let Some(value) = expr_as_string(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        Ok(Expr::string(
            value
                .chars()
                .flat_map(char::to_lowercase)
                .collect::<String>(),
        ))
    }

    pub(in crate::runtime) fn eval_lower_case_q(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        Ok(
            self.eval_string_scalar_predicate(head, args, "LowerCaseQ", |value| {
                value.chars().all(char::is_lowercase)
            }),
        )
    }

    pub(in crate::runtime) fn eval_upper_case_q(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        Ok(
            self.eval_string_scalar_predicate(head, args, "UpperCaseQ", |value| {
                value.chars().all(char::is_uppercase)
            }),
        )
    }

    pub(in crate::runtime) fn eval_capitalize(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_string_or_string_list_map(head, args, "Capitalize", capitalize_value)
    }

    pub(in crate::runtime) fn eval_decapitalize(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_string_or_string_list_map(head, args, "Decapitalize", decapitalize_value)
    }

    fn eval_string_scalar_predicate(
        &mut self,
        head: &Expr,
        args: &[Expr],
        symbol: &str,
        predicate: fn(&str) -> bool,
    ) -> Expr {
        if args.len() != 1 {
            return self.invalid_call_with_exact_arg_count(symbol, args, 1);
        }

        let Some(value) = expr_as_string(&args[0]) else {
            return Expr::call(head.clone(), args.to_vec());
        };

        bool_expr(predicate(value))
    }

    fn eval_string_or_string_list_map(
        &mut self,
        head: &Expr,
        args: &[Expr],
        symbol: &str,
        mapper: fn(&str) -> String,
    ) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count(symbol, args, 1));
        }

        if let Some(value) = expr_as_string(&args[0]) {
            return Ok(Expr::string(mapper(value)));
        }

        if let Some(values) = expr_as_string_list(&args[0]) {
            return Ok(string_list_expr(
                values.into_iter().map(|value| mapper(&value)).collect(),
            ));
        }

        Ok(Expr::call(head.clone(), args.to_vec()))
    }
}

fn capitalize_value(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };

    first.to_uppercase().chain(chars).collect()
}

fn decapitalize_value(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };

    first.to_lowercase().chain(chars).collect()
}
