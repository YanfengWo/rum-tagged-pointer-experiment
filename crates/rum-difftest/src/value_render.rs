use rum_core::{Atom, BigInt, Expr, Rational};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Precedence {
    Lowest = 0,
    Plus = 1,
    Times = 2,
    Power = 3,
    Atom = 4,
}

pub fn render_input_form_like(expr: &Expr) -> String {
    render_expr(expr, Precedence::Lowest)
}

fn render_expr(expr: &Expr, parent_precedence: Precedence) -> String {
    let precedence = expr_precedence(expr);
    let rendered = match expr {
        Expr::Atom(atom) => atom.to_string(),
        Expr::Call { head, args } => render_call(head, args),
    };

    if precedence < parent_precedence {
        format!("({rendered})")
    } else {
        rendered
    }
}

fn expr_precedence(expr: &Expr) -> Precedence {
    match expr {
        Expr::Atom(_) => Precedence::Atom,
        Expr::Call { head, .. } => match symbol_head_name(head) {
            Some("Plus") => Precedence::Plus,
            Some("Times") => Precedence::Times,
            Some("Power") => Precedence::Power,
            _ => Precedence::Atom,
        },
    }
}

fn render_call(head: &Expr, args: &[Expr]) -> String {
    match symbol_head_name(head) {
        Some("List") => render_wrapped("{", "}", args),
        Some("Association") => render_wrapped("<|", "|>", args),
        Some("Rule") if args.len() == 2 => render_infix(&args[0], " -> ", &args[1]),
        Some("RuleDelayed") if args.len() == 2 => render_infix(&args[0], " :> ", &args[1]),
        Some("Plus") => render_plus(args),
        Some("Times") => render_times(args),
        Some("Power") if args.len() == 2 => render_power(&args[0], &args[1]),
        Some("MessageName") if args.len() == 2 => render_message_name(&args[0], &args[1]),
        Some("Blank") => render_blank("_", args),
        Some("BlankSequence") => render_blank("__", args),
        Some("BlankNullSequence") => render_blank("___", args),
        Some("Pattern") if args.len() == 2 => render_pattern(&args[0], &args[1]),
        _ => {
            let rendered_args = args
                .iter()
                .map(|arg| render_expr(arg, Precedence::Lowest))
                .collect::<Vec<_>>()
                .join(", ");
            format!("{}[{rendered_args}]", render_expr(head, Precedence::Lowest))
        }
    }
}

fn render_blank(prefix: &str, args: &[Expr]) -> String {
    match args {
        [] => prefix.to_string(),
        [head] => format!("{prefix}{}", render_expr(head, Precedence::Atom)),
        _ => render_head_form(prefix_to_blank_head(prefix), args),
    }
}

fn render_pattern(name: &Expr, pattern: &Expr) -> String {
    if let (Some(name), Some(blank)) = (symbol_head_name(name), render_blank_like(pattern)) {
        format!("{name}{blank}")
    } else {
        render_head_form("Pattern", &[name.clone(), pattern.clone()])
    }
}

fn render_blank_like(expr: &Expr) -> Option<String> {
    let Expr::Call { head, args } = expr else {
        return None;
    };
    match symbol_head_name(head) {
        Some("Blank") => Some(render_blank("_", args)),
        Some("BlankSequence") => Some(render_blank("__", args)),
        Some("BlankNullSequence") => Some(render_blank("___", args)),
        _ => None,
    }
}

fn prefix_to_blank_head(prefix: &str) -> &str {
    match prefix {
        "_" => "Blank",
        "__" => "BlankSequence",
        "___" => "BlankNullSequence",
        _ => unreachable!("only known blank prefixes are rendered"),
    }
}

fn render_head_form(head: &str, args: &[Expr]) -> String {
    let rendered_args = args
        .iter()
        .map(|arg| render_expr(arg, Precedence::Lowest))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{head}[{rendered_args}]")
}

fn render_message_name(symbol: &Expr, tag: &Expr) -> String {
    let Some(tag) = message_tag_text(tag) else {
        let rendered_args = [symbol, tag]
            .iter()
            .map(|arg| render_expr(arg, Precedence::Lowest))
            .collect::<Vec<_>>()
            .join(", ");
        return format!("MessageName[{rendered_args}]");
    };

    format!("{}::{tag}", render_expr(symbol, Precedence::Atom))
}

fn message_tag_text(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Atom(Atom::String(tag)) => Some(tag.clone()),
        Expr::Atom(Atom::Symbol(tag)) => Some(tag.name().to_string()),
        _ => None,
    }
}

fn render_wrapped(open: &str, close: &str, args: &[Expr]) -> String {
    let body = args
        .iter()
        .map(|arg| render_expr(arg, Precedence::Lowest))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{open}{body}{close}")
}

fn render_infix(lhs: &Expr, op: &str, rhs: &Expr) -> String {
    format!(
        "{}{op}{}",
        render_expr(lhs, Precedence::Lowest),
        render_expr(rhs, Precedence::Lowest)
    )
}

fn render_plus(args: &[Expr]) -> String {
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

fn render_times(args: &[Expr]) -> String {
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

fn render_power(base: &Expr, exponent: &Expr) -> String {
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

fn symbol_head_name(expr: &Expr) -> Option<&str> {
    match expr {
        Expr::Atom(Atom::Symbol(symbol)) => Some(symbol.name()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn int(value: i64) -> Expr {
        Expr::integer(BigInt::from(value))
    }

    fn rational(numerator: i64, denominator: i64) -> Expr {
        Expr::rational(Rational::new(BigInt::from(numerator), BigInt::from(denominator)).unwrap())
    }

    fn plus(args: Vec<Expr>) -> Expr {
        Expr::call(Expr::system_symbol("Plus"), args)
    }

    fn times(args: Vec<Expr>) -> Expr {
        Expr::call(Expr::system_symbol("Times"), args)
    }

    fn power(base: Expr, exponent: Expr) -> Expr {
        Expr::call(Expr::system_symbol("Power"), vec![base, exponent])
    }

    #[test]
    fn renders_lists_and_nested_lists_like_input_form() {
        let expr = Expr::call(
            Expr::system_symbol("List"),
            vec![
                Expr::symbol("a"),
                Expr::call(
                    Expr::system_symbol("List"),
                    vec![Expr::symbol("b"), Expr::symbol("c")],
                ),
            ],
        );

        assert_eq!(render_input_form_like(&expr), "{a, {b, c}}");
    }

    #[test]
    fn renders_rules_and_associations_like_input_form() {
        let expr = Expr::call(
            Expr::system_symbol("Association"),
            vec![
                Expr::call(
                    Expr::system_symbol("Rule"),
                    vec![Expr::symbol("a"), Expr::symbol("lhs")],
                ),
                Expr::call(
                    Expr::system_symbol("RuleDelayed"),
                    vec![Expr::symbol("b"), Expr::symbol("rhs")],
                ),
            ],
        );

        assert_eq!(render_input_form_like(&expr), "<|a -> lhs, b :> rhs|>");
    }

    #[test]
    fn renders_blank_patterns_like_input_form() {
        let typed_blank = Expr::call(
            Expr::system_symbol("Blank"),
            vec![Expr::system_symbol("String")],
        );
        assert_eq!(render_input_form_like(&typed_blank), "_String");

        let sequence_blank = Expr::call(Expr::system_symbol("BlankSequence"), Vec::new());
        assert_eq!(render_input_form_like(&sequence_blank), "__");

        let named_typed_blank = Expr::call(
            Expr::system_symbol("Pattern"),
            vec![
                Expr::symbol("x"),
                Expr::call(
                    Expr::system_symbol("BlankNullSequence"),
                    vec![Expr::system_symbol("Integer")],
                ),
            ],
        );
        assert_eq!(render_input_form_like(&named_typed_blank), "x___Integer");
    }

    #[test]
    fn renders_message_name_like_input_form() {
        let held_message = Expr::call(
            Expr::system_symbol("HoldForm"),
            vec![Expr::call(
                Expr::system_symbol("MessageName"),
                vec![Expr::symbol("f"), Expr::string("argx")],
            )],
        );

        assert_eq!(render_input_form_like(&held_message), "HoldForm[f::argx]");
    }

    #[test]
    fn renders_plus_times_and_power_like_input_form() {
        let expanded = plus(vec![int(-1), power(Expr::symbol("x"), int(2))]);
        assert_eq!(render_input_form_like(&expanded), "-1 + x^2");

        let cubic_terms = plus(vec![
            power(Expr::symbol("x"), int(3)),
            power(Expr::symbol("y"), int(3)),
            times(vec![
                int(3),
                Expr::symbol("x"),
                power(Expr::symbol("y"), int(2)),
            ]),
        ]);
        assert_eq!(render_input_form_like(&cubic_terms), "x^3 + y^3 + 3*x*y^2");
    }

    #[test]
    fn renders_negative_terms_rationals_and_reciprocals_like_input_form() {
        let difference = plus(vec![
            Expr::symbol("x"),
            times(vec![int(-1), Expr::symbol("y")]),
        ]);
        assert_eq!(render_input_form_like(&difference), "x - y");

        let rational_term = times(vec![rational(1, 6), Expr::symbol("x")]);
        assert_eq!(render_input_form_like(&rational_term), "x/6");

        let quotient = times(vec![
            power(times(vec![Expr::symbol("x"), Expr::symbol("y")]), int(-1)),
            plus(vec![Expr::symbol("x"), Expr::symbol("y")]),
        ]);
        assert_eq!(render_input_form_like(&quotient), "(x + y)/(x*y)");
    }
}
