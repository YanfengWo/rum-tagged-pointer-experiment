use rum_core::{BigInt, Expr, Rational};

use super::render_input_form_like;

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
fn renders_assignment_forms_like_input_form() {
    let set = Expr::call(
        Expr::system_symbol("Set"),
        vec![
            Expr::symbol("x"),
            Expr::call(
                Expr::system_symbol("Plus"),
                vec![Expr::symbol("x"), Expr::integer(BigInt::from(1_i32))],
            ),
        ],
    );
    assert_eq!(render_input_form_like(&set), "x = x + 1");

    let delayed_rule_with_set = Expr::call(
        Expr::system_symbol("RuleDelayed"),
        vec![Expr::symbol("a"), set],
    );
    assert_eq!(
        render_input_form_like(&delayed_rule_with_set),
        "a :> (x = x + 1)"
    );
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
