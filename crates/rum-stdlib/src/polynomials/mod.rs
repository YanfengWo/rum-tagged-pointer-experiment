use std::collections::BTreeSet;

use rum_core::{Atom, BigInt, Expr, Rational, SYSTEM_CONTEXT};
use rum_runtime::kernel_api::{
    EvalError, Session, compare_expr_canonical, expr_as_exact_integer, expr_as_exact_rational,
    expr_as_non_negative_usize, expr_as_symbol, is_system_symbol_name, system_expr,
};

mod candidates;
mod core;
mod factorization;
mod rational_function;

use self::{
    core::{Monomial, Polynomial, parse_coefficient_list_monomials},
    factorization::PolynomialFactorization,
    rational_function::RationalFunction,
};

pub(crate) fn eval_expand(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if args.len() != 1 {
        return Ok(session.invalid_call_with_exact_arg_count("Expand", args, 1));
    }

    let Some(polynomial) = Polynomial::from_expr(&args[0]) else {
        return Ok(args[0].clone());
    };

    polynomial.to_expr(session)
}

pub(crate) fn eval_coefficient(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if !(2..=3).contains(&args.len()) {
        return Ok(session.invalid_call_with_arg_range("Coefficient", args, 2, 3));
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
        .to_expr(session)
}

pub(crate) fn eval_coefficient_list(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if args.len() != 2 {
        return Ok(session.invalid_call_with_exact_arg_count("CoefficientList", args, 2));
    }

    let Some(polynomial) = Polynomial::from_expr(&args[0]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    let Some(variables) = parse_coefficient_list_monomials(&args[1]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    polynomial.coefficient_list_expr(&variables, session)
}

pub(crate) fn eval_together(
    session: &mut Session,
    head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if args.len() != 1 {
        return Ok(session.invalid_call_with_exact_arg_count("Together", args, 1));
    }

    let Some(rational_function) = RationalFunction::from_expr(&args[0]) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };

    rational_function.to_expr(session)
}

pub(crate) fn eval_factor(
    session: &mut Session,
    _head: &Expr,
    args: &[Expr],
) -> Result<Expr, EvalError> {
    if args.len() != 1 {
        return Ok(session.invalid_call_with_exact_arg_count("Factor", args, 1));
    }

    let Some(polynomial) = Polynomial::from_expr(&args[0]) else {
        return Ok(args[0].clone());
    };

    PolynomialFactorization::from_polynomial(polynomial).to_expr(session)
}

fn expr_contains_pattern_language(expr: &Expr) -> bool {
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
