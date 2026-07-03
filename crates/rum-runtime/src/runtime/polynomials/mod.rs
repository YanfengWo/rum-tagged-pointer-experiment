use super::*;

mod candidates;
mod core;
mod factorization;
mod rational_function;

use self::{
    core::{Monomial, Polynomial, parse_coefficient_list_monomials},
    factorization::PolynomialFactorization,
    rational_function::RationalFunction,
};

impl Session {
    pub(super) fn eval_expand(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("Expand", args, 1));
        }

        let Some(polynomial) = Polynomial::from_expr(&args[0]) else {
            return Ok(args[0].clone());
        };

        polynomial.to_expr(self)
    }

    pub(super) fn eval_coefficient(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if !(2..=3).contains(&args.len()) {
            return Ok(self.invalid_call_with_arg_range("Coefficient", args, 2, 3));
        }

        let Some(polynomial) = Polynomial::from_expr(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let Some(target) = Monomial::from_expr(&args[1]).filter(|target| !target.is_one()) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        let power = match args.get(2) {
            None => 1,
            Some(arg) => {
                if let Some(value) = expr_as_non_negative_usize(arg) {
                    value
                } else if let Some(value) = expr_as_exact_integer(arg) {
                    if value < BigInt::from(0_u8) {
                        return Ok(Expr::integer(0_i32.into()));
                    }

                    return Ok(Expr::call(head.clone(), args.to_vec()));
                } else {
                    return Ok(Expr::call(head.clone(), args.to_vec()));
                }
            }
        };

        polynomial
            .coefficient_for_monomial_power(&target, power)
            .to_expr(self)
    }

    pub(super) fn eval_coefficient_list(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        if args.len() != 2 {
            return Ok(self.invalid_call_with_exact_arg_count("CoefficientList", args, 2));
        }

        let Some(polynomial) = Polynomial::from_expr(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        let Some(variables) = parse_coefficient_list_monomials(&args[1]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        polynomial.coefficient_list_expr(&variables, self)
    }

    pub(super) fn eval_together(&mut self, head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("Together", args, 1));
        }

        let Some(rational_function) = RationalFunction::from_expr(&args[0]) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };

        rational_function.to_expr(self)
    }

    pub(super) fn eval_factor(&mut self, _head: &Expr, args: &[Expr]) -> Result<Expr, EvalError> {
        if args.len() != 1 {
            return Ok(self.invalid_call_with_exact_arg_count("Factor", args, 1));
        }

        let Some(polynomial) = Polynomial::from_expr(&args[0]) else {
            return Ok(args[0].clone());
        };

        PolynomialFactorization::from_polynomial(polynomial).to_expr(self)
    }

    pub(super) fn eval_polynomial_q(
        &mut self,
        _head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        match args {
            [expr] => Ok(polynomial_predicate_bool_expr(
                is_polynomial_expression_unconstrained(expr),
            )),
            [expr, vars_expr] => {
                let Some(variables) = parse_polynomial_variable_spec(vars_expr) else {
                    return Ok(self.invalid_call_with_arg_form(
                        "PolynomialQ",
                        args,
                        "expected a variable or variable list",
                    ));
                };
                Ok(polynomial_predicate_bool_expr(
                    is_polynomial_expression_in_variables(expr, &variables),
                ))
            }
            _ => Ok(self.invalid_call_with_arg_range("PolynomialQ", args, 1, 2)),
        }
    }

    pub(super) fn eval_polynomial_expression_q(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_polynomial_like_expression_q(
            head,
            args,
            "PolynomialExpressionQ",
            is_polynomial_expression_in_variables,
        )
    }

    pub(super) fn eval_rational_expression_q(
        &mut self,
        head: &Expr,
        args: &[Expr],
    ) -> Result<Expr, EvalError> {
        self.eval_polynomial_like_expression_q(
            head,
            args,
            "RationalExpressionQ",
            is_rational_expression_in_variables,
        )
    }

    fn eval_polynomial_like_expression_q<F>(
        &mut self,
        head: &Expr,
        args: &[Expr],
        symbol: &str,
        predicate: F,
    ) -> Result<Expr, EvalError>
    where
        F: Fn(&Expr, &[Expr]) -> bool,
    {
        match args {
            [expr, vars_expr] => {
                let Some(variables) = parse_polynomial_variable_spec(vars_expr) else {
                    return Ok(self.invalid_call_with_arg_form(
                        symbol,
                        args,
                        "expected a variable or variable list",
                    ));
                };
                Ok(polynomial_predicate_bool_expr(predicate(expr, &variables)))
            }
            [_, _, _] => Ok(Expr::call(head.clone(), args.to_vec())),
            _ => Ok(self.invalid_call_with_arg_range(symbol, args, 2, 3)),
        }
    }
}

fn parse_polynomial_variable_spec(expr: &Expr) -> Option<Vec<Expr>> {
    if let Expr::Call { head, args } = expr {
        if is_system_symbol_name(head, heads::structure::LIST) {
            return args.iter().map(parse_indeterminate_expr).collect();
        }
    }

    parse_indeterminate_expr(expr).map(|variable| vec![variable])
}

fn is_polynomial_expression_unconstrained(expr: &Expr) -> bool {
    if expr_as_exact_rational(expr).is_some() {
        return true;
    }
    if expr_contains_pattern_language(expr) {
        return false;
    }

    match expr {
        Expr::Atom(Atom::String(_)) => false,
        Expr::Atom(_) => true,
        Expr::Call { head, args } if is_system_symbol_name(head, "Plus") => {
            args.iter().all(is_polynomial_expression_unconstrained)
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Times") => {
            args.iter().all(is_polynomial_expression_unconstrained)
        }
        Expr::Call { head, args } if is_system_symbol_name(head, "Power") && args.len() == 2 => {
            if let Some(exponent) = expr_as_non_negative_usize(&args[1]) {
                return exponent == 0 || is_polynomial_expression_unconstrained(&args[0]);
            }

            if expr_as_exact_integer(&args[1]).is_some_and(|exponent| exponent < BigInt::from(0_u8))
            {
                return is_opaque_polynomial_indeterminate(&args[0]);
            }

            true
        }
        Expr::Call { .. } => true,
    }
}

fn is_polynomial_expression_in_variables(expr: &Expr, variables: &[Expr]) -> bool {
    if expr_as_exact_rational(expr).is_some() {
        return true;
    }
    if variables.iter().any(|variable| variable == expr) {
        return true;
    }

    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Plus") => args
            .iter()
            .all(|arg| is_polynomial_expression_in_variables(arg, variables)),
        Expr::Call { head, args } if is_system_symbol_name(head, "Times") => args
            .iter()
            .all(|arg| is_polynomial_expression_in_variables(arg, variables)),
        Expr::Call { head, args } if is_system_symbol_name(head, "Power") && args.len() == 2 => {
            expr_as_non_negative_usize(&args[1]).is_some()
                && is_polynomial_expression_in_variables(&args[0], variables)
        }
        _ => is_valid_polynomial_coefficient(expr, variables),
    }
}

fn is_rational_expression_in_variables(expr: &Expr, variables: &[Expr]) -> bool {
    if is_polynomial_expression_in_variables(expr, variables) {
        return true;
    }

    match expr {
        Expr::Call { head, args } if is_system_symbol_name(head, "Plus") => args
            .iter()
            .all(|arg| is_rational_expression_in_variables(arg, variables)),
        Expr::Call { head, args } if is_system_symbol_name(head, "Times") => args
            .iter()
            .all(|arg| is_rational_expression_in_variables(arg, variables)),
        Expr::Call { head, args } if is_system_symbol_name(head, "Power") && args.len() == 2 => {
            expr_as_exact_integer(&args[1]).is_some()
                && is_rational_expression_in_variables(&args[0], variables)
        }
        _ => is_valid_polynomial_coefficient(expr, variables),
    }
}

fn is_valid_polynomial_coefficient(expr: &Expr, variables: &[Expr]) -> bool {
    !expr_contains_pattern_language(expr)
        && variables
            .iter()
            .all(|variable| !expr_contains_exact_subexpr(expr, variable))
}

fn expr_contains_exact_subexpr(expr: &Expr, needle: &Expr) -> bool {
    if expr == needle {
        return true;
    }

    match expr {
        Expr::Call { head, args } => {
            expr_contains_exact_subexpr(head, needle)
                || args
                    .iter()
                    .any(|arg| expr_contains_exact_subexpr(arg, needle))
        }
        Expr::Atom(_) => false,
    }
}

fn is_opaque_polynomial_indeterminate(expr: &Expr) -> bool {
    if expr_contains_pattern_language(expr) {
        return false;
    }

    match expr {
        Expr::Atom(_) => false,
        Expr::Call { head, .. }
            if is_system_symbol_name(head, "Plus")
                || is_system_symbol_name(head, "Times")
                || is_system_symbol_name(head, "Power") =>
        {
            false
        }
        Expr::Call { .. } => true,
    }
}

fn polynomial_predicate_bool_expr(value: bool) -> Expr {
    system_expr(if value {
        heads::structure::TRUE
    } else {
        heads::structure::FALSE
    })
}

pub(super) fn expr_contains_pattern_language(expr: &Expr) -> bool {
    match expr {
        Expr::Call { head, args } => {
            is_pattern_language_head(head)
                || expr_contains_pattern_language(head)
                || args.iter().any(expr_contains_pattern_language)
        }
        _ => false,
    }
}

fn is_pattern_language_head(expr: &Expr) -> bool {
    matches!(
        expr_as_symbol(expr),
        Some(symbol)
            if symbol.context() == SYSTEM_CONTEXT
                && matches!(
                    symbol.name(),
                    "Alternatives"
                        | "Blank"
                        | "BlankNullSequence"
                        | "BlankSequence"
                        | "Condition"
                        | "Except"
                        | "HoldPattern"
                        | "Optional"
                        | "OrderlessPatternSequence"
                        | "Pattern"
                        | "PatternSequence"
                        | "PatternTest"
                        | "Repeated"
                        | "RepeatedNull"
                        | "Verbatim"
                )
    )
}

pub fn parse_indeterminate_expr(expr: &Expr) -> Option<Expr> {
    if expr_contains_pattern_language(expr) {
        return None;
    }

    match expr {
        Expr::Atom(Atom::Integer(_))
        | Expr::Atom(Atom::Rational(_))
        | Expr::Atom(Atom::String(_)) => None,
        Expr::Call { head, .. }
            if is_system_symbol_name(head, "Plus")
                || is_system_symbol_name(head, "Times")
                || is_system_symbol_name(head, "Power") =>
        {
            None
        }
        _ => Some(expr.clone()),
    }
}
