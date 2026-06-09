use rum_core::{Atom, BigInt, Expr, Rational};

use super::{Precedence, render_expr, symbol_head_name};

pub(super) fn render_plus(args: &[Expr]) -> String {
    if args.is_empty() {
        return "Plus[]".to_string();
    }

    let mut rendered = String::new();
    for (index, arg) in args.iter().enumerate() {
        let (negative, term) = render_plus_term(arg);
        if index == 0 {
            if negative {
                rendered.push('-');
            }
            rendered.push_str(&term);
        } else if negative {
            rendered.push_str(" - ");
            rendered.push_str(&term);
        } else {
            rendered.push_str(" + ");
            rendered.push_str(&term);
        }
    }
    rendered
}

fn render_plus_term(expr: &Expr) -> (bool, String) {
    if let Some(abs_value) = negative_integer_abs(expr) {
        return (true, abs_value.to_string());
    }
    if let Some(abs_value) = negative_rational_abs(expr) {
        return (true, abs_value.to_string());
    }
    if let Expr::Call { head, args } = expr {
        if symbol_head_name(head) == Some("Times") {
            if let Some((abs_first, rest)) = split_negative_first_factor(args) {
                let mut positive_args = Vec::with_capacity(args.len());
                if !is_one_expr(&abs_first) || rest.is_empty() {
                    positive_args.push(abs_first);
                }
                positive_args.extend(rest.iter().cloned());
                return (true, render_times(&positive_args));
            }
        }
    }

    (false, render_expr(expr, Precedence::Plus))
}

pub(super) fn render_times(args: &[Expr]) -> String {
    if args.is_empty() {
        return "Times[]".to_string();
    }

    let mut negative = false;
    let mut numerator = Vec::new();
    let mut denominator = Vec::new();

    for arg in args {
        if let Some(value) = integer_value(arg) {
            let abs_value = if value < &BigInt::from(0_i32) {
                negative = !negative;
                -value.clone()
            } else {
                value.clone()
            };
            if abs_value != BigInt::from(1_i32) {
                numerator.push(abs_value.to_string());
            }
            continue;
        }

        if let Some(value) = rational_value(arg) {
            if value.numerator() < &BigInt::from(0_i32) {
                negative = !negative;
            }
            let abs_numerator = bigint_abs(value.numerator());
            if abs_numerator != BigInt::from(1_i32) {
                numerator.push(abs_numerator.to_string());
            }
            if value.denominator() != &BigInt::from(1_i32) {
                denominator.push(value.denominator().to_string());
            }
            continue;
        }

        if let Some((base, positive_exponent)) = negative_power(arg) {
            denominator.push(render_positive_power(base, positive_exponent));
            continue;
        }

        numerator.push(render_expr(arg, Precedence::Times));
    }

    let numerator_text = if numerator.is_empty() {
        "1".to_string()
    } else {
        numerator.join("*")
    };
    let mut rendered = if denominator.is_empty() {
        numerator_text
    } else {
        format!("{numerator_text}/{}", render_denominator(&denominator))
    };

    if negative {
        rendered = format!("-{rendered}");
    }
    rendered
}

pub(super) fn render_power(base: &Expr, exponent: &Expr) -> String {
    if let Some(value) = integer_value(exponent) {
        if value < &BigInt::from(0_i32) {
            return format!(
                "1/{}",
                render_positive_power(base, Expr::integer(-value.clone()))
            );
        }
    }
    render_positive_power(base, exponent.clone())
}

fn render_positive_power(base: &Expr, exponent: Expr) -> String {
    match integer_value(&exponent) {
        Some(value) if value == &BigInt::from(1_i32) => render_expr(base, Precedence::Power),
        _ => format!(
            "{}^{}",
            render_expr(base, Precedence::Power),
            render_expr(&exponent, Precedence::Power)
        ),
    }
}

fn render_denominator(factors: &[String]) -> String {
    let joined = factors.join("*");
    let already_parenthesized = joined.starts_with('(') && joined.ends_with(')');
    if !already_parenthesized
        && (factors.len() > 1
            || joined.contains('*')
            || joined.contains(" + ")
            || joined.contains(" - "))
    {
        format!("({joined})")
    } else {
        joined
    }
}

fn split_negative_first_factor(args: &[Expr]) -> Option<(Expr, &[Expr])> {
    let (first, rest) = args.split_first()?;
    if let Some(abs_value) = negative_integer_abs(first) {
        return Some((Expr::integer(abs_value), rest));
    }
    if let Some(abs_value) = negative_rational_abs(first) {
        return Some((Expr::rational(abs_value), rest));
    }
    None
}

fn negative_power(expr: &Expr) -> Option<(&Expr, Expr)> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    if symbol_head_name(head) != Some("Power") || args.len() != 2 {
        return None;
    }

    let exponent = integer_value(&args[1])?;
    (exponent < &BigInt::from(0_i32)).then(|| (&args[0], Expr::integer(-exponent.clone())))
}

fn is_one_expr(expr: &Expr) -> bool {
    match expr {
        Expr::Atom(Atom::Integer(value)) => value == &BigInt::from(1_i32),
        Expr::Atom(Atom::Rational(value)) => value.is_one(),
        _ => false,
    }
}

fn negative_integer_abs(expr: &Expr) -> Option<BigInt> {
    let value = integer_value(expr)?;
    (value < &BigInt::from(0_i32)).then(|| -value.clone())
}

fn negative_rational_abs(expr: &Expr) -> Option<Rational> {
    let value = rational_value(expr)?;
    (value.numerator() < &BigInt::from(0_i32)).then(|| {
        Rational::new(-value.numerator().clone(), value.denominator().clone())
            .expect("negating a normalized rational should keep a non-zero denominator")
    })
}

fn integer_value(expr: &Expr) -> Option<&BigInt> {
    match expr {
        Expr::Atom(Atom::Integer(value)) => Some(value),
        _ => None,
    }
}

fn rational_value(expr: &Expr) -> Option<&Rational> {
    match expr {
        Expr::Atom(Atom::Rational(value)) => Some(value),
        _ => None,
    }
}

fn bigint_abs(value: &BigInt) -> BigInt {
    if value < &BigInt::from(0_i32) {
        -value.clone()
    } else {
        value.clone()
    }
}
